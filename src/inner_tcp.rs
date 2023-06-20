use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::{net::TcpStream, sync::mpsc::Receiver};

use crate::{error::ConnectError, Conn, ConnBuilderConfig, Protocol, ConnNew};

#[derive(Debug, Clone)]
pub struct InnerTcpConn {
    pub ip: String,
    pub port: u16,
    pub protocol: Protocol,
    stream: Option<Arc<TcpStream>>,
}

unsafe impl Send for InnerTcpConn {}
unsafe impl Sync for InnerTcpConn {}

impl ConnNew for InnerTcpConn {
    fn new(config: ConnBuilderConfig) -> Self {
        Self {
            ip: config.host.clone(),
            port: config.port,
            protocol: Protocol::TCP,
            stream: None,
        }
    }
}

#[async_trait]
impl Conn for InnerTcpConn {
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
