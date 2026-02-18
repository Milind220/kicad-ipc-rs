use thiserror::Error;

#[derive(Debug, Error)]
pub enum KiCadError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("protocol error: {0}")]
    Protocol(String),
}
