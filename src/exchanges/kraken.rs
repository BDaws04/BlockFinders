use crate::exchanges::exchange::Exchange;
use crate::errors::ExchangeError;
use crate::types::{Order, OrderSide};
use crate::config::{
    ORDER_BOOK_DEPTH,
    TICKER,
};
use serde::Serialize;
use tokio::task;
use tokio_tungstenite::tungstenite;
use futures_util::SinkExt;
use futures_util::StreamExt;

pub struct KrakenExchange {
    api_key: String,
    api_secret: String,
    order_url: String,
    websocket_url: String,
}
#[derive(Serialize)]
pub struct OrderBookSubscribe {
    method: String,
    params: OrderBookSubscribeParams,
}
#[derive(Serialize)]
pub struct OrderBookSubscribeParams {
    channel: String,
    symbol: Vec<String>,
    depth: usize,
    snapshot: bool,
}

impl KrakenExchange {
    pub fn new(api_key: String, api_secret: String) -> Self {
        KrakenExchange {
            api_key,
            api_secret,
            order_url: "https://api.kraken.com/0/private/AddOrder".to_string(),
            websocket_url: "wss://ws.kraken.com/v2".to_string(),
        }
    }
 }

#[async_trait::async_trait]
impl Exchange for KrakenExchange {

    async fn subscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {
        if symbol != TICKER {
            return Err(ExchangeError::InvalidSymbol(symbol.to_string()));
        }

        let symbol = format!("{}/USD", symbol);

        let subscribe_message = OrderBookSubscribe {
            method: "subscribe".to_string(),
            params: OrderBookSubscribeParams {
                channel: "book".to_string(),
                symbol: vec![symbol.to_string()],
                depth: ORDER_BOOK_DEPTH,
                snapshot: true,
            },
        };

        let json_message = match serde_json::to_string(&subscribe_message) {
            Ok(json) => json,
            Err(_) => return Err(ExchangeError::SubscriptionFailed("Failed to serialize subscribe message".to_string())),
        };

        let url = &self.websocket_url;

        let mut socket = match tokio_tungstenite::connect_async(url).await {
            Ok((socket, _)) => socket,
            Err(e) => return Err(ExchangeError::WebSocketError(e)),
        };

        let message = tungstenite::Message::Text(json_message.into());

        socket.send(message).await.map_err(ExchangeError::WebSocketError)?;

        task::spawn(async move {
            while let Some(message) = socket.next().await {
                match message {
                    Ok(tungstenite::Message::Text(text)) => {
                        println!("Received: {}", text);
                    }
                    Ok(tungstenite::Message::Close(_)) => {
                        println!("Connection closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    async fn unsubscribe_ob(&self, _symbol: &str) -> Result<(), ExchangeError> {
        unimplemented!()
    }
    async fn place_order(&self, _order: Order) -> Result<(), ExchangeError> {
        unimplemented!()
    }
    async fn get_fees(&self) -> Result<f64, ExchangeError> {
        unimplemented!()
    }

}