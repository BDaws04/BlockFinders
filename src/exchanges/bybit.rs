use crate::exchanges::exchange::Exchange;
use crate::errors::{ExchangeError, OrderPlaceError};
use crate::types::{Order, OrderSide};
use crate::config::{
    TICKER,
};
use reqwest::Client;
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::connect_async;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering}
};
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex::encode;
use crate::config::ORDER_BOOK_DEPTH;

type HmacSha256 = Hmac<Sha256>;


pub struct BybitExchange {
    api_key: String,
    api_secret: String,
    order_url: String,
    websocket_url: String,
    client: Client,
    active: Arc<AtomicBool>,
    fees: f64, 
}

impl BybitExchange {
    pub fn new(api_key: String, api_secret: String) -> Self {
        BybitExchange {
            api_key,
            api_secret,
            order_url: "https://api.bybit.com/v5/order/create".to_string(),
            websocket_url: "wss://stream.bybit.com/v5/public/spot".to_string(),
            client: Client::new(),
            active: Arc::new(AtomicBool::new(false)),
            fees: 0.0,
        }
    }
}

#[async_trait::async_trait]
impl Exchange for BybitExchange {
    async fn subscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {
        if symbol != TICKER {
            return Err(ExchangeError::InvalidSymbol(symbol.to_string()));
        }

        let pair = format!("{}USDT", symbol);
        let order_book_arg = format!("orderbook.{}.{}",ORDER_BOOK_DEPTH, pair);

        let subscribe_message = json!({
            "op": "subscribe",
            "args": [
                order_book_arg,
            ]
        });

        let subscribe_message_json = serde_json::to_string(&subscribe_message)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))?;

        let (mut socket, _) = connect_async(&self.websocket_url).await
            .map_err(|e| ExchangeError::WebSocketError(e))?;

        socket.send(Message::Text(subscribe_message_json.into())).await
            .map_err(|e| ExchangeError::WebSocketError(e.into()))?;

        self.active.store(true, Ordering::SeqCst);

        while let Some(result) = socket.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                Ok(Message::Close(_)) => {
                    println!("Connection closed");
                    break;
                }
                Err(e) => {
                    return Err(ExchangeError::WebSocketError(e));
                }
                _ => {}
            }
            if !self.active.load(Ordering::SeqCst) {
                eprintln!("Unsubscribing from order book for {}", symbol);
                socket.close(None).await.ok();
                self.active.store(false, Ordering::SeqCst);
                break; 
            }
        }

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
    async fn place_order(&self, order: Order) -> Result<(), OrderPlaceError>{
        if order.symbol != TICKER {
            return  Err(OrderPlaceError::Other(
                "Invalid symbol for Bybit exchange".to_string(),
            ));
        }
        let pair = format!("{}USDT", order.symbol);
        let recv_window = 5000;

        let body = json!({
            "category": "spot",
            "symbol": pair,
            "side": match order.side {
                OrderSide::Buy => "Buy",
                OrderSide::Sell => "Sell",
            },
            "orderType": "Market",
            "qty": order.volume.to_string(),
            "timeInForce": "GTC",
        });

        let body_str = body.to_string();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        let sign_payload = format!("{}{}{}", timestamp, recv_window, body_str);
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .map_err(|e| OrderPlaceError::HmacError(format!("Failed to create HMAC SHA256 instance: {}", e)))?;
        mac.update(sign_payload.as_bytes());

        let signature = encode(mac.finalize().into_bytes());

        let res = self.client
            .post(&self.order_url)
            .header("X-BYBIT-APIKEY", &self.api_key)
            .header("X-BYBIT-SIGNATURE", signature)
            .header("X-BYBIT-TIMESTAMP", timestamp.to_string())
            .header("X-BYBIT-RECV-WINDOW", recv_window.to_string())
            .json(&body)
            .send()
            .await
            .map_err(|e| OrderPlaceError::NetworkError(e))?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(OrderPlaceError::Other(
                format!("Failed to place order, response code: {}", res.status())
            ))
        }
    }
}