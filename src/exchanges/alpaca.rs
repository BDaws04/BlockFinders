use crate::exchanges::exchange::Exchange;
use crate::errors::{ExchangeError, OrderPlaceError};
use crate::types::{Order, OrderSide};
use crate::config::{
    ORDER_BOOK_DEPTH,
    TICKER,
};
use serde::Serialize;
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};
use base64::{engine::general_purpose, Engine as _};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::connect_async;
use std::collections::HashMap;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering}
};

#[derive(Serialize)]
pub struct OrderRequest {
    symbol: String,
    qty: String,
    side: String,
    type_: String,
    time_in_force: String,
}

pub struct AlpacaExchange {
    api_key: String,
    api_secret: String,
    order_url: String,
    websocket_url: String,
    client: Client,
    active: Arc<AtomicBool>,
    fees: f64, 
}

impl AlpacaExchange {
    pub fn new(api_key: String, api_secret: String) -> Self {
        AlpacaExchange {
            api_key,
            api_secret,
            order_url: "https://api.alpaca.markets/v2/orders".to_string(),
            websocket_url: "wss://stream.data.alpaca.markets/v2/iex".to_string(),
            client: Client::new(),
            active: Arc::new(AtomicBool::new(false)),
            fees: 0.0
        }
    }
}

#[async_trait::async_trait]
impl Exchange for AlpacaExchange {
    async fn subscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {
        unimplemented!();
    }
    async fn unsubscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {
        unimplemented!();
    }
    async fn place_order(&self, order: Order) -> Result<(), OrderPlaceError> {
        let pair = format!("{}/USD", order.symbol);
        let url = &self.order_url;

        let oq = OrderRequest {
            symbol: pair,
            qty: order.volume.to_string(),
            side: match order.side {
                OrderSide::Buy => "buy".to_string(),
                OrderSide::Sell => "sell".to_string(),
            },
            type_: "market".to_string(),
            time_in_force: "gtc".to_string(),
        };

        let res = self.client
            .post(url)
            .header("Apca-Api-Key-Id", &self.api_key)
            .header("Apca-Api-Secret-Key", &self.api_secret)
            .header("Content-Type", "application/json")
            .json(&oq)
            .send()
            .await
            .map_err(OrderPlaceError::Http)?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(OrderPlaceError::Other(
                format!("Failed to place order, response code: {}", res.status())
            ))
        }
    }
}