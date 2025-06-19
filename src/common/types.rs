#! [allow(unused_imports)]
#![allow(dead_code)]

pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: &str,
    pub quantity: f64,
    pub side: OrderSide
}