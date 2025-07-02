use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub volume: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OrderSide {
    Buy,
    Sell,
}