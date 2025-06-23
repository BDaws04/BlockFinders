#! [allow(unused_imports)]
#! [allow(dead_code)]


use async_trait::async_trait;
use anyhow::Result;
use crate::common::types::{Order, OrderBook, Symbol};

#[async_trait]
pub trait TradeExecutor: Send + Sync {
    // Order execution methods
    async fn place_order(&self, order: Order) -> Result<()>;
    
    async fn cancel_order(&self, order_id: &str) -> Result<()>;
    
    async fn get_order_status(&self, order_id: &str) -> Result<Order>;

    // Batch order execution methods
    async fn place_batch_orders(&self, orders: Vec<Order>) -> Result<()>;
    async fn cancel_batch_orders(&self, order_ids: Vec<String>) -> Result<()>;
}