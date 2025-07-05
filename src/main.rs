mod errors;
mod config;
mod exchanges;
mod types;
mod order_book;

use dotenv::dotenv;
use std::{env, sync::Arc};
use tokio::{
    signal,
    sync::{mpsc::unbounded_channel, Notify, Mutex},
};
use crate::{
    exchanges::{alpaca, bybit, kraken},
    types::{OBOrder, OrderRequest, OrderSide},
    order_book::UnifiedOrderBook,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (sender, receiver) = unbounded_channel::<OBOrder>();

    let kraken_api_key = env::var("KRAKEN_API_KEY").expect("Missing KRAKEN_API_KEY");
    let kraken_api_secret = env::var("KRAKEN_API_SECRET").expect("Missing KRAKEN_API_SECRET");
    let bybit_api_key = env::var("BYBIT_API_KEY").expect("Missing BYBIT_API_KEY");
    let bybit_api_secret = env::var("BYBIT_API_SECRET").expect("Missing BYBIT_API_SECRET");
    let alpaca_api_key = env::var("ALPACA_API_KEY").expect("Missing ALPACA_API_KEY");
    let alpaca_api_secret = env::var("ALPACA_API_SECRET").expect("Missing ALPACA_API_SECRET");

    let kraken_exchange = Arc::new(kraken::KrakenExchange::new(kraken_api_key, kraken_api_secret, sender.clone()));
    let bybit_exchange = Arc::new(bybit::BybitExchange::new(bybit_api_key, bybit_api_secret, sender.clone()));
    let alpaca_exchange = Arc::new(alpaca::AlpacaExchange::new(alpaca_api_key, alpaca_api_secret, sender.clone()));

    let order_book = Arc::new(UnifiedOrderBook::new(receiver));
    let order_book_clone = order_book.clone();


    tokio::spawn(async move {
        order_book_clone.run().await;
    });

    println!("Unified order book has been started.");

    let shutdown_notify = Arc::new(Notify::new());

    let exchanges: Vec<(&str, Arc<dyn exchanges::exchange::Exchange + Send + Sync>)> = vec![
        ("Kraken", kraken_exchange.clone() as Arc<dyn exchanges::exchange::Exchange + Send + Sync>),
        ("Bybit", bybit_exchange.clone() as Arc<dyn exchanges::exchange::Exchange + Send + Sync>),
        ("Alpaca", alpaca_exchange.clone() as Arc<dyn exchanges::exchange::Exchange + Send + Sync>),
    ];

    for (name, exchange) in exchanges {
        let shutdown_notify = shutdown_notify.clone();
        let ticker = config::TICKER.to_string();
        tokio::spawn(async move {
            if let Err(e) = exchange.subscribe_ob(&ticker).await {
                eprintln!("Failed to subscribe to {}: {}", name, e);
                return;
            }
            println!("Subscribed to order book for {} on {}", ticker, name);

            shutdown_notify.notified().await;

            if let Err(e) = exchange.unsubscribe_ob(&ticker).await {
                eprintln!("Failed to unsubscribe from {}: {}", name, e);
            } else {
                println!("Unsubscribed from {} order book for {}", name, ticker);
            }
        });
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    let request = OrderRequest {
        symbol: config::TICKER.to_string(),
        side: OrderSide::Sell,
        volume: 1.0,
    };

    match order_book.get_quote(request).await {
        Ok(response) => {
            println!("Best quote: {:?}", response);
        }
        Err(e) => {
            eprintln!("Error getting best quote: {}", e);
        }
    }

    println!("Press Ctrl+C to exit...");
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    shutdown_notify.notify_waiters();

    println!("Exiting...");
}

