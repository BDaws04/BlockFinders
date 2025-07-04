mod errors;
mod config;
mod exchanges;
mod types;
mod order_book;
use dotenv::dotenv;
use std::env;
use exchanges::exchange::Exchange;
use tokio::signal;
use std::sync::Arc;
use crate::exchanges::bybit;


#[tokio::main]
async fn main() {
    dotenv().ok();
    /* 
    let kraken_api_key = env::var("KRAKEN_API_KEY").unwrap();
    let kraken_api_secret = env::var("KRAKEN_API_SECRET").unwrap();
    let kraken_exchange = Arc::new(
        exchanges::kraken::KrakenExchange::new(kraken_api_key, kraken_api_secret),
    );
    match kraken_exchange.subscribe_ob(&config::TICKER).await {
        Ok(_) => println!("Subscribed to order book for {}", config::TICKER),
        Err(e) => eprintln!("Failed to subscribe: {}", e),
    }

    let alpaca_api_key = env::var("ALPACA_API_KEY").unwrap();
    let alpaca_api_secret = env::var("ALPACA_API_SECRET").unwrap();
    let alpaca_exchange = Arc::new(
        exchanges::alpaca::AlpacaExchange::new(alpaca_api_key, alpaca_api_secret),
    );
    match alpaca_exchange.subscribe_ob(&config::TICKER).await {
        Ok(_) => println!("Subscribed to order book for {}", config::TICKER),
        Err(e) => eprintln!("Failed to subscribe: {}", e),
    }
    */

    let bybit_api_key = env::var("BYBIT_API_KEY").unwrap();
    let bybit_api_secret = env::var("BYBIT_API_SECRET").unwrap();
    let bybit_exchange = Arc::new(
        exchanges::bybit::BybitExchange::new(bybit_api_key, bybit_api_secret),
    );

    match bybit_exchange.subscribe_ob(&config::TICKER).await {
        Ok(_) => println!("Subscribed to order book for {}", config::TICKER),
        Err(e) => eprintln!("Failed to subscribe: {}", e),
    }

    println!("Press Ctrl+C to exit...");


    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    match bybit_exchange.unsubscribe_ob(&config::TICKER).await {
        Ok(_) => println!("Unsubscribed from order book for {}", config::TICKER),
        Err(e) => eprintln!("Failed to unsubscribe: {}", e),
    }
    println!("Exiting...");
}