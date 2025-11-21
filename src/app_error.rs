use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO Error: '{0}'")]
    Io(#[from] std::io::Error),
    #[error("Internal error: '{0}'")]
    Internal(String),
    #[error("missing env: '{0}'")]
    MissingEnv(String),
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("WS Connect: {0}")]
    TungsteniteConnect(String),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
