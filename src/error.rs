use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ConnectError {
    #[error("Not connected")]
    Disconnect(String),
    #[error("Connection error")]
    Connection(String),
    #[error("Send error {0}")]
    SendError(String),
    #[error("Recv error {0}")]
    RecvError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),
    #[error("Connection timeout")]
    ConnectionTimeout,
    #[error("Connection refused")]
    ConnectionRefused,
    #[error("Connection reset")]
    ConnectionReset,
    // 重连中
    #[error("Reconnecting")]
    Reconnecting,
    // 重连失败
    #[error("Reconnect failed")]
    ReconnectFailed,
    #[error("unknown data store error: {0}")]
    Unknown(String),
}