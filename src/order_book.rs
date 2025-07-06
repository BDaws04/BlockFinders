use std::collections::{BTreeMap, VecDeque};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use crate::types::{OBOrder, OrderRequest, OrderSide, PriceResponse};
struct SideOrderBook {
    orders: BTreeMap<u64, VecDeque<OBOrder>>,
    query_receiver: UnboundedReceiver<OrderRequest>,
    receiver: UnboundedReceiver<OBOrder>,
    active: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    is_buy: bool
}
use crate::errors::OrderBookError;


impl SideOrderBook {
    fn new(query_receiver: UnboundedReceiver<OrderRequest>, receiver: UnboundedReceiver<OBOrder>, is_buy: bool) -> Self {
        Self {
            orders: BTreeMap::new(),
            query_receiver,
            receiver,
            active: Arc::new(AtomicBool::new(true)),
            pause: Arc::new(AtomicBool::new(false)),    
            is_buy,
        }
    }

    async fn run(mut self) {
        while self.active.load(Ordering::SeqCst) {
            while self.pause.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }

            tokio::select! {
                Some(order) = self.receiver.recv() => {
                    self.process_order(order);
                }

                Some(order_request) = self.query_receiver.recv() => {
                    if let Ok(response) = self.get_best_quote(order_request) {
                        println!("Best quote: {:?}", response);
                    } else {
                        println!("Failed to get best quote");
                    }
                }

                else => {
                    self.active.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }
    }


    fn process_order(&mut self, order: OBOrder) {
        let price = order.price;
        if let Some(queue) = self.orders.get_mut(&order.price) {
            let mut i = 0;
            let mut found = false;
            while i < queue.len() {
                if queue[i].exchange == order.exchange {
                    found = true;
                    if order.volume == 0 {
                        queue.remove(i);
                    } else {
                        queue[i].volume = order.volume;
                    }
                    break;
                }
                i += 1;
            }
            if !found && order.volume != 0 {
                queue.push_back(order);
            }
        } else if order.volume != 0 {
            let mut queue = VecDeque::new();
            queue.push_back(order);
            self.orders.insert(price, queue);
        }

        if self.orders.get(&price).map_or(false, |queue| queue.is_empty()) {
            self.orders.remove(&price);
        }
    }

    pub fn get_best_quote(&self, order: OrderRequest) -> Result<PriceResponse, OrderBookError> {
        println!("Made it to get_best_quote with order: {:?}", order);
        if !self.active.load(Ordering::SeqCst) {
            return Err(OrderBookError::InactiveOrderBook);
        }

        self.pause.store(true, Ordering::SeqCst);

        let formatted_volume = (order.volume * 1_000_000.0) as u64; 
        let symbol = order.symbol.clone();

        let mut total_volume = 0u64;
        let mut alpaca_volume = 0u64;
        let mut kraken_volume = 0u64;
        let mut bybit_volume = 0u64;
        let mut weighted_price_sum = 0u128; 

        if self.is_buy {
            for (&price, queue) in self.orders.iter().rev() {
                for order in queue.iter() {
                    let avail = order.volume.min(formatted_volume - total_volume);
                    if avail == 0 {
                        break;
                    }

                    total_volume += avail;
                    weighted_price_sum += (price as u128) * (avail as u128);

                    match order.exchange.as_str() {
                        "Alpaca" => alpaca_volume += avail,
                        "Kraken" => kraken_volume += avail,
                        "Bybit" => bybit_volume += avail,
                        _ => {}
                    }

                    if total_volume >= formatted_volume {
                        break;
                    }
                }
                if total_volume >= formatted_volume {
                    break;
                }
            }
        } else {
            for (&price, queue) in self.orders.iter() {
                for order in queue.iter() {
                    let avail = order.volume.min(formatted_volume - total_volume);
                    if avail == 0 {
                        break;
                    }

                    total_volume += avail;
                    weighted_price_sum += (price as u128) * (avail as u128);

                    match order.exchange.as_str() {
                        "Alpaca" => alpaca_volume += avail,
                        "Kraken" => kraken_volume += avail,
                        "Bybit" => bybit_volume += avail,
                        _ => {}
                    }

                    if total_volume >= formatted_volume {
                        break;
                    }
                }
                if total_volume >= formatted_volume {
                    break;
                }
            }
        }

        self.pause.store(false, Ordering::SeqCst);

        if total_volume == 0 {
            return Err(OrderBookError::InsufficientVolume("Not enough volume available".to_string()));
        }

        let vwap_scaled = (weighted_price_sum / total_volume as u128) as f64 / 100.0;

        let total_volume_scaled = total_volume as f64 / 1_000_000.0;
        let alpaca_volume_scaled = alpaca_volume as f64 / 1_000_000.0;
        let kraken_volume_scaled = kraken_volume as f64 / 1_000_000.0;
        let bybit_volume_scaled = bybit_volume as f64 / 1_000_000.0;

        Ok(PriceResponse {
            symbol,
            side: if self.is_buy { OrderSide::Buy } else { OrderSide::Sell },
            total_volume: total_volume_scaled,
            alpaca_volume: alpaca_volume_scaled,
            kraken_volume: kraken_volume_scaled,
            bybit_volume: bybit_volume_scaled,
            vwap: vwap_scaled,
        })
    }



    }


pub struct UnifiedOrderBook {
    main_receiver: tokio::sync::Mutex<UnboundedReceiver<OBOrder>>,
    buy_sender: UnboundedSender<OBOrder>,
    sell_sender: UnboundedSender<OBOrder>,
    buy_query_sender: UnboundedSender<OrderRequest>,
    sell_query_sender: UnboundedSender<OrderRequest>,
    active: Arc<AtomicBool>,
}

impl UnifiedOrderBook {
    pub fn new(main_receiver: UnboundedReceiver<OBOrder>) -> Self {
        let (buy_sender, buy_receiver) = unbounded_channel();
        let (sell_sender, sell_receiver) = unbounded_channel();

        let (buy_query_sender, buy_query_receiver) = unbounded_channel();
        let (sell_query_sender, sell_query_receiver) = unbounded_channel();

        let active = Arc::new(AtomicBool::new(true));

        {
            tokio::spawn(SideOrderBook::new(buy_query_receiver, buy_receiver, true).run());
        }
        {
            tokio::spawn(SideOrderBook::new(sell_query_receiver, sell_receiver, false).run());
        }

        Self {
            main_receiver: tokio::sync::Mutex::new(main_receiver),
            buy_sender,
            sell_sender,
            buy_query_sender,
            sell_query_sender,
            active,

        }
    }

    pub async fn run(&self) {
        while self.active.load(Ordering::SeqCst) {
            let maybe_order = {
                // Lock mutex just for the recv call
                let mut receiver = self.main_receiver.lock().await;
                receiver.recv().await
            };

            match maybe_order {
                Some(order) => {
                    match order.side {
                        OrderSide::Buy => {
                            let _ = self.buy_sender.send(order);
                        }
                        OrderSide::Sell => {
                            let _ = self.sell_sender.send(order);
                        }
                    }
                }
                None => {
                    self.active.store(false, Ordering::SeqCst);
                }
            }
        }
    }


    pub async fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    pub async fn get_quote(&self, order: OrderRequest) -> Result<(), OrderBookError> {
        match order.side {
            OrderSide::Buy => self.buy_query_sender.send(order).map_err(|_| OrderBookError::ChannelSendError),
            OrderSide::Sell => self.sell_query_sender.send(order).map_err(|_| OrderBookError::ChannelSendError),
        }
    }
}
