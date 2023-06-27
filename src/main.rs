use conn::{ConnBuilderConfig, Protocol, ConnBuilder, ConnectError};

#[tokio::main]
async fn main() {
    let connect_opt = ConnBuilderConfig {
        host: "127.0.0.1".to_string(),
        port: 9673,
        protocol: Protocol::WEBSOCKET,
        error_callback: Box::new(|ERR: ConnectError| {
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