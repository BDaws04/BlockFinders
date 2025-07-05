use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub volume: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OBOrder {
    pub exchange: String,
    pub side: OrderSide,
    pub volume: u64,
    pub price: u64,
}

impl OBOrder {
    pub fn new(exchange: String, side: OrderSide, volume: u64, price: u64) -> Self {
        OBOrder {
            exchange,
            side,
            volume,
            price,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub volume: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub total_volume: f64,
    pub side: OrderSide,
    pub alpaca_volume: f64,
    pub kraken_volume: f64,
    pub bybit_volume: f64,
    pub vwap: f64,
}