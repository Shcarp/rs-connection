# rs-connections

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Table of Contents

-   [Introduction](#introduction)
-   [Features](#features)
-   [Installation](#installation)
-   [Usage](#usage)
-   [API Documentation](#api-documentation)
-   [Contributing](#contributing)
-   [License](#license)
x
## Introduction

This crate is used to create a connection to a server. It can be used to create a connection to a server using the TCP or WebSocket protocol. The connection is created using the tokio library. (TCP and UDP is not supported yet)

## Features

-   Support for TCP, UDP, and WebSocket protocols.
-   Connection state management (Init, Connecting, Connected, Closed, Closing, Reconnect).
-   Event handling for various connection-related events.

## Installation

Add the following line to your `Cargo.toml` file:

```toml
[dependencies]
rs-connections = "0.2.1"
```

## Usage

```rust
use rs_connections::{ConnBuilderConfig, Protocol, ConnBuilder, ConnectError, ConnectionInterface};

#[tokio::main]
async fn main() {
    let connect_opt = ConnBuilderConfig {
        host: "127.0.0.1".to_string(),
        port: 9673,
        heartbeat_time: Some(10000),  // Optional heartbeat time
        protocol: Protocol::WEBSOCKET,
    };

    // Create a connection using the ConnBuilder
    let mut conn = ConnBuilder::new(connect_opt).build();
}
```

### Connecting to a Server

To establish a connection to the server:

```rust
async fn connect_to_server() {
    let mut conn = // ... create and configure the connection
    conn.connect().await.unwrap();
}
```

### Handling Connection Events

You can listen for different connection events using the Emitter trait:

#### Base usage

```rust
use rs_connections::{
    ConnBuilder, ConnBuilderConfig, ConnectionInterface, Emitter, EventHandler, Protocol,
    CONNECTED_EVENT,
};

#[tokio::main]
async fn main() {
    let connect_opt = ConnBuilderConfig {
        host: "127.0.0.1".to_string(),
        port: 9673,
        heartbeat_time: Some(10000),
        protocol: Protocol::WEBSOCKET,
    };

    let mut conn = ConnBuilder::new(connect_opt).build();
    conn.connect().await.unwrap();

    let my_event = EventHandler::new(|data: &str| {
        println!("event connecting: {}", data);
    });

    conn.on("my_event", my_event.clone());
    // Emit the event
    // The parameter types must match
    conn.emit("my_event", "hello world");

    // Remove the event
    conn.off("my_event", my_event);

    // more parameters, use tuple
    conn.on("my_event", |data: (&str, &str)| {
        println!("event connecting: {}", data.0);
        println!("event connecting: {}", data.1);
    })
    
    conn.emit("my_event", ("hello", "world"));
}

```

#### Provided basic events are:

```rust
pub static CONNECTING_EVENT: &str = "connecting";
pub static CONNECTED_EVENT: &str = "connected";
pub static CLOSE_EVENT: &str = "close";
pub static DISCONNECT_EVENT: &str = "disconnect";
pub static ERROR_EVENT: &str = "error";
pub static MESSAGE_EVENT: &str = "message";
pub static RECONNECT_EVENT: &str = "reconnect";
```

-   `connecting`: Emitted when the connection is being established.
    -   listen parameter: `&str`
    -   emit parameter: `&str`
-   `connected`: Emitted when the connection is established.
    -   listen parameter: `&str`
    -   emit parameter: `&str`
-   `disconnect`: Emitted when the connection is disconnect.
    -   listen parameter: `&str`
    -   emit parameter: `&str`
-   `reconnect`: Emitted when the connection is re-established.
    -   listen parameter: `String`
    -   emit parameter: `String`
-   `close`: Emitted when the connection is close.
    -   listen parameter: `String`
    -   emit parameter: `String`
-   `error`: Emitted when an error occurs.
    -   listen parameter: `String`
    -   emit parameter: `String`
-   `message`: Emitted when a message is received.
    -   listen parameter:
        -   `String` Text message
        -   `Vec<u8>` Binary message
    -   emit parameter: `&str`
        -   `String` Test message
        -   `Vec<u8>` Binary message

#### Example

```rust
use rs_connections::{ConnBuilderConfig, Protocol, ConnBuilder, ConnectError, ConnectionInterface};

#[tokio::main]
async fn main() {
    let connect_opt = ConnBuilderConfig {
        host: "127.0.0.1".to_string(),
        port: 9673,
        heartbeat_time: Some(10000),  // Optional heartbeat time
        protocol: Protocol::WEBSOCKET,
    };

    let mut conn = ConnBuilder::new(connect_opt).build();

    let handle_connecting = EventHandler::new(|data: &str| {
            println!("event connecting: {}", data);
    });

    let handle_connected = EventHandler::new(|data: &str| {
        println!("event connected: {}", data);
    });

    let handle_disconnect = EventHandler::new(|data: &str| {
        println!("event disconnect: {}", data);
    });

    let handle_reconnect = EventHandler::new(|data: String| {
        println!("event reconnect: {}", data);
    });

    let handle_close = EventHandler::new(|data: String| {
        println!("event close: {}", data);
    });

    let handle_error = EventHandler::new(|data: String| {
        println!("event error: {}", data);
    });

    let handle_text_message = EventHandler::new(|data: String| {
        println!("event message: {}", data);
    });
    let handle_binary_message = EventHandler::new(|data: Vec<u8>| {
        println!("event binary message: {:?}", data);
    });
    // Add event listener

    // add connecting event listener
    conn.on(CONNECTING_EVENT, handle_connecting.clone());
    // add connected event listener
    conn.on(CONNECTED_EVENT, handle_connected.clone());
    // add disconnect event listener
    conn.on(DISCONNECT_EVENT, handle_disconnect.clone());
    // add reconnect event listener
    conn.on(RECONNECT_EVENT, handle_reconnect.clone());
    // add close event listener
    conn.on(CLOSE_EVENT, handle_close.clone());
    // add error event listener
    conn.on(ERROR_EVENT, handle_error.clone());
    // add message event listener
    conn.on(MESSAGE_EVENT, handle_text_message.clone());
    // add binary message event listener
    conn.on(MESSAGE_EVENT, handle_binary_message.clone());

    conn.connect().await.unwrap();

}
```

### Sending Messages

You can send messages to the server using the `send` method:

```rust
async fn send_message() {
    let mut conn = // ... create and configure the connection
    conn.connect().await.unwrap();

    conn.send("Hello World").await.unwrap();
}
```

### Receiving Messages

You can receive messages from the server using the `receive` method or use message event:

```rust
async fn receive_message() {
    let mut conn = // ... create and configure the connection
    conn.connect().await.unwrap();

    loop {
        match conn.receive().await {
            Ok(data) => {
                // data is String or Vec<u8>
                println!("receive");
            },
            Err(_) => {
                println!("receive err");
            },
        }
    }
}

async fn receive_message() {
    let mut conn = // ... create and configure the connection
    conn.connect().await.unwrap();

    conn.on(MESSAGE_EVENT, |data: String| {
        println!("Received message: {}", data);
    });

    conn.on(MESSAGE_EVENT, |data: Vec<u8>| {
        println!("Received binary message: {:?}", data);
    });

}
```

### Closing the Connection

You can close the connection using the `disconnect` method:

```rust
async fn close_connection() {
    let mut conn = // ... create and configure the connection
    conn.connect().await.unwrap();

    conn.disconnect().await.unwrap();
}
```

## API Documentation

You can find the API documentation [here](https://docs.rs/rs-connections/).

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request if you have a way to improve this project.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
```