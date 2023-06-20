use crate::{error::ConnectError, Conn, ConnBuilderConfig, ConnNew, ConnectionStatus, Protocol};
use async_trait::async_trait;
use std::{
    fmt::Debug,
    net::TcpStream,
    sync::{atomic::AtomicU64, Arc},
    sync::{
        atomic::{AtomicU8, Ordering},
        RwLock,
    },
};
use tokio::{sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
}};
use websocket::{
    sync::{Reader, Writer},
    ClientBuilder, OwnedMessage,
};

const HEARTBEAT_INTERVAL: u64 = 10 * 1000;

const PING: &[u8] = b"ping";

pub struct InnerWebsocket {
    pub ip: String,
    pub port: u16,
    pub protocol: Protocol,
    state: Arc<AtomicU8>,
    reader: Option<Arc<Mutex<Reader<TcpStream>>>>,
    writer: Option<Arc<Mutex<Writer<TcpStream>>>>,
    last_heartbeat: Arc<AtomicU64>,
    recv_sender: Sender<Vec<u8>>,
    recv_receiver: Arc<Mutex<Receiver<Vec<u8>>>>,
    conn_task: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

unsafe impl Send for InnerWebsocket {}
unsafe impl Sync for InnerWebsocket {}

impl Clone for InnerWebsocket {
    fn clone(&self) -> Self {
        Self {
            ip: self.ip.clone(),
            port: self.port.clone(),
            protocol: self.protocol.clone(),
            state: self.state.clone(),
            reader: self.reader.clone(),
            writer: self.writer.clone(),
            last_heartbeat: self.last_heartbeat.clone(),
            conn_task: self.conn_task.clone(),
            recv_sender: self.recv_sender.clone(),
            recv_receiver: self.recv_receiver.clone(),
        }
    }
}

impl Debug for InnerWebsocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Websocket")
            .field("host", &self.ip)
            .field("port", &self.port)
            .field("protocol", &self.protocol)
            .finish()
    }
}

impl Drop for InnerWebsocket {
    fn drop(&mut self) {
        let mut tasks = self.conn_task.write().unwrap();
        tasks.iter().for_each(|task| {
            task.abort();
        });
        tasks.clear();
        drop(tasks);
    }
}

impl InnerWebsocket {
    fn get_state(&self) -> u8 {
        self.state.load(Ordering::Relaxed)
    }

    async fn error_callback(&mut self, error: String) {
        // let mut callback = self.error_callback.lock().await;
        // callback(error);
    }

    // 开始发送心跳信息
    fn start_heartbeat(&self) {
        let heartbeat = self.writer.clone().unwrap();

        let mut start_time = chrono::Utc::now().timestamp_millis() as u64;

        let heartbeat_task = tokio::spawn(async move {
            loop {
                let mut write = heartbeat.lock().await;
                let now = chrono::Utc::now().timestamp_millis() as u64;
                if now - start_time >= HEARTBEAT_INTERVAL {
                    write.send_message(&websocket::Message::ping(PING)).unwrap();
                    start_time = chrono::Utc::now().timestamp_millis() as u64;
                }
                drop(write);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
        self.conn_task.write().unwrap().push(heartbeat_task);
    }

    // 检测最后一次收到信息时间差
    fn check_close(&mut self) {
        let mut this = self.clone();
        let last_time = self.last_heartbeat.clone();
        let check_task = tokio::spawn(async move {
            loop {
                let now = chrono::Utc::now().timestamp_millis() as u64;
                if now - last_time.load(Ordering::Relaxed) >= HEARTBEAT_INTERVAL + 1000 {
                    // 重连
                    match this.reconnect().await {
                        Ok(_) => continue,
                        Err(_) => {
                            this.error_callback("Connection already closed".to_owned())
                                .await;
                            return;
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });

        self.conn_task.write().unwrap().push(check_task);
    }

    // 开始接收信息
    fn start_recv(&mut self) {
        let mut this = self.clone();
        let reader = self.reader.clone().unwrap();
        let recv_sender = self.recv_sender.clone();

        std::thread::spawn(move || {
            // 创建一个tokio 运行时
            let rt = tokio::runtime::Runtime::new().unwrap();
            let task = async {
                loop {
                    let message = reader.lock().await.recv_message();
                    let message = match message {
                        Ok(message) => {
                            this.last_heartbeat.store(
                                chrono::Utc::now().timestamp_millis() as u64,
                                Ordering::Relaxed,
                            );
                            match message {
                                OwnedMessage::Ping(payload) => {
                                    this.writer
                                        .clone()
                                        .unwrap()
                                        .lock()
                                        .await
                                        .send_message(&websocket::Message::pong(payload))
                                        .unwrap();
                                    continue;
                                }
                                OwnedMessage::Pong(_) => {
                                    continue;
                                }
                                OwnedMessage::Text(text) => {
                                    recv_sender.send(text.into_bytes()).await.unwrap();
                                }
                                OwnedMessage::Binary(binary) => {
                                    println!("binary: {:?}", binary);
                                    recv_sender
                                        .send(binary)
                                        .await
                                        .or_else(|err| {
                                            Err({
                                                println!("send error");
                                            })
                                        })
                                        .unwrap();
                                }
                                OwnedMessage::Close(_) => {
                                    this.error_callback("Connection already closed".to_owned())
                                        .await;
                                    return;
                                }
                                _ => {}
                            }
                        }
                        Err(_) => {
                        }
                    };
                    drop(message);
                }
            };
            rt.block_on(task);
        });
    }

    async fn reconnect(&mut self) -> Result<(), ()> {
        if (self.get_state() == ConnectionStatus::ConnectStateClosed as u8)
            || (self.get_state() == ConnectionStatus::ConnectStateClosing as u8)
            || (self.get_state() == ConnectionStatus::ConnectStateReconnect as u8)
        {
            return Ok(());
        }
        let mut count = 0;
        self.state.store(
            ConnectionStatus::ConnectStateReconnect as u8,
            Ordering::Relaxed,
        );
        loop {
            self.conn_task.write().unwrap().clear();
            std::thread::sleep(std::time::Duration::from_secs(10));
            let (reader, writer) =
                match ClientBuilder::new(&format!("ws://{}:{}", self.ip, self.port))
                    .unwrap()
                    .connect_insecure()
                {
                    Ok(conn) => conn.split().unwrap(),
                    Err(error) => {
                        println!("连接失败: {}", error);
                        count += 1;
                        if count >= 10 {
                            self.state.store(
                                ConnectionStatus::ConnectStateClosing.into(),
                                Ordering::Relaxed,
                            );
                            return Err(());
                        }
                        continue;
                    }
                };

            let mut reader_guard = self.reader.as_mut().unwrap().lock().await;
            let mut writer_guard = self.writer.as_mut().unwrap().lock().await;

            let old_reader = std::mem::replace(&mut *reader_guard, reader);
            let old_writer = std::mem::replace(&mut *writer_guard, writer);

            drop(old_reader);
            drop(old_writer);

            self.state.store(
                ConnectionStatus::ConnectStateConnected.into(),
                Ordering::Relaxed,
            );
            return Ok(());
        }
    }
}

impl ConnNew for InnerWebsocket {
    fn new(target: ConnBuilderConfig) -> Self {
        let (sender, receiver) = channel::<Vec<u8>>(20);
        InnerWebsocket {
            ip: target.host,
            port: target.port,
            protocol: Protocol::WEBSOCKET,
            state: Arc::new(AtomicU8::new(ConnectionStatus::ConnectStateInit.into())),
            reader: None,
            writer: None,
            last_heartbeat: Arc::new(AtomicU64::new(0)),
            conn_task: Arc::new(RwLock::new(Vec::new())),
            // error_callback: Arc::new(Mutex::new(target.error_callback)),
            recv_sender: sender,
            recv_receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[async_trait]
impl Conn for InnerWebsocket {
    fn clone_box(&self) -> Box<dyn Conn> {
        Box::new(self.clone())
    }

    fn get_address(&self) -> String {
        return format!("{}:{}", self.ip, self.port);
    }

    async fn connect(&mut self) -> Result<bool, ConnectError> {
        self.state.store(
            ConnectionStatus::ConnectStateConnecting.into(),
            Ordering::Relaxed,
        );
        let origin_conn = ClientBuilder::new(&format!("ws://{}:{}", self.ip, self.port))
            .or_else(|err| {
                self.state.store(
                    ConnectionStatus::ConnectStateClosed.into(),
                    Ordering::Relaxed,
                );
                Err(ConnectError::ConnectionError(err.to_string()))
            })?
            .connect_insecure()
            .or_else(|err| {
                self.state.store(
                    ConnectionStatus::ConnectStateClosed.into(),
                    Ordering::Relaxed,
                );
                Err(ConnectError::ConnectionError(err.to_string()))
            })?;

        let (reader, writer) = origin_conn
            .split()
            .or_else(|err| Err(ConnectError::ConnectionError(err.to_string())))?;
        let reader = Arc::new(Mutex::new(reader));
        let writer = Arc::new(Mutex::new(writer));

        self.last_heartbeat.store(
            chrono::Utc::now().timestamp_millis() as u64,
            Ordering::Relaxed,
        );

        self.reader = Some(reader);
        self.writer = Some(writer);

        self.start_heartbeat();
        self.check_close();
        self.start_recv();

        self.state.swap(
            ConnectionStatus::ConnectStateConnected.into(),
            Ordering::Relaxed,
        );
        Ok(true)
    }

    async fn disconnect(&mut self) -> Result<bool, ConnectError> {
        self.state.store(
            ConnectionStatus::ConnectStateClosing.into(),
            Ordering::Relaxed,
        );
        let mut send = self.writer.as_mut().unwrap().lock().await;
        let mut tasks = self.conn_task.write().unwrap();
        tasks.iter().for_each(|task| {
            task.abort();
        });
        tasks.clear();
        drop(tasks);
        match send.send_message(&websocket::Message::close()) {
            Ok(_) => {
                self.state.store(
                    ConnectionStatus::ConnectStateClosed.into(),
                    Ordering::Relaxed,
                );
                Ok(true)
            }
            Err(err) => {
                drop(send);
                Err(ConnectError::Disconnect(err.to_string()))
            }
        }
    }

    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError> {
        if let Some(writer) = self.writer.as_mut() {
            let mut send = writer.lock().await;
            match send.send_message(&websocket::Message::binary(data)) {
                Ok(_) => return Ok(true),
                Err(err) => {
                    drop(send);
                    if self.get_state() == ConnectionStatus::ConnectStateConnected.into() {
                        // 重连
                        match self.reconnect().await {
                            Ok(_) => match self.send(data).await {
                                Ok(_) => return Ok(true),
                                Err(_) => {
                                    return Err(ConnectError::SendError(
                                        "重连后发送失败".to_string(),
                                    ));
                                }
                            },
                            Err(_) => {
                                self.state.store(
                                    ConnectionStatus::ConnectStateClosed.into(),
                                    Ordering::Relaxed,
                                );
                                return Err(ConnectError::SendError("重连失败".to_string()));
                            }
                        }
                    }
                    return Err(ConnectError::SendError(err.to_string()));
                }
            }
        }
        Err(ConnectError::SendError("发送失败".to_string()))
    }

    async fn receive(&mut self) -> Result<Vec<u8>, ()> {
        let mut recv = self.recv_receiver.lock().await;
        match recv.recv().await {
            Some(data) => {
                return Ok(data);
            }
            None => {
                return Ok(Vec::new());
            }
        };
    }
}
