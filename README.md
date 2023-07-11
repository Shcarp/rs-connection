### Connections
#### Description
This crate is used to create a connection to a server. It can be used to create a connection to a server using the TCP or WebSocket protocol. The connection is created using the tokio library. (TCP and UDP is not supported yet)
#### Example
```
use conn::{ConnBuilderConfig, Protocol, ConnBuilder};

#[tokio::main]
async fn main() {
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
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
```
#### Functions
##### ConnBuilderConfig
###### Description
This struct is used to configure the connection.
###### Fields
* host: String - The host to connect to.
* port: u16 - The port to connect to.
* protocol: Protocol - The protocol to use.
* error_callback: Box<Fn(String)> - The callback to call when an error occurs.
