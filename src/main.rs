mod errors;
mod config;
mod exchanges;
mod types;
use dotenv::dotenv;
use std::env;
use exchanges::exchange::Exchange;
use tokio::signal;
use std::sync::Arc;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let kraken_api_key = env::var("KRAKEN_API_KEY").unwrap();
    let kraken_api_secret = env::var("KRAKEN_API_SECRET").unwrap();
    let kraken_exchange = Arc::new(
        exchanges::kraken::KrakenExchange::new(kraken_api_key, kraken_api_secret),
    );
    match kraken_exchange.subscribe_ob(&config::TICKER).await {
        Ok(_) => println!("Subscribed to order book for {}", config::TICKER),
        Err(e) => eprintln!("Failed to subscribe: {}", e),
    }
    println!("Press Ctrl+C to exit...");
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    match kraken_exchange.unsubscribe_ob(&config::TICKER).await {
        Ok(_) => println!("Unsubscribed from order book for {}", config::TICKER),
        Err(e) => eprintln!("Failed to unsubscribe: {}", e),
    }
    println!("Unsubscribed from order book for {}", config::TICKER);
    println!("Exiting...");
}