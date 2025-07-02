mod errors;
mod config;
mod exchanges;
mod types;
use dotenv::dotenv;
use std::env;
use exchanges::exchange::Exchange;
use tokio::signal;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let kraken_api_key = env::var("KRAKEN_API_KEY").unwrap();
    let kraken_api_secret = env::var("KRAKEN_API_SECRET").unwrap();
    let kraken_exchange = exchanges::kraken::KrakenExchange::new(kraken_api_key, kraken_api_secret);
    match kraken_exchange.subscribe_ob("BTC").await {
        Ok(_) => println!("Subscribed to BTC order book"),
        Err(e) => eprintln!("Failed to subscribe: {}", e),
    }
    println!("Press Ctrl+C to exit...");
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    println!("Exiting...");
}