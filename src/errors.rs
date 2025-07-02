use thiserror::Error;


#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Failed to subscribe: {0}")]
    SubscriptionFailed(String),

    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Unknown error: {0}")]
    Unknown(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum OrderPlaceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("HMAC error: {0}")]
    HmacError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_urlencoded::ser::Error),

    #[error("Other error: {0}")]
    Other(String),
}