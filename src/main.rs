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

    let handle_connected = EventHandler::new(|data: &str| {
        println!("event connecting: {}", data);
    });

    conn.on(CONNECTED_EVENT, handle_connected.clone());

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
