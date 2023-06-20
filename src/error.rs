use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectError {
    #[error("Not connected")]
    Disconnect(String),
    #[error("Connection error")]
    Connection(#[from] std::io::Error),
    #[error("Send error {0}")]
    SendError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("unknown data store error")]
    Unknown,
}