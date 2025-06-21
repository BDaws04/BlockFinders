use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use crate::common::types::{OrderBook, Symbol, OHLCV};
use anyhow::Result;
use reqwest::Client;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse};

pub struct KrakenIngestor{
    api_key: String,
    client: Client,
    base_url: String,
}

impl KrakenIngestor {
    pub fn new(api_key: String) -> Self {
        KrakenIngestor {
            api_key,
            client: Client::new(),
            base_url,
        }
    }
}

#[async_trait::async_trait]
impl MarketDataIngestor for KrakenIngestor {
    // Implement the methods defined in the MarketDataIngestor trait
    async fn get_order_book(&self, symbol: Symbol) -> Result<OrderBook> {
        // Implementation for fetching order book from Kraken
        unimplemented!()
    }

    async fn get_ohlvc(&self, symbol: Symbol, interval: &str) -> Result<OHLCV> {
        // Implementation for fetching OHLCV data from Kraken
        unimplemented!()
    }

    async fn get_buy_price(&self, symbol: Symbol) -> Result<f64> {
        // Implementation for fetching buy price from Kraken
        unimplemented!()
    }

    async fn get_sell_price(&self, symbol: Symbol) -> Result<f64> {
        // Implementation for fetching sell price from Kraken
        unimplemented!()
    }

    async fn subscribe_order_book(&self, symbol: Symbol) -> Result<()> {
        // Implementation for subscribing to order book updates
        unimplemented!()
    }

    async fn unsubscribe_order_book(&self, symbol: Symbol) -> Result<()> {
        // Implementation for unsubscribing from order book updates
        unimplemented!()
    }

    async fn subscribe_ohlcv(&self, symbol: Symbol, interval: &str) -> Result<()> {
        // Implementation for subscribing to OHLCV updates
        unimplemented!()
    }

    async fn unsubscribe_ohlcv(&self, symbol: Symbol, interval: &str) -> Result<()> {
        // Implementation for unsubscribing from OHLCV updates
        unimplemented!()
    }

    async fn subscribe_ticker(&self, symbol: &str) -> Result<()> {
        // Implementation for subscribing to ticker updates
        unimplemented!()
    }

    async fn unsubscribe_ticker(&self, symbol: &str) -> Result<()> {
        // Implementation for unsubscribing from ticker updates
        unimplemented!()
    }

    async fn get_exchange_status(&self) -> Result<ExchangeStatusResponse> {
        
    }

    async fn get_fees(&self, usd_amount: f64) -> Result<OrderFeesResponse> { 
        match usd_amount {
            amount if amount < 10000.0 => Ok(OrderFeesResponse { maker_fee: 0.0025, taker_fee: 0.004 }),
            amount if amount < 50000.0 => Ok(OrderFeesResponse { maker_fee: 0.0020, taker_fee: 0.0035 }),
            amount if amount < 100000.0 => Ok(OrderFeesResponse { maker_fee: 0.0014, taker_fee: 0.0024 }),
            amount if amount < 250000.0 => Ok(OrderFeesResponse { maker_fee: 0.0012, taker_fee: 0.0022 }),
            amount if amount < 500000.0 => Ok(OrderFeesResponse { maker_fee: 0.0010, taker_fee: 0.0020 }),
            amount if amount < 1000000.0 => Ok(OrderFeesResponse { maker_fee: 0.0008, taker_fee: 0.0018 }),
            amount if amount < 2500000.0 => Ok(OrderFeesResponse { maker_fee: 0.0006, taker_fee: 0.0016 }),
            amount if amount < 5000000.0 => Ok(OrderFeesResponse { maker_fee: 0.0004, taker_fee: 0.0014 }),
            amount if amount < 10000000.0 => Ok(OrderFeesResponse { maker_fee: 0.0002, taker_fee: 0.0012 }),
            _ => Ok(OrderFeesResponse { maker_fee: 0.0, taker_fee: 0.0010 }),
        }
    }
}