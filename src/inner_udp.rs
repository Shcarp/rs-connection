use std::sync::Arc;

use async_trait::async_trait;
use tokio::{net::TcpStream, sync::broadcast::Receiver};
use websocket::receiver;

use crate::{Conn, ConnBuilderConfig, Protocol, error::ConnectError, ConnNew};

#[derive(Debug, Clone)]
pub struct InnerUdpConn {
    pub ip: String,
    pub port: u16,
    pub protocol: Protocol,
    stream: Option<Arc<TcpStream>>,
}

unsafe impl Send for InnerUdpConn {}
unsafe impl Sync for InnerUdpConn {}

impl ConnNew for InnerUdpConn {
    fn new(config: ConnBuilderConfig) -> Self {
        Self {
            ip: config.host.clone(),
            port: config.port,
            protocol: Protocol::UDP,
            stream: None,
        }
    }
}

#[async_trait]
impl Conn for InnerUdpConn {
    fn clone_box(&self) -> Box<dyn Conn> {
        Box::new(self.clone())
    }

    fn get_address(&self) -> String {
        return format!("{}:{}", self.ip, self.port);
    }


    async fn connect(&mut self) -> Result<bool, ConnectError> {
        return Ok(true);
    }
    async fn disconnect(&mut self) -> Result<bool, ConnectError> {
        return Ok(true);
    }
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError> {
        return Ok(true);
    }
    async fn receive(&mut self) -> Result<Vec<u8>, ()>  {
        return Ok(vec![0]);
    }
}
