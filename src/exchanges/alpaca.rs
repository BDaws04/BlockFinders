use crate::exchanges::exchange::Exchange;
use crate::errors::{ExchangeError, OrderPlaceError};
use crate::types::{Order, OrderSide};
use crate::config::{
    TICKER,
};
use serde::Serialize;
use reqwest::Client;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::connect_async;
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
            websocket_url: "wss://stream.data.alpaca.markets/v1beta3/crypto/us".to_string(),
            client: Client::new(),
            active: Arc::new(AtomicBool::new(false)),
            fees: 0.0
        }
    }
}

#[async_trait::async_trait]
impl Exchange for AlpacaExchange {
    async fn subscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {
        if symbol != TICKER {
            return Err(ExchangeError::InvalidSymbol(symbol.to_string()));
        }

        let symbol_str = format!("{}/USD", symbol);

        let auth_message = serde_json::json!({
            "action": "auth",
            "key": self.api_key,
            "secret": self.api_secret,
        });

        let json_auth = serde_json::to_string(&auth_message)
            .map_err(|e| ExchangeError::SubscriptionFailed(format!("Failed to serialize subscribe message: {}", e)))?;

        let (mut socket, _) = connect_async(&self.websocket_url).await?;

        if let Some(result) = socket.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if !text.contains("success") {
                        return Err(ExchangeError::SubscriptionFailed("Authentication failed".to_string()));
                    }
                    println!("Successfully connected");
                }
                Ok(Message::Close(_)) => {
                    return Err(ExchangeError::ConnectionClosed);
                }
                Err(e) => {
                    return Err(ExchangeError::WebSocketError(e));
                }
                _ => {}
            }
        }

        socket.send(Message::Text(json_auth.into())).await?;

        if let Some(result) = socket.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if !text.contains("authenticated") {
                        return Err(ExchangeError::SubscriptionFailed("Authentication failed".to_string()));
                    }
                    println!("Successfully authenticated");
                }
                Ok(Message::Close(_)) => {
                    return Err(ExchangeError::ConnectionClosed);
                }
                Err(e) => {
                    return Err(ExchangeError::WebSocketError(e));
                }
                _ => {}
            }
        }

        let subscribe_message = serde_json::json!({
            "action": "subscribe",
            "orderbooks" : [symbol_str],
        });

        let json_subscribe = serde_json::to_string(&subscribe_message)
            .map_err(|e| ExchangeError::SubscriptionFailed(format!("Failed to serialize subscribe message: {}", e)))?;

        socket.send(Message::Text(json_subscribe.into())).await?;


        self.active.store(true, Ordering::SeqCst);

        let active = Arc::clone(&self.active);
        let symbol_owned = symbol.to_string();

        tokio::spawn(async move {
            while let Some(result) = socket.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        if !text.contains("true"){
                         println!("Received: {}", text);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("Connection closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
                if !active.load(Ordering::SeqCst) {
                    eprintln!("Unsubscribing from order book for {}", symbol_owned);
                    socket.close(None).await.ok();
                    active.store(false, Ordering::SeqCst);
                    break; 
                }
            }
        });

        Ok(())

    }
    async fn unsubscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {
        match self.active.load(Ordering::SeqCst) {
            true => {
                self.active.store(false, Ordering::SeqCst);
                Ok(())
            }
            false => Err(ExchangeError::ConnectionClosed),
        }
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