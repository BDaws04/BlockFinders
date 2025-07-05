use crate::exchanges::exchange::Exchange;
use crate::errors::{ExchangeError, OrderPlaceError};
use crate::types::{Order, OrderSide, OBOrder};
use crate::config::{
    ORDER_BOOK_DEPTH,
    TICKER,
};
use serde::Serialize;
use reqwest::Client;
use tokio::sync::mpsc::UnboundedSender;
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

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WsMessage {
    data: Vec<BookData>,
}

#[derive(Debug, Deserialize)]
struct BookData {
    bids: Vec<Level>,
    asks: Vec<Level>,
}

#[derive(Debug, Deserialize)]
struct Level {
    price: f64,
    qty: f64,
}


type HmacSha512 = Hmac<Sha512>;

pub struct KrakenExchange {
    name: String,
    api_key: String,
    api_secret: String,
    order_url: String,
    websocket_url: String,
    client: Client,
    active: Arc<AtomicBool>,
    sender: UnboundedSender<OBOrder>,
    fees: f64, 
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
    pub fn new(api_key: String, api_secret: String, sender: UnboundedSender<OBOrder>) -> Self {
        KrakenExchange {
            name: "Kraken".to_string(),
            api_key,
            api_secret,
            order_url: "https://api.kraken.com/0/private/AddOrder".to_string(),
            websocket_url: "wss://ws.kraken.com/v2".to_string(),
            client: Client::new(),
            active: Arc::new(AtomicBool::new(false)),
            sender,
            fees: 0.0026, 
        }
    }
    pub fn get_nonce() -> String {
        let now: SystemTime = std::time::SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        (since_epoch.as_millis()).to_string()
    }
    pub fn sign_request(path: &str, nonce: &str, post_data: &str, api_secret: &str) -> Result<String, OrderPlaceError> {
        let mut sha256 = Sha256::new();
        sha256.update(nonce.as_bytes());
        sha256.update(post_data.as_bytes());
        let hash = sha256.finalize();

        let mut data = Vec::new();
        data.extend_from_slice(path.as_bytes());
        data.extend_from_slice(&hash);

        let decoded_secret = general_purpose::STANDARD
            .decode(api_secret)
            .map_err(OrderPlaceError::Base64Decode)?;

        let mut mac = HmacSha512::new_from_slice(&decoded_secret)
            .map_err(|e| OrderPlaceError::HmacError(format!("Failed to create HMAC SHA512 instance: {}", e)))?;

        mac.update(&data);
        let signature = mac.finalize().into_bytes();
        Ok(general_purpose::STANDARD.encode(signature))
    }
}

    #[async_trait::async_trait]
    impl Exchange for KrakenExchange {

    async fn subscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError> {

        if symbol != TICKER {
            return Err(ExchangeError::InvalidSymbol(symbol.to_string()));
        }

        let symbol_str = format!("{}/USD", symbol);

        let subscribe_message = serde_json::json!({
            "method": "subscribe",
            "params": {
                "channel": "book",
                "symbol": [symbol_str],
                "depth": ORDER_BOOK_DEPTH,
                "snapshot": false,
            }
        });

        let json_message = serde_json::to_string(&subscribe_message)
            .map_err(|e| ExchangeError::SubscriptionFailed(format!("Failed to serialize subscribe message: {}", e)))?;

        let (mut socket, _) = connect_async(&self.websocket_url).await?;
        socket.send(Message::Text(json_message.into())).await?;

        self.active.store(true, Ordering::SeqCst);

        let active = Arc::clone(&self.active);
        let symbol_owned = symbol.to_string();
        let exchange_name = self.name.clone();
        let sender = self.sender.clone();

        tokio::spawn(async move {
            while let Some(result) = socket.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            for book_data in ws_msg.data {
                                for bid in book_data.bids {
                                    if bid.qty > 0.0 {
                                        let ob_order = OBOrder {
                                            exchange: exchange_name.clone(),
                                            side: OrderSide::Buy,
                                            price: (bid.price * 100.0) as u64,
                                            volume: (bid.qty * 1_000_000.0) as u64,
                                        };
                                        if let Err(e) = sender.send(ob_order) {
                                            eprintln!("Failed to send OBOrder: {}", e);
                                        }
                                    }
                                }
                                for ask in book_data.asks {
                                    if ask.qty > 0.0 {
                                        let ob_order = OBOrder {
                                            exchange: exchange_name.clone(),
                                            side: OrderSide::Sell,
                                            price: (ask.price * 100.0) as u64,
                                            volume: (ask.qty * 1_000_000.0) as u64,
                                        };
                                        if let Err(e) = sender.send(ob_order) {
                                            eprintln!("Failed to send OBOrder: {}", e);
                                        }
                                    }
                                }
                            }
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


    async fn unsubscribe_ob(&self, _symbol: &str) -> Result<(), ExchangeError> {
        match self.active.load(Ordering::SeqCst) {
            true => {
                self.active.store(false, Ordering::SeqCst);
                Ok(())
            }
            false => Err(ExchangeError::ConnectionClosed),
        }
    }
    async fn place_order(&self, _order: Order) -> Result<(), OrderPlaceError> {
        let nonce = KrakenExchange::get_nonce();
        let mut params = HashMap::new();

        params.insert("nonce", nonce.as_str());
        params.insert("ordertype", "market");
        params.insert("type", match _order.side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        });
        let volume_str = _order.volume.to_string();
        params.insert("volume", &volume_str);
        let pair = format!("{}/USD", _order.symbol);
        params.insert("pair", &pair);

        let post_data = serde_urlencoded::to_string(&params)
            .map_err(OrderPlaceError::Serialization)?;

        let signature = KrakenExchange::sign_request(self.order_url.as_str(), &nonce, &post_data, &self.api_secret)?;

        let res = self.client.post(&self.order_url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .body(post_data)
            .send()
            .await
            .map_err(OrderPlaceError::Http)?;

        if res.status().is_success() {
            Ok(())
        } else {
            let error_message = res.text().await.map_err(OrderPlaceError::Http)?;
            Err(OrderPlaceError::Other(format!("Failed to place order: {}", error_message)))
        }
    }
}