#! [allow(unused_imports)]
#! [allow(dead_code)]


use async_trait::async_trait;
use anyhow::Result;
use crate::common::types::{OrderBookResponse, OrderFeesResponse, Symbol, ExchangeStatusResponse, MarketSnapshotResponse, OHLCVResponse};


#[async_trait]
pub trait MarketDataIngestor: Send + Sync {
    // REST data fetching methods
    async fn get_order_book(&self, symbol: Symbol) -> Result<OrderBookResponse>;
    
    async fn get_ohlvc(&self, symbol: Symbol) -> Result<OHLCVResponse>;

    async fn get_market_snapshot(&self, symbol: Symbol) -> Result<MarketSnapshotResponse>;

    // WebSocket methods
    async fn subscribe_order_book(&self, symbol: Symbol) -> Result<()>;
    async fn unsubscribe_order_book(&self, symbol: Symbol) -> Result<()>;

    async fn subscribe_ohlcv(&self, symbol: Symbol, interval: &str) -> Result<()>;
    async fn unsubscribe_ohlcv(&self, symbol: Symbol, interval: &str) -> Result<()>;

    async fn subscribe_ticker(&self, symbol: &str) -> Result<()>;
    async fn unsubscribe_ticker(&self, symbol: &str) -> Result<()>;

    // General methods
    async fn get_exchange_status(&self) -> Result<ExchangeStatusResponse>;
    async fn get_fees(&self, usd_amount: f64) -> Result<OrderFeesResponse>; 
}
