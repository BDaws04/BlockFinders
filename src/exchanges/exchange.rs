use async_trait::async_trait;
use crate::errors::ExchangeError;
use crate::types::{Order};

#[async_trait]
pub trait Exchange {
    async fn subscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError>;
    async fn unsubscribe_ob(&self, symbol: &str) -> Result<(), ExchangeError>;

    async fn place_order(&self, order: Order) -> Result<(), ExchangeError>;
    async fn get_fees(&self) -> Result<f64, ExchangeError>;
}