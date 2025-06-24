#! [allow(unused_imports)]
#![allow(dead_code)]

use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct MaxPriceLevel(pub PriceLevel);

impl Eq for MaxPriceLevel {}

impl PartialEq for MaxPriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.0.price == other.0.price
    }
}

impl Ord for MaxPriceLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.price.partial_cmp(&other.0.price).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for MaxPriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct MinPriceLevel(pub PriceLevel);

impl Eq for MinPriceLevel {}

impl PartialEq for MinPriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.0.price == other.0.price
    }
}

impl Ord for MinPriceLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.price.partial_cmp(&self.0.price).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for MinPriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
pub enum Exchanges {
    Bybit,
    Kraken,
    Alpaca,
}

#[derive(Debug, Clone)]
pub struct RoutedOrder {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub bybit_volume: f64,
    pub kraken_volume: f64,
    pub alpaca_volume: f64,
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

#[derive(Debug, Clone, Deserialize, Copy)]
pub struct PriceLevel {
    pub price: f64,    
    pub quantity: f64,
    pub exchange: Exchanges,
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