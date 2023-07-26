mod base;
mod builder;
mod conn;
mod error;
mod inner_tcp;
mod inner_udp;
mod inner_websocket;

pub use error::ConnectError;
pub use rs_event_emitter::{EventHandler, Handle};

pub use base::{
    ConnectionInterface, Emitter, Protocol, ConnectionBaseInterface, CLOSE_EVENT, CONNECTED_EVENT, CONNECTING_EVENT,
    DISCONNECT_EVENT, ERROR_EVENT, MESSAGE_EVENT, RECONNECT_EVENT,
};

pub use builder::{ConnBuilder, ConnBuilderConfig};

pub use conn::Conn;

pub const HEARTBEAT_INTERVAL: u64 = 10 * 1000;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        base::{
            ConnectionInterface, Emitter, Protocol, CLOSE_EVENT, CONNECTED_EVENT, CONNECTING_EVENT,
            DISCONNECT_EVENT, ERROR_EVENT, MESSAGE_EVENT, RECONNECT_EVENT,
        },
        builder::{ConnBuilder, ConnBuilderConfig},
    };

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
                }
                Err(_) => {
                    println!("receive err");
                }
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

        let handle_connecting = EventHandler::new(Box::new(|data: &str| {
            println!("event connecting: {}", data);
        }));

        let handle_connected = EventHandler::new(Box::new(|data: &str| {
            println!("event connected: {}", data);
        }));

        let handle_close = EventHandler::new(Box::new(|data: String| {
            println!("event close: {}", data);
        }));

        let handle_disconnect = EventHandler::new(Box::new(|data: &str| {
            println!("event disconnect: {}", data);
        }));

        let handle_error = EventHandler::new(Box::new(|data: ConnectError| {
            println!("event error: {}", data);
        }));

        let handle_test_message = EventHandler::new(Box::new(|data: String| {
            println!("event message: {}", data);
        }));

        let handle_binary_message = EventHandler::new(Box::new(|data: Vec<u8>| {
            println!("event binary message: {:?}", data);
        }));

        let handle_reconnect = EventHandler::new(Box::new(|data: String| {
            println!("event reconnect: {}", data);
        }));

        conn.on(CONNECTING_EVENT, Arc::new(handle_connecting.clone()));
        conn.on(CONNECTED_EVENT, Arc::new(handle_connected.clone()));
        conn.on(CLOSE_EVENT, Arc::new(handle_close.clone()));
        conn.on(DISCONNECT_EVENT, Arc::new(handle_disconnect.clone()));
        conn.on(ERROR_EVENT, Arc::new(handle_error.clone()));
        conn.on(MESSAGE_EVENT, Arc::new(handle_test_message.clone()));
        conn.on(MESSAGE_EVENT, Arc::new(handle_binary_message.clone()));
        conn.on(RECONNECT_EVENT, Arc::new(handle_reconnect.clone()));

        conn.connect().await.unwrap();

        loop {
            match conn.receive().await {
                Ok(_) => {
                    println!("receive");
                }
                Err(_) => {
                    println!("receive err");
                }
            }
        }
    }
}
