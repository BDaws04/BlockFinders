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

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Error, Debug)]
pub enum OrderPlaceError {
    #[error("HTTP error: {0}")]
    Http(reqwest::Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("HMAC error: {0}")]
    HmacError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_urlencoded::ser::Error),

    #[error("Network error: {0}")]
    NetworkError(reqwest::Error),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum OrderBookError {
    #[error("Failed to receive order: {0}")]
    ReceiveError(String),

    #[error("Failed to process order: {0}")]
    ProcessError(String),

    #[error("Order book is empty")]
    EmptyOrderBook,

    #[error("Unknown error: {0}")]
    Unknown(#[from] std::io::Error),

    #[error("Insufficient volume: {0}")]
    InsufficientVolume(String),

    #[error("Channel send error")]
    ChannelSendError,

    #[error("Inactive order book")]
    InactiveOrderBook,
}