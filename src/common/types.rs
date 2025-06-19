#! [allow(unused_imports)]
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum Symbol{
    BTC,
    ETH,
    SOL
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: Symbol,
    pub quantity: f64,
    pub side: OrderSide
}

#[derive(Debug, Clone)]
pub struct OrderBook {

}

#[derive(Debug, Clone)]
pub struct OHLCVEntry {
    pub timestamp: i64, 
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct OHLCV {
    pub symbol: Symbol,                
    pub entries: Vec<OHLCVEntry>,    
}