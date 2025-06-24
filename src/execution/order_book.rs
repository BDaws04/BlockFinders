use crate::ingestor::bybit_ingestor::BybitIngestor;
use crate::ingestor::kraken_ingestor::KrakenIngestor;
use crate::ingestor::alpaca_ingestor::AlpacaIngestor;
use crate::common::types::{OrderBookResponse, Symbol, RoutedOrder, OrderSide, MaxPriceLevel, MinPriceLevel, Exchanges};
use crate::ingestor::market_data_ingestor::MarketDataIngestor; 
use std::collections::BinaryHeap;

pub struct OrderBook {
    kraken: KrakenIngestor,
    bybit: BybitIngestor,
    alpaca: AlpacaIngestor,
    // add executors for each exchange
}

impl OrderBook {
    pub fn new(kraken: KrakenIngestor, bybit: BybitIngestor, alpaca: AlpacaIngestor) -> Self {
        OrderBook {
            kraken,
            bybit,
            alpaca,
        }
    }

    pub async fn route_order(&self, symbol: Symbol, side: OrderSide, amount: f64) -> Result<RoutedOrder, anyhow::Error> {
            let alpaca_order_book_response: OrderBookResponse = self.alpaca.get_order_book(symbol.clone()).await?;
            let kraken_order_book_response: OrderBookResponse = self.kraken.get_order_book(symbol.clone()).await?;
            let bybit_order_book_response: OrderBookResponse = self.bybit.get_order_book(symbol.clone()).await?;
        if side == OrderSide::Sell{
            let mut bids: BinaryHeap<MaxPriceLevel> = BinaryHeap::new();
            for level in alpaca_order_book_response.book.bids.iter() {
                bids.push(MaxPriceLevel(*level));
                println!("Alpaca Bid: {:?}", level);
            }
            for level in kraken_order_book_response.book.bids.iter() {
                bids.push(MaxPriceLevel(*level));
                println!("Kraken Bid: {:?}", level);
            }
            for level in bybit_order_book_response.book.bids.iter() {
                bids.push(MaxPriceLevel(*level));
                println!("Bybit Bid: {:?}", level);
            }
            let mut required_volume = amount;
            let mut bybit_volume = 0.0;
            let mut kraken_volume = 0.0;
            let mut alpaca_volume = 0.0;
            while required_volume > 0.0 && !bids.is_empty() {
                let max_level = bids.pop().unwrap();
                let volume_at_level = max_level.0.quantity;
                if volume_at_level <= required_volume {
                    required_volume -= volume_at_level;
                    match max_level.0.exchange {
                        Exchanges::Bybit => bybit_volume += volume_at_level,
                        Exchanges::Kraken => kraken_volume += volume_at_level,
                        Exchanges::Alpaca => alpaca_volume += volume_at_level,
                    }
                } else {
                    match max_level.0.exchange {
                        Exchanges::Bybit => bybit_volume += required_volume,
                        Exchanges::Kraken => kraken_volume += required_volume,
                        Exchanges::Alpaca => alpaca_volume += required_volume,
                    }
                    required_volume = 0.0;
                }
            }
            if required_volume > 0.0 {
                return Err(anyhow::anyhow!("Not enough volume available to fill the order"));
            }
            Ok(RoutedOrder {
                symbol,
                side,
                bybit_volume,
                kraken_volume,
                alpaca_volume,
            })
        } else {
            let mut asks: BinaryHeap<MinPriceLevel> = BinaryHeap::new();
            for level in alpaca_order_book_response.book.asks.iter() {
                asks.push(MinPriceLevel(*level));
                println!("Alpaca Ask: {:?}", level);
            }
            for level in kraken_order_book_response.book.asks.iter() {
                asks.push(MinPriceLevel(*level));
                println!("Kraken Ask: {:?}", level);
            }
            for level in bybit_order_book_response.book.asks.iter() {
                asks.push(MinPriceLevel(*level));
                println!("Bybit Ask: {:?}", level);
            }
            let mut required_volume = amount;
            let mut bybit_volume = 0.0;
            let mut kraken_volume = 0.0;
            let mut alpaca_volume = 0.0;
            while required_volume > 0.0 && !asks.is_empty() {
                let min_level = asks.pop().unwrap();
                let volume_at_level = min_level.0.quantity;
                if volume_at_level <= required_volume {
                    required_volume -= volume_at_level;
                    match min_level.0.exchange {
                        Exchanges::Bybit => bybit_volume += volume_at_level,
                        Exchanges::Kraken => kraken_volume += volume_at_level,
                        Exchanges::Alpaca => alpaca_volume += volume_at_level,
                    }
                } else {
                    match min_level.0.exchange {
                        Exchanges::Bybit => bybit_volume += required_volume,
                        Exchanges::Kraken => kraken_volume += required_volume,
                        Exchanges::Alpaca => alpaca_volume += required_volume,
                    }
                    required_volume = 0.0;
                }
            }
            if required_volume > 0.0 {
                return Err(anyhow::anyhow!("Not enough volume available to fill the order"));
            }
            Ok(RoutedOrder {
                symbol,
                side,
                bybit_volume,
                kraken_volume,
                alpaca_volume,
            })
        }
    }
}

    // Add more methods as needed