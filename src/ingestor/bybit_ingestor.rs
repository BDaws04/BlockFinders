use std::collections::HashMap;

use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use anyhow::Result;
use reqwest::Client;

use serde::Deserialize;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse, MarketSnapshotResponse, OrderBookResponse, OHLCVResponse};
use crate::common::types::{OrderBook, PriceLevel, Symbol, Bar};

pub struct BybitIngestor {
    client: Client,
    base_url: String,
    api_key: String,
}

impl BybitIngestor {
    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        let base_url = "https://api.bybit.com".to_string();
        BybitIngestor {
            client,
            base_url,
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl MarketDataIngestor for BybitIngestor {
    async fn get_order_book(&self, symbol: Symbol) -> Result<OrderBookResponse> {
        unimplemented!()
    }
    async fn get_ohlvc(&self, symbol: Symbol) -> Result<OHLCVResponse> {
        unimplemented!()
    }
    async fn get_market_snapshot(&self, symbol: Symbol) -> Result<MarketSnapshotResponse> {
        unimplemented!()
    }
    async fn get_exchange_status(&self) -> Result<ExchangeStatusResponse> {
        let url = format!("{}/v5/market/time", self.base_url);
        let response = self.client.get(&url)
            .send()
            .await?;
        if (response.status().is_success()) {
            Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::ONLINE })
        } else {
            Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::OFFLINE })
        }
    }
    async fn get_fees(&self, usd_amount: f64) -> Result<OrderFeesResponse> {
        Ok(OrderFeesResponse { maker_fee: (0.0010), taker_fee: (0.0010) })
    }
}