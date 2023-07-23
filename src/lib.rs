mod error;
mod inner_tcp;
mod inner_udp;
mod inner_websocket;
use std::fmt::Debug;
use async_trait::async_trait;
pub use rs_event_emitter::{EventHandler, Handle};
pub use error::ConnectError;
pub use inner_websocket::InnerWebsocket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    TCP,
    UDP,
    WEBSOCKET,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    ConnectStateInit,
    ConnectStateConnecting,
    ConnectStateConnected,
    ConnectStateClosed,
    ConnectStateClosing,
    ConnectStateReconnect,
}

impl From<u8> for ConnectionStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => ConnectionStatus::ConnectStateInit,
            1 => ConnectionStatus::ConnectStateConnecting,
            2 => ConnectionStatus::ConnectStateConnected,
            3 => ConnectionStatus::ConnectStateClosed,
            4 => ConnectionStatus::ConnectStateClosing,
            5 => ConnectionStatus::ConnectStateReconnect,
            _ => ConnectionStatus::ConnectStateInit,
        }
    }
}

impl Into<u8> for ConnectionStatus {
    fn into(self) -> u8 {
        match self {
            ConnectionStatus::ConnectStateInit => 0,
            ConnectionStatus::ConnectStateConnecting => 1,
            ConnectionStatus::ConnectStateConnected => 2,
            ConnectionStatus::ConnectStateClosed => 3,
            ConnectionStatus::ConnectStateClosing => 4,
            ConnectionStatus::ConnectStateReconnect => 5,
        }
    }
}

pub const HEARTBEAT_INTERVAL: u64 = 10 * 1000;

pub struct ConnBuilderConfig {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub heartbeat_time: Option<u64>,
    // pub error_callback: Box<dyn FnMut(ConnectError) + Send + Sync>,
}

impl Default for ConnBuilderConfig  {
    fn default() -> Self {
        ConnBuilderConfig {
            host: "127.0.0.1".to_owned(),
            port: 9673,
            protocol: Protocol::WEBSOCKET,
            heartbeat_time: Some(HEARTBEAT_INTERVAL),
            // error_callback: Box::new(|err: ConnectError| {
            //     println!("ERR: {}", err);
            // })
        }
    }
}

impl Debug for ConnBuilderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnBuilderConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .finish()
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::WEBSOCKET
    }
}

pub static CONNECTING_EVENT: &str = "connecting";
pub static CONNECTED_EVENT: &str = "connected";
pub static CLOSE_EVENT: &str = "close";
pub static DISCONNECT_EVENT: &str = "disconnect";
pub static ERROR_EVENT: &str = "error";
pub static MESSAGE_EVENT: &str = "message";
pub static RECONNECT_EVENT: &str = "reconnect";

pub trait Emitter: Send + Sync {
    fn emit<T: Clone + 'static + Debug>(&mut self, event: &'static str, data: T) -> ();

    fn on(&mut self, event: &'static str, callback: impl Handle + 'static) -> ();

    fn off(&mut self, event: &'static str, callback: &(impl Handle + 'static)) -> ();
}

#[async_trait]
pub trait ConnectionInterface: Send + Sync {
    async fn connect(&mut self) -> Result<bool, ConnectError>;
    async fn disconnect(&mut self) -> Result<bool, ConnectError>;
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError>;
    async fn receive(&mut self) -> Result<Vec<u8>, ()>;
}

pub trait ConnectionBaseInterface {
    fn get_address(&self) -> String;
}

pub trait ConnNew {
    fn new(config: ConnBuilderConfig) -> Self;
}

pub trait ConnectionTrait: ConnNew + Sync + Send + Clone {}

pub enum ConnBuilder {
    TCP(ConnBuilderConfig),
    UDP(ConnBuilderConfig),
    WEBSOCKET(ConnBuilderConfig),
}

impl ConnBuilder {
    pub fn new(config: ConnBuilderConfig) -> Self {
        match config.protocol {
            Protocol::TCP => ConnBuilder::TCP(config),
            Protocol::UDP => ConnBuilder::UDP(config),
            Protocol::WEBSOCKET => ConnBuilder::WEBSOCKET(config),
        }
    }

    pub fn build(self) -> impl ConnectionInterface + Emitter + Clone + ConnectionBaseInterface {
        match self {
            ConnBuilder::WEBSOCKET(config) => InnerWebsocket::new(config),
            _ => panic!("not support"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_test_connect() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
            heartbeat_time: Some(HEARTBEAT_INTERVAL),
            protocol: Protocol::WEBSOCKET,
        };

        let mut conn = ConnBuilder::new(connect_opt).build();

        conn.connect().await.unwrap();
        // conn.send(data.as_bytes()).await.unwrap();
        loop {
            match conn.receive().await {
                Ok(_) => {
                    println!("receive");
                },
                Err(_) => {
                    println!("receive err");
                },
            }
        }
    }

    #[tokio::test]
    async fn it_test_event() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
            heartbeat_time: Some(HEARTBEAT_INTERVAL),
            protocol: Protocol::WEBSOCKET,
        };

        let mut conn = ConnBuilder::new(connect_opt).build();

        let handle_connecting = EventHandler::new(|data: &str| {
            println!("event connecting: {}", data);
        });

        let handle_connected = EventHandler::new(|data: &str| {
            println!("event connected: {}", data);
        });

        let handle_close = EventHandler::new(|data: String| {
            println!("event close: {}", data);
        });

        let handle_disconnect = EventHandler::new(|data: &str| {
            println!("event disconnect: {}", data);
        });

        let handle_error = EventHandler::new(|data: String| {
            println!("event error: {}", data);
        });

        let handle_test_message = EventHandler::new(|data: String| {
            println!("event message: {}", data);
        });

        let handle_binary_message = EventHandler::new(|data: Vec<u8>| {
            println!("event binary message: {:?}", data);
        });

        let handle_reconnect = EventHandler::new(|data: String| {
            println!("event reconnect: {}", data);
        });

        conn.on(CONNECTING_EVENT, handle_connecting.clone());
        conn.on(CONNECTED_EVENT, handle_connected.clone());
        conn.on(CLOSE_EVENT, handle_close.clone());
        conn.on(DISCONNECT_EVENT, handle_disconnect.clone());
        conn.on(ERROR_EVENT, handle_error.clone());
        conn.on(MESSAGE_EVENT, handle_test_message.clone());
        conn.on(MESSAGE_EVENT, handle_binary_message.clone());
        conn.on(RECONNECT_EVENT, handle_reconnect.clone());

        conn.connect().await.unwrap();

        loop {
            println!("send");
            match conn.receive().await {
                Ok(_) => {
                    println!("receive");
                },
                Err(_) => {
                    println!("receive err");
                },
            }
        }
    }

}