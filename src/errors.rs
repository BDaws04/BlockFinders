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
