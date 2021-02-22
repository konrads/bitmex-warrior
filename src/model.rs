use serde::{Deserialize, Serialize};
use std::fmt::Display;


#[derive(Serialize, Deserialize, Debug, Display, PartialEq, Clone, Copy)]
pub enum OrderType {
    Limit,
    StopLimit,
    Market
}
const ALL_ORDER_TYPES: &'static [OrderType] = &[OrderType::Limit, OrderType::StopLimit, OrderType::Market];

#[derive(Serialize, Deserialize, Debug, Display, PartialEq, Clone, Copy)]
pub enum OrderStatus {
    NotYetIssued,
    New,
    Filled,
    PartiallyFilled,
    Canceled,
    Rejected
}

#[derive(Serialize, Deserialize, Debug, Display, PartialEq, Clone, Copy)]
pub enum PriceType {
    Bid,
    Ask
}

#[derive(Serialize, Deserialize, Debug, Display, PartialEq, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExchangeOrder {
    pub cl_ord_id: String,
    pub ord_status: OrderStatus,
    pub ord_type: OrderType,
    pub price: f64,
    pub qty: f64,
    pub side: Side,
}

#[derive(Debug, PartialEq)]
pub enum OrchestratorEvent {
    Buy(PriceType),  // from user
    Sell(PriceType), // from user
    CancelLast,      // from user
    UpQty,           // from user
    DownQty,         // from user
    RotateOrderType, // from user
    NewBid(f64),     // from WS
    NewAsk(f64),     // from WS
    NewStatus(String),  // from WS
    UpdateOrder(ExchangeOrder),  // from WS/Rest
    Exit             // from user
}

#[derive(Debug, PartialEq)]
pub enum ExchangeAction<'a> {
    IssueOrder(ExchangeOrder),
    CancelOrder(&'a str)
}

#[derive(Debug, PartialEq)]
pub struct State {
    pub bid: f64,
    pub ask: f64,
    pub qty: f64,
    pub qty_increment: f64,
    pub order: Option<ExchangeOrder>,
    pub status: String,
    pub has_refreshed: bool,  // FIXME: shouldn't be public...
    pub order_type_ind: usize
}

impl State {
    pub fn new(qty: f64, qty_increment: f64)-> Self {
        State { bid: -1.0, ask: -1.0, qty: qty, qty_increment: qty_increment, order: None, status: "".to_string(), has_refreshed: false, order_type_ind: 0 }
    }

    pub fn order_type(&self) -> OrderType {
        (&ALL_ORDER_TYPES[self.order_type_ind]).clone()
    }

    pub fn rotate_order_type(&mut self) -> () {
        self.order_type_ind = (self.order_type_ind + 1) % ALL_ORDER_TYPES.len()
    }
}