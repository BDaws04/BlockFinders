#[cfg(test)]
mod tests {
    use super::*;
    use crate::order_book::UnifiedOrderBook;
    use crate::types::{OBOrder, OrderSide};
    use tokio::sync::mpsc::unbounded_channel;
    use std::sync::Arc;
    use std::time::Instant;
    use futures::future::join_all;
    use rand::Rng;  

    #[tokio::test]
    async fn benchmark_process_1m_orders_from_10_tasks() {
        let (sender, receiver) = unbounded_channel::<OBOrder>();
        let order_book = Arc::new(UnifiedOrderBook::new(receiver));
        let order_book_clone = Arc::clone(&order_book);

        // Start the order book processing
        tokio::spawn(async move {
            order_book_clone.run().await;
        });

        // Give the order book a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let total_orders = 1_000_000usize;
        let num_tasks = 10;
        let orders_per_task = total_orders / num_tasks;

        let start = Instant::now();

        let mut handles = Vec::with_capacity(num_tasks);

        for _ in 0..num_tasks {
            let sender_clone = sender.clone();
            handles.push(tokio::spawn(async move {
                let mut rng = rand::thread_rng();
                for _ in 0..orders_per_task {
                    let side = if rng.gen_bool(0.5) {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    };
                    let price = rng.gen_range(950..=1050);      
                    let volume = rng.gen_range(100..=10_000);    

                    let order = OBOrder {
                        exchange: "TestExchange".to_string(),
                        side,
                        price,
                        volume,
                    };

                    if sender_clone.send(order).is_err() {
                        break;
                    }
                }
            }));
        }

        join_all(handles).await;

        let duration = start.elapsed();
        println!("Processed 1,000,000 orders in {:?}", duration);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}


