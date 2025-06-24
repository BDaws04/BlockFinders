mod common;
mod ingestor;
mod execution;
mod config;
use dotenv::dotenv;
use std::env;

use crate::execution::order_book::OrderBook;
use crate::common::types::{RoutedOrder, Symbol, OrderSide};

use crate::ingestor::{alpaca_ingestor, bybit_ingestor, kraken_ingestor};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let alpaca_api_key = env::var("ALPACA_API_KEY").expect("ALPACA_API_KEY not set");
    let alpaca_api_secret = env::var("ALPACA_API_SECRET").expect("ALPACA_API_SECRET not set");
    let bybit_api_key = env::var("BYBIT_API_KEY").expect("BYBIT_API_KEY not set");
    let bybit_api_secret = env::var("BYBIT_API_SECRET").expect("BYBIT_API_SECRET not set");
    let kraken_api_key = env::var("KRAKEN_API_KEY").expect("KRAKEN_API_KEY not set");
    let kraken_api_secret = env::var("KRAKEN_API_SECRET").expect("KRAKEN_API_SECRET not set");

    let alpaca_ingestor = alpaca_ingestor::AlpacaIngestor::new(alpaca_api_key, alpaca_api_secret, "https://data.alpaca.markets".to_string());
    let bybit_ingestor = bybit_ingestor::BybitIngestor::new(bybit_api_key, bybit_api_secret, "https://api-testnet.bybit.com".to_string());
    let kraken_ingestor = kraken_ingestor::KrakenIngestor::new(kraken_api_key, kraken_api_secret, "https://api.kraken.com/0/public/".to_string());

    let order_book = OrderBook::new(kraken_ingestor, bybit_ingestor, alpaca_ingestor);
    let routed_order: Result<RoutedOrder, anyhow::Error> = order_book.route_order(Symbol::ETH, OrderSide::Buy, 20.0).await;
    println!("{:?}", routed_order);

}

