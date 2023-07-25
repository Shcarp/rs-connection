use std::{fmt::Debug, sync::Arc, any::Any};
use crate::{base::Protocol, HEARTBEAT_INTERVAL, conn::Conn, inner_websocket::InnerWebsocket};

pub struct ConnBuilderConfig {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub heartbeat_time: Option<u64>,
}

impl Default for ConnBuilderConfig  {
    fn default() -> Self {
        ConnBuilderConfig {
            host: "127.0.0.1".to_owned(),
            port: 9673,
            protocol: Protocol::WEBSOCKET,
            heartbeat_time: Some(HEARTBEAT_INTERVAL),
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

    pub fn build(self) -> Conn {
        match self {
            ConnBuilder::WEBSOCKET(config) => Conn::new(InnerWebsocket::new(config)),
            _ => panic!("not support"),
        }
    }
}

