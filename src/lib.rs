mod error;
mod inner_tcp;
mod inner_udp;
mod inner_websocket;
use std::{fmt::Debug};

use async_trait::async_trait;
use error::ConnectError;
use inner_tcp::InnerTcpConn;
use inner_udp::InnerUdpConn;
pub use inner_websocket::InnerWebsocket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    TCP,
    UDP,
    WEBSOCKET,
}

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

pub struct ConnBuilderConfig {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub error_callback: Box<dyn FnMut(String) + Send + Sync>,
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
        Protocol::TCP
    }
}

pub trait ConnNew {
    fn new(config: ConnBuilderConfig) -> Self;
}

#[async_trait]
pub trait Conn: Send + Sync {
    fn get_address(&self) -> String;
    fn clone_box(&self) -> Box<dyn Conn>;
    async fn connect(&mut self) -> Result<bool, ConnectError>;
    async fn disconnect(&mut self) -> Result<bool, ConnectError>;
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError>;
    async fn receive(&mut self) -> Result<Vec<u8>, ()>;
}

pub trait ConnectionTrait: ConnNew + Sync + Send + Clone {}

pub struct Connection(Box<dyn Conn>);

impl Clone for Connection {
    fn clone(&self) -> Self {
        Connection(self.0.clone_box())
    }
}

unsafe impl Sync for Connection {}
unsafe impl Send for Connection {}

impl Connection {
    pub fn get_address(&self) -> String {
        return self.0.get_address();
    }
    pub async fn connect(&mut self) -> Result<bool, ConnectError> {
        return self.0.connect().await;
    }
    pub async fn disconnect(&mut self) -> Result<bool, ConnectError> {
        return self.0.disconnect().await;
    }
    pub async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError> {
        return self.0.send(data).await;
    }
    pub async fn receive(&mut self) -> Result<Vec<u8>, ()> {
        println!("receive");
        return self.0.receive().await;
    }
}

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

    pub fn build(self) -> Connection {
        match self {
            ConnBuilder::TCP(config) => Connection(Box::new(InnerTcpConn::new(config))),
            ConnBuilder::UDP(config) => Connection(Box::new(InnerUdpConn::new(config))),
            ConnBuilder::WEBSOCKET(config) => Connection(Box::new(InnerWebsocket::new(config))),
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
            protocol: Protocol::WEBSOCKET,
            error_callback: Box::new(|ERR: String| {
                println!("ERR: {}", ERR);
            }),
        };

        let mut conn = ConnBuilder::new(connect_opt).build();
        conn.connect().await.unwrap();
        loop {
            // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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

    #[tokio::test]
    async fn it_test_kind_of_connect() {
        let connect_opt = ConnBuilderConfig {
            host: "127.0.0.1".to_string(),
            port: 9673,
            protocol: Protocol::WEBSOCKET,
            error_callback: Box::new(|ERR: String| {
                println!("ERR: {}", ERR);
            }),
        };

        let mut conn = ConnBuilder::new(connect_opt).build();
        conn.connect().await;
    }
}
