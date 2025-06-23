#! [allow(unused_imports)]
#![allow(dead_code)]

use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub enum Symbol{
    BTC,
    ETH,
    SOL
}
impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::BTC => write!(f, "BTC"),
            Symbol::ETH => write!(f, "ETH"),
            Symbol::SOL => write!(f, "SOL"),
        }
    }
}
impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        match s {
            "BTC" => Symbol::BTC,
            "ETH" => Symbol::ETH,
            "SOL" => Symbol::SOL,
            _ => panic!("Unknown symbol: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: Symbol,
    pub quantity: f64,
    pub side: OrderSide
}

#[derive(Debug, Clone, Deserialize)]
pub struct OHLCVResponse {
    pub symbol: Symbol,
    pub bar: Bar,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bar {
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub time: String,
    pub volume: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookResponse {
    pub symbol: Symbol,
    pub book: OrderBook,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBook {
    pub asks: Vec<PriceLevel>,  
    pub bids: Vec<PriceLevel>, 
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceLevel {
    pub price: f64,    
    pub quantity: f64, 
}


#[derive(Debug, Clone)]
pub enum WsMessage {
    SubscribeOrderBook(Symbol),
    UnsubscribeOrderBook(Symbol),
    SubscribeTicker(Symbol),
    UnsubscribeTicker(Symbol),
}

#[derive(Debug, Clone)]
pub struct MarketSnapshotResponse {
    pub symbol: Symbol,
    pub ask_price: f64,
    pub ask_quantity: f64,
    pub bid_price: f64,
    pub bid_quantity: f64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExchangeStatus{
    ONLINE,
    OFFLINE,
    MAINTENANCE,
}
#[derive(Debug, Clone)]
pub struct ExchangeStatusResponse {
    pub status: ExchangeStatus,
}

#[derive(Debug, Clone)]
pub struct OrderFeesResponse {
    pub maker_fee: f64,
    pub taker_fee: f64,
}