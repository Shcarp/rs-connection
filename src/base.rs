use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use rs_event_emitter::Handle;

use crate::ConnectError;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    TCP,
    UDP,
    WEBSOCKET,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::WEBSOCKET
    }
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

pub static CONNECTING_EVENT: &str = "connecting";
pub static CONNECTED_EVENT: &str = "connected";
pub static CLOSE_EVENT: &str = "close";
pub static DISCONNECT_EVENT: &str = "disconnect";
pub static ERROR_EVENT: &str = "error";
pub static MESSAGE_EVENT: &str = "message";
pub static RECONNECT_EVENT: &str = "reconnect";

pub trait Emitter: Send + Sync {
    fn emit(&mut self, event: &'static str, data: Box<dyn Any>) -> ();

    fn on(&mut self, event: &'static str, callback: Arc<dyn Handle>) -> ();

    fn off(&mut self, event: &'static str, callback: Arc<dyn Handle>) -> ();
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
