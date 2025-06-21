use std::collections::HashMap;

use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use anyhow::Result;
use reqwest::Client;

use tokio::sync::mpsc::{Sender};
use serde::Deserialize;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse, MarketSnapshotResponse, OrderBookResponse, OHLCVResponse};
use crate::common::types::{OrderBook, PriceLevel, Symbol, Bar};
use crate::common::types::WsMessage;

pub struct AlpacaIngestor {
    api_key: String,
    secret_key: String,
    client: Client,
    base_url: String,

    ws_sender: Sender<WsMessage>  
}

impl AlpacaIngestor {
    pub fn new(api_key: String, secret_key: String, base_url: String) -> Self {

        let (ws_sender, _ws_receiver) = tokio::sync::mpsc::channel(100);
        // remember to handle receiver
        /*
        tokio::spawn(async move {
            websocket_task(ws_receiver).await;
        });
         */

        AlpacaIngestor {
            api_key,
            secret_key,
            client: Client::new(),
            base_url,
            ws_sender,
        }
    }
}

#[async_trait::async_trait]
impl MarketDataIngestor for AlpacaIngestor {
    // Implement the methods defined in the MarketDataIngestor trait
    async fn get_order_book(&self, symbol: Symbol) -> Result<OrderBookResponse> {

        #[derive(Debug, Clone, Deserialize)]
        struct RawPriceLevel {
            p: f64,
            s: f64,
        }

        #[derive(Debug, Clone, Deserialize)]
        struct RawOrderBook {
            a: Vec<RawPriceLevel>,
            b: Vec<RawPriceLevel>,
        }


        let url = format!("{}/v1beta3/crypto/us/latest/orderbooks?symbol={}%2Fusd", self.base_url, symbol);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;

            let raw_map: HashMap<String, RawOrderBook> = serde_json::from_str(&text)?;

            let key = format!("{}/USD", symbol.to_string().to_uppercase());

            if let Some(raw_book) = raw_map.get(&key) {
                let book = OrderBook {
                    asks: raw_book.a.iter().map(|pl| PriceLevel { price: pl.p, quantity: pl.s }).collect(),
                    bids: raw_book.b.iter().map(|pl| PriceLevel { price: pl.p, quantity: pl.s }).collect(),
                };

                Ok(OrderBookResponse { symbol, book })
            } else {
                Ok(OrderBookResponse {
                    symbol,
                    book: OrderBook { asks: vec![], bids: vec![] },
                })
            }
        } else {
            Ok(OrderBookResponse {
                symbol,
                book: OrderBook { asks: vec![], bids: vec![] },
            })
        }
    }


    async fn get_ohlvc(&self, symbol: Symbol) -> Result<OHLCVResponse> {

        #[derive(Debug, Clone, Deserialize)]
        struct RawBar {
            c: f64,
            h: f64,
            l: f64,
            n: u32,
            o: f64,
            t: String,
            v: f64,
            vw: f64,
        }

        #[derive(Debug, Clone, Deserialize)]
        struct RawBars {
            bars: HashMap<String, RawBar>,
        }

        // Your common structs (assumed defined elsewhere):
        // pub struct OhlcvResponse { pub symbol: Symbol, pub bar: Bar }
        // pub struct Bar { pub close: f64, pub high: f64, pub low: f64, pub open: f64, pub time: String, pub volume: f64, /* etc */ }

        let url = format!("{}/v1beta3/crypto/us/latest/ohlcv?symbol={}%2Fusd", self.base_url, symbol);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;

            let raw_bars: RawBars = serde_json::from_str(&text)?;

            let key = format!("{}/USD", symbol.to_string().to_uppercase());

            if let Some(raw_bar) = raw_bars.bars.get(&key) {
                let bar = Bar {
                    close: raw_bar.c,
                    high: raw_bar.h,
                    low: raw_bar.l,
                    open: raw_bar.o,
                    time: raw_bar.t.clone(),
                    volume: raw_bar.v,
                };

                Ok(OHLCVResponse {symbol, bar})
            } else {
                Ok(OHLCVResponse {
                    symbol,
                    bar: Bar {
                        close: 0.0,
                        high: 0.0,
                        low: 0.0,
                        open: 0.0,
                        time: "".to_string(),
                        volume: 0.0,
                    },
                })
            }
        } else {
            Ok(OHLCVResponse {
                symbol,
                bar: Bar {
                    close: 0.0,
                    high: 0.0,
                    low: 0.0,
                    open: 0.0,
                    time: "".to_string(),
                    volume: 0.0,
                },
            })
        }
    }


    async fn get_market_snapshot(&self, symbol: Symbol) -> Result<MarketSnapshotResponse> {
        let url = format!("{}/v1beta3/crypto/us/latest/quotes?symbol={}%2Fusd", self.base_url, symbol);
        let response = self.client.get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let snapshot = MarketSnapshotResponse {
                symbol,
                ask_price: json["ap"].as_f64().unwrap_or(0.0),
                ask_quantity: json["as"].as_f64().unwrap_or(0.0),
                bid_price: json["bp"].as_f64().unwrap_or(0.0),
                bid_quantity: json["bs"].as_f64().unwrap_or(0.0),
            };
            Ok(snapshot)
        }
        else {
            Ok(MarketSnapshotResponse {
                symbol,
                ask_price: 0.0,
                ask_quantity: 0.0,
                bid_price: 0.0,
                bid_quantity: 0.0,
            })
        }
            
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
        let url = format!("{}/v2/clock", self.base_url);
        let response = self.client.get(&url)
            .header("APCA_API_KEY_ID", &self.api_key)
            .header("APCA_API_SECRET_KEY", &self.secret_key)
            .send()
            .await?;
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let status = if json["is_open"].as_bool().unwrap_or(false) {
                ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::ONLINE }
            } else {
                ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::OFFLINE }
            };
            Ok(status)
        } else {
            Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::OFFLINE })
        }
    }

    async fn get_fees(&self, usd_amount: f64) -> Result<OrderFeesResponse> {
        match usd_amount {
            amount if amount < 100_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0015, taker_fee: 0.0025 }),
            amount if amount < 500_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0012, taker_fee: 0.0022 }),
            amount if amount < 1_000_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0010, taker_fee: 0.0020 }),
            amount if amount < 10_000_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0008, taker_fee: 0.0018 }),
            amount if amount < 25_000_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0005, taker_fee: 0.0015 }),
            amount if amount < 50_000_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0002, taker_fee: 0.0013 }),
            amount if amount < 100_000_000.0 => Ok(OrderFeesResponse { maker_fee: 0.0002, taker_fee: 0.0012 }),
            _ => Ok(OrderFeesResponse { maker_fee: 0.0, taker_fee: 0.0010 }),
        }
    }

}