use crate::ingestor::market_data_ingestor::MarketDataIngestor;

use anyhow::Result;
use reqwest::Client;

use serde::Deserialize;

use crate::common::types::{OrderFeesResponse, ExchangeStatusResponse, MarketSnapshotResponse, OrderBookResponse, OHLCVResponse};
use crate::common::types::{OrderBook, PriceLevel, Symbol, Exchanges};

pub struct BybitIngestor {
    client: Client,
    base_url: String,
    api_key: String,
    secret_key: String,
}

impl BybitIngestor {
    pub fn new(api_key: String, secret_key: String, base_url: String) -> Self {
        let client = Client::new();
        BybitIngestor {
            client,
            base_url,
            api_key,
            secret_key,
        }
    }
}

#[async_trait::async_trait]
impl MarketDataIngestor for BybitIngestor {
    async fn get_order_book(&self, symbol: Symbol) -> Result<OrderBookResponse> {

        #[derive(Debug, Clone, Deserialize)]
        struct RawPriceLevel(String, String); 

        #[derive(Debug, Clone, Deserialize)]
        struct RawResult {
            s: String,
            a: Vec<RawPriceLevel>,
            b: Vec<RawPriceLevel>,
        }

        #[derive(Debug, Clone, Deserialize)]
        struct ApiResponse {
            retCode: i32,
            retMsg: String,
            result: RawResult,
        }

        let pair = symbol.to_string() + "USDT";
        let url = format!("{}/v5/market/orderbook?category=spot&symbol={}&limit=100", self.base_url, pair);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let api_resp: ApiResponse = serde_json::from_str(&text)?;

            if api_resp.retCode == 0 {
                let raw_book = api_resp.result;

                let asks = raw_book.a.iter().filter_map(|pl| {
                    Some(PriceLevel {
                        price: pl.0.parse().ok()?,
                        quantity: pl.1.parse().ok()?,
                        exchange: Exchanges::Bybit
                    })
                }).collect();

                let bids = raw_book.b.iter().filter_map(|pl| {
                    Some(PriceLevel {
                        price: pl.0.parse().ok()?,
                        quantity: pl.1.parse().ok()?,
                        exchange: Exchanges::Bybit
                    })
                }).collect();

                let book = OrderBook { asks, bids };

                Ok(OrderBookResponse {
                    symbol: symbol.clone(),
                    book,
                })
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::GET;
    use tokio;

    // Helper to create a BybitIngestor with a mocked base_url
    fn make_test_ingestor(api_key: &str, secret_key: &str, base_url: &str) -> BybitIngestor {
        BybitIngestor {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
        }
    }

    #[tokio::test]
    async fn test_get_order_book_success() {
        let server = MockServer::start();
        let symbol = Symbol::from("BTC");
        let pair = format!("{}{}", symbol.to_string(), "/USDT");

        let mock_response = serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "s": pair,
                "a": [["30000.1", "1.5"]],
                "b": [["29999.9", "2.5"]]
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v5/market/orderbook")
                .query_param("category", "spot")
                .query_param("symbol", &pair)
                .query_param("limit", "100");
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("test_api_key", "test_secret_key", &server.base_url());
        let result = ingestor.get_order_book(symbol.clone()).await.unwrap();

        mock.assert();

        assert_eq!(result.symbol, symbol);
        assert_eq!(result.book.asks.len(), 1);
        assert_eq!(result.book.bids.len(), 1);
        assert_eq!(result.book.asks[0].price, 30000.1);
        assert_eq!(result.book.asks[0].quantity, 1.5);
        assert_eq!(result.book.bids[0].price, 29999.9);
        assert_eq!(result.book.bids[0].quantity, 2.5);
    }

    #[tokio::test]
    async fn test_get_order_book_non_success_status() {
        let server = MockServer::start();
        let symbol = Symbol::from("BTC");
        let pair = format!("{}{}", symbol.to_string(), "/USDT");

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v5/market/orderbook")
                .query_param("category", "spot")
                .query_param("symbol", &pair)
                .query_param("limit", "100");
            then.status(500);
        });

        let ingestor = make_test_ingestor("test_api_key", "test_secret_key", &server.base_url());
        let result = ingestor.get_order_book(symbol.clone()).await.unwrap();

        mock.assert();

        assert_eq!(result.symbol, symbol);
        assert!(result.book.asks.is_empty());
        assert!(result.book.bids.is_empty());
    }

    #[tokio::test]
    async fn test_get_order_book_retcode_not_zero() {
        let server = MockServer::start();
        let symbol = Symbol::from("BTC");
        let pair = format!("{}{}", symbol.to_string(), "/USDT");

        let mock_response = serde_json::json!({
            "retCode": 1,
            "retMsg": "Error",
            "result": {
                "s": pair,
                "a": [["30000.1", "1.5"]],
                "b": [["29999.9", "2.5"]]
            }
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v5/market/orderbook")
                .query_param("category", "spot")
                .query_param("symbol", &pair)
                .query_param("limit", "100");
            then.status(200)
                .header("content-type", "application/json")
                .body(mock_response.to_string());
        });

        let ingestor = make_test_ingestor("test_api_key", "test_secret_key", &server.base_url());
        let result = ingestor.get_order_book(symbol.clone()).await.unwrap();

        mock.assert();

        assert_eq!(result.symbol, symbol);
        assert!(result.book.asks.is_empty());
        assert!(result.book.bids.is_empty());
    }

    #[tokio::test]
    async fn test_get_exchange_status_online() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v5/market/time");
            then.status(200);
        });

        let ingestor = make_test_ingestor("api_key", "secret_key", &server.base_url());
        let result = ingestor.get_exchange_status().await.unwrap();

        mock.assert();

        assert_eq!(result.status, crate::common::types::ExchangeStatus::ONLINE);
    }

    #[tokio::test]
    async fn test_get_exchange_status_offline() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v5/market/time");
            then.status(500);
        });

        let ingestor = make_test_ingestor("api_key", "secret_key", &server.base_url());
        let result = ingestor.get_exchange_status().await.unwrap();

        mock.assert();

        assert_eq!(result.status, crate::common::types::ExchangeStatus::OFFLINE);
    }

    #[tokio::test]
    async fn test_get_fees_returns_expected() {
        let ingestor = make_test_ingestor("api_key", "secret_key", "http://localhost");
        let res = ingestor.get_fees(1000.0).await.unwrap();
        assert!((res.maker_fee - 0.0010).abs() < 1e-9);
        assert!((res.taker_fee - 0.0010).abs() < 1e-9);
    }
}
