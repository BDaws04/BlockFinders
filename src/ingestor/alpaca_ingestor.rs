use std::collections::HashMap;

use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use anyhow::Result;
use reqwest::Client;

use serde::Deserialize;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse, MarketSnapshotResponse, OrderBookResponse, OHLCVResponse};
use crate::common::types::{OrderBook, PriceLevel, Symbol, Bar, Exchanges};


pub struct AlpacaIngestor {
    api_key: String,
    secret_key: String,
    client: Client,
    base_url: String,
}

impl AlpacaIngestor {
    pub fn new(api_key: String, secret_key: String, base_url: String) -> Self {
        AlpacaIngestor {
            api_key,
            secret_key,
            client: Client::new(),
            base_url,
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

        #[derive(Debug, Clone, Deserialize)]
        struct OrderBooksWrapper {
            orderbooks: HashMap<String, RawOrderBook>,
        }

        let url = format!("{}/v1beta3/crypto/us/latest/orderbooks?symbols={}%2FUSD", self.base_url, symbol);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;

            let wrapper: OrderBooksWrapper = serde_json::from_str(&text)?;
            let key = format!("{}/USD", symbol.to_string().to_uppercase());

            if let Some(raw_book) = wrapper.orderbooks.get(&key) {
                let book = OrderBook {
                    asks: raw_book.a.iter().map(|pl| PriceLevel {
                        price: pl.p,
                        quantity: pl.s,
                        exchange: Exchanges::Alpaca,
                    }).collect(),
                    bids: raw_book.b.iter().map(|pl| PriceLevel {
                        price: pl.p,
                        quantity: pl.s,
                        exchange: Exchanges::Alpaca,
                    }).collect(),
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

        let url = format!("{}/v1beta3/crypto/us/latest/ohlcv?symbol={}/USD", self.base_url, symbol);
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
        let url = format!("{}/v1beta3/crypto/us/latest/quotes?symbol={}/USD", self.base_url, symbol);
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::GET;
    use tokio;

    fn make_test_ingestor(base_url: &str) -> AlpacaIngestor {
        AlpacaIngestor::new(
            "fake_api_key".to_string(),
            "fake_secret".to_string(),
            base_url.to_string()
        )
    }

    #[tokio::test]
    async fn test_get_order_book_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1beta3/crypto/us/latest/orderbooks")
                .query_param("symbol", "BTC/USD");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
                    "BTC/USD": {
                        "a": [{"p": 30000.0, "s": 1.0}],
                        "b": [{"p": 29950.0, "s": 2.0}]
                    }
                }"#);
        });

        let ingestor = make_test_ingestor(&server.base_url());
        let result = ingestor.get_order_book(Symbol::from("BTC")).await.unwrap();

        assert_eq!(result.book.asks[0].price, 30000.0);
        assert_eq!(result.book.bids[0].quantity, 2.0);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_ohlcv_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1beta3/crypto/us/latest/ohlcv")
                .query_param("symbol", "ETH/USD");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
                    "bars": {
                        "ETH/USD": {
                            "c": 2100.0,
                            "h": 2150.0,
                            "l": 2080.0,
                            "n": 1000,
                            "o": 2095.0,
                            "t": "2025-06-21T00:00:00Z",
                            "v": 300.0,
                            "vw": 2105.0
                        }
                    }
                }"#);
        });

        let ingestor = make_test_ingestor(&server.base_url());
        let result = ingestor.get_ohlvc(Symbol::from("ETH")).await.unwrap();;

        assert_eq!(result.bar.close, 2100.0);
        assert_eq!(result.bar.volume, 300.0);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_market_snapshot_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1beta3/crypto/us/latest/quotes")
                .query_param("symbol", "SOL/USD");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{ "ap": 45.0, "as": 1.5, "bp": 44.5, "bs": 2.0 }"#);
        });

        let ingestor = make_test_ingestor(&server.base_url());
        let result = ingestor.get_market_snapshot(Symbol::from("SOL")).await.unwrap();

        assert_eq!(result.ask_price, 45.0);
        assert_eq!(result.bid_quantity, 2.0);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_exchange_status_online() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/clock");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{ "is_open": true }"#);
        });

        let ingestor = make_test_ingestor(&server.base_url());
        let result = ingestor.get_exchange_status().await.unwrap();

        use crate::common::types::ExchangeStatus;
        assert_eq!(result.status, ExchangeStatus::ONLINE);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_fees_tiered() {
        let ingestor = make_test_ingestor("http://localhost");
        let tiers = vec![
            (50_000.0, 0.0015),
            (200_000.0, 0.0012),
            (800_000.0, 0.0010),
            (5_000_000.0, 0.0008),
            (20_000_000.0, 0.0005),
            (40_000_000.0, 0.0002),
            (90_000_000.0, 0.0002),
            (150_000_000.0, 0.0),
        ];
        for (amount, expected_fee) in tiers {
            let res = ingestor.get_fees(amount).await.unwrap();
            assert_eq!(res.maker_fee, expected_fee);
        }
    }
}

