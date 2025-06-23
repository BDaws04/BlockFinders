use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use anyhow::Result;
use reqwest::Client;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse, MarketSnapshotResponse, OrderBookResponse, OHLCVResponse};
use crate::common::types::Symbol;

pub struct KrakenIngestor{
    api_key: String,
    client: Client,
    base_url: String,
}

impl KrakenIngestor {
    pub fn new(api_key: String, base_url: String) -> Self {
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
    async fn get_order_book(&self, symbol: Symbol) -> Result<OrderBookResponse> {
        // Implementation for fetching order book from Kraken
        unimplemented!()
    }

    async fn get_ohlvc(&self, symbol: Symbol) -> Result<OHLCVResponse> {
        // Implementation for fetching OHLCV data from Kraken
        unimplemented!()
    }

    async fn get_market_snapshot(&self, symbol: Symbol) -> Result<MarketSnapshotResponse> {
        unimplemented!()
    }   

    async fn get_exchange_status(&self) -> Result<ExchangeStatusResponse> {
        unimplemented!()
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