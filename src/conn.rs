use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use rs_event_emitter::Handle;

use crate::{base::{ConnectionInterface, Emitter, ConnectionBaseInterface}, ConnectError};

pub trait ConnAssemble: ConnectionInterface + Emitter + ConnectionBaseInterface {}

pub struct Conn (Box<dyn ConnAssemble>);

impl Conn {
    pub fn new<T: ConnAssemble + 'static>(conn: T) -> Self {
        Conn(Box::new(conn))
    }
}

#[async_trait]
impl ConnectionInterface for Conn {
    async fn connect(&mut self) -> Result<bool, ConnectError> {
        self.0.connect().await
    }
    async fn disconnect(&mut self) -> Result<bool, ConnectError> {
        self.0.disconnect().await
    }
    async fn send(&mut self, data: &[u8]) -> Result<bool, ConnectError> {
        self.0.send(data).await
    }
    async fn receive(&mut self) -> Result<Vec<u8>, ()> {
        self.0.receive().await
    }
}

impl Emitter for Conn {
    fn emit(&mut self, event: &'static str, data: Box<dyn Any>) -> () {
        self.0.emit(event, data);
    }

    fn on(&mut self, event: &'static str, callback: Arc<dyn Handle>) -> () {
        self.0.on(event, callback);
    }

    fn off(&mut self, event: &'static str, callback: Arc<dyn Handle>) -> () {
        self.0.off(event, callback);
    }
}

impl ConnectionBaseInterface for Conn {
    fn get_address(&self) -> String {
        self.0.get_address()
    }
}
