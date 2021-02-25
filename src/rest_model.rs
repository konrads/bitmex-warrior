use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;
use super::model::{Side, OrderStatus, OrderType};


#[derive(Deserialize, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(rename = "clOrdID")]
    pub cl_ord_id: String,
    pub symbol: String,
    pub side: Side,
    pub order_qty: f64,
    pub ord_status: OrderStatus,
    pub ord_type: Option<OrderType>,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Response {
    Order(Order)
}
