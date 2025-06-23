use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use anyhow::Result;
use reqwest::Client;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse, MarketSnapshotResponse, OrderBookResponse, OHLCVResponse};
use crate::common::types::{OrderBook, PriceLevel};
use crate::common::types::Symbol;

use serde::Deserialize;
use  std::collections::HashMap;

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

        #[derive(Debug, Clone, Deserialize)]
        struct RawOrderBookLevel(String, String, u64); 

        #[derive(Debug, Clone, Deserialize)]
        struct RawOrderBook {
            asks: Vec<RawOrderBookLevel>,
            bids: Vec<RawOrderBookLevel>,
        }

        #[derive(Debug, Deserialize)]
        struct KrakenResponse {
            error: Vec<String>,
            result: HashMap<String, RawOrderBook>,
        }

        let pair = symbol.to_string() + "/USD";
        let url = format!("https://api.kraken.com/0/public/Depth?pair={}", pair);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let resp: KrakenResponse = serde_json::from_str(&text)?;

            if !resp.error.is_empty() {
                anyhow::bail!("Kraken API error: {:?}", resp.error);
            }

            if let Some(raw_book) = resp.result.get(&pair) {
                let asks = raw_book.asks.iter().map(|level| {
                    PriceLevel {
                        price: level.0.parse().unwrap_or(0.0),
                        quantity: level.1.parse().unwrap_or(0.0),
                    }
                }).collect();

                let bids = raw_book.bids.iter().map(|level| {
                    PriceLevel {
                        price: level.0.parse().unwrap_or(0.0),
                        quantity: level.1.parse().unwrap_or(0.0),
                    }
                }).collect();

                let book = OrderBook { asks, bids };

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
        let pair: String = symbol.to_string() + "/USD";
        let url = format!("{}/0/public/Ticker?pair={}", self.base_url, pair);
        let response = self.client.get(&url)
            .send()
            .await?;
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let result = &json["result"][&pair];
            let ohlcv = OHLCVResponse {
                symbol,
                bar: crate::common::types::Bar {
                    close: result["c"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                    high: result["h"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                    low: result["l"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                    open: result["o"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                    time: result["t"].as_str().unwrap_or("unknown").to_string(),
                    volume: result["v"][1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                },
            };
            Ok(ohlcv)
        } else {
            Ok(OHLCVResponse {
                symbol,
                bar: crate::common::types::Bar {
                    close: 0.0,
                    high: 0.0,
                    low: 0.0,
                    open: 0.0,
                    time: "unknown".to_string(),
                    volume: 0.0,
                },
            })
        }
    }

    async fn get_market_snapshot(&self, symbol: Symbol) -> Result<MarketSnapshotResponse> {
        let pair: String = symbol.to_string() + "/USD";
        let url = format!("{}/0/public/Ticker?pair={}", self.base_url, pair);
        let response = self.client.get(&url)
            .send()
            .await?;
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let result = &json["result"][&pair];
            let market_snapshot = MarketSnapshotResponse {
                symbol,
                ask_price:result["a"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                ask_quantity: result["a"][1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                bid_price: result["b"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                bid_quantity: result["b"][1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
            };
            Ok(market_snapshot)
        } else {
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
        let url = format!("{}/0/public/SystemStatus", self.base_url);
        let response = self.client.get(&url)
            .send()
            .await?;
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let status = json["result"]["status"].as_str().unwrap_or("unknown").to_string();

            if (status == "online"){
                Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::ONLINE })
            }
            else if (status == "maintenance") {
                Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::MAINTENANCE })
            }
            else {
                Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::OFFLINE })
            }
        } else {
            Ok(ExchangeStatusResponse { status: crate::common::types::ExchangeStatus::OFFLINE })
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::GET;
    use tokio;
    use crate::common::types::{ExchangeStatus, Symbol};

    // Helper to create a KrakenIngestor with a mocked base_url
    fn make_test_ingestor(api_key: &str, base_url: &str) -> KrakenIngestor {
        KrakenIngestor {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    #[tokio::test]
    async fn test_get_order_book_success() {
        let server = MockServer::start();
        let symbol = Symbol::from("BTC");
        let pair = format!("{}/USD", symbol);

        let mock_response = serde_json::json!({
            "error": [],
            "result": {
                pair.clone(): {
                    "asks": [["30000.1", "1.5", "123"]],
                    "bids": [["29999.9", "2.5", "456"]]
                }
            }
        });
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/0/public/Depth")
                .query_param("pair", &pair);
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("api_key", &server.base_url());
        let result = ingestor.get_order_book(symbol.clone()).await.unwrap();

        println!("Order book response: {:?}", result.book.asks);

        assert_eq!(result.symbol, symbol);
        assert_eq!(result.book.asks.len(), 100);
    }

    #[tokio::test]
    async fn test_get_ohlvc_success() {
        let server = MockServer::start();
        let symbol = Symbol::from("ETH");
        let pair = format!("{}/USD", symbol);

        let mock_response = serde_json::json!({
            "error": [],
            "result": {
                pair.clone(): {
                    "c": ["2100.5", "5"],
                    "h": ["2150.3", "10"],
                    "l": ["2080.2", "8"],
                    "o": "2095.0",
                    "t": "1622548800", 
                    "v": ["100", "300.5"]
                }
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/0/public/Ticker")
                .query_param("pair", &pair);
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("api_key", &server.base_url());
        let result = ingestor.get_ohlvc(symbol.clone()).await.unwrap();

        assert_eq!(result.symbol, symbol);
        assert!((result.bar.close - 2100.5).abs() < 1e-6);
        assert!((result.bar.high - 2150.3).abs() < 1e-6);
        assert!((result.bar.low - 2080.2).abs() < 1e-6);
        assert!((result.bar.open - 2095.0).abs() < 1e-6);
        assert_eq!(result.bar.time, "1622548800");
        assert!((result.bar.volume - 300.5).abs() < 1e-6);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_market_snapshot_success() {
        let server = MockServer::start();
        let symbol = Symbol::from("SOL");
        let pair = format!("{}/USD", symbol);

        let mock_response = serde_json::json!({
            "error": [],
            "result": {
                pair.clone(): {
                    "a": ["45.0", "1.5"],
                    "b": ["44.5", "2.0"]
                }
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/0/public/Ticker")
                .query_param("pair", &pair);
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("api_key", &server.base_url());
        let result = ingestor.get_market_snapshot(symbol.clone()).await.unwrap();

        assert_eq!(result.symbol, symbol);
        assert!((result.ask_price - 45.0).abs() < 1e-6);
        assert!((result.ask_quantity - 1.5).abs() < 1e-6);
        assert!((result.bid_price - 44.5).abs() < 1e-6);
        assert!((result.bid_quantity - 2.0).abs() < 1e-6);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_exchange_status_online() {
        let server = MockServer::start();

        let mock_response = serde_json::json!({
            "error": [],
            "result": {
                "status": "online"
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/0/public/SystemStatus");
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("api_key", &server.base_url());
        let result = ingestor.get_exchange_status().await.unwrap();

        assert_eq!(result.status, ExchangeStatus::ONLINE);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_exchange_status_maintenance() {
        let server = MockServer::start();

        let mock_response = serde_json::json!({
            "error": [],
            "result": {
                "status": "maintenance"
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/0/public/SystemStatus");
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("api_key", &server.base_url());
        let result = ingestor.get_exchange_status().await.unwrap();

        assert_eq!(result.status, ExchangeStatus::MAINTENANCE);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_exchange_status_offline_for_other_status() {
        let server = MockServer::start();

        let mock_response = serde_json::json!({
            "error": [],
            "result": {
                "status": "something_else"
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/0/public/SystemStatus");
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("api_key", &server.base_url());
        let result = ingestor.get_exchange_status().await.unwrap();

        assert_eq!(result.status, ExchangeStatus::OFFLINE);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_fees_tiered() {
        let ingestor = make_test_ingestor("api_key", "http://localhost");
        let tiers = vec![
            (5_000.0, (0.0025, 0.004)),
            (20_000.0, (0.0020, 0.0035)),
            (70_000.0, (0.0014, 0.0024)),
            (200_000.0, (0.0012, 0.0022)),
            (400_000.0, (0.0010, 0.0020)),
            (800_000.0, (0.0008, 0.0018)),
            (2_000_000.0, (0.0006, 0.0016)),
            (4_000_000.0, (0.0004, 0.0014)),
            (8_000_000.0, (0.0002, 0.0012)),
            (20_000_000.0, (0.0, 0.0010)),
        ];

        for (amount, (expected_maker, expected_taker)) in tiers {
            let res = ingestor.get_fees(amount).await.unwrap();
            assert!((res.maker_fee - expected_maker).abs() < 1e-6);
            assert!((res.taker_fee - expected_taker).abs() < 1e-6);
        }
    }
}
