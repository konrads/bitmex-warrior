use serde::{Deserialize, Serialize};

use super::model::{OrderStatus, OrderType, Side};


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
