use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO Error")]
    Io(#[from] std::io::Error),
    #[error("Internal error: '{0}'")]
    Internal(String),
    #[error("missing env: '{0}'")]
    MissingEnv(String),
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("Internal Database Error: {0}")]
    TungsteniteConnect(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
