use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;
use super::model::{Side, OrderStatus, OrderType};


#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "op", content = "args")]
pub enum Request{
    Subscribe(Vec<String>),
    #[serde(rename = "authKeyExpires")]
    Authenticate(String, i64, String)
}

#[derive(Serialize, Deserialize, Debug, Display, PartialEq)]
#[serde(rename_all = "lowercase")]
//#[serde(deny_unknown_fields)]
pub enum TableAction {
    Partial,
    Update,
    Insert,
    Delete,
}

#[derive(Serialize, Deserialize, Debug, Display, PartialEq)]
pub enum TickDirection {
    ZeroPlusTick,
    PlusTick,
    ZeroMinusTick,
    MinusTick,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
#[serde(tag = "table")]
#[serde(rename_all = "camelCase")]
//#[serde(deny_unknown_fields)]
pub enum Table {
    Trade {
        action: TableAction,
        data: Vec<TradeRow>
    },
    OrderBookL2 {
        action: TableAction,
        data: Vec<OrderBookRow>
    },
    OrderBookL2_25 {
        action: TableAction,
        data: Vec<OrderBookRow>
    },
    OrderBook10 {
        action: TableAction,
        data: Vec<OrderBookRow>,
    },
    Order {
        action: TableAction,
        data: Vec<OrderRow>
    },
    Funding {
        action: TableAction,
        data: Vec<FundingRow>
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Response {
    Info {
        info: String,
        version: String,
        timestamp: DateTime<Utc>,
    },
    Subscribe {
        subscribe: String,
        success: bool,
    },
    Error {
        status: u16,
        error: String,
        request: Request,
    },
    Table(Table)
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TradeRow {
    pub processed: Option<DateTime<Utc>>,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub side: Side,
    pub size: u64,
    pub price: f64,
    pub tick_direction: TickDirection,
    #[serde(rename = "trdMatchID")]
    pub trade_match_id: Uuid,
    pub gross_value: Option<u64>,
    pub home_notional: Option<f64>,
    pub foreign_notional: Option<f64>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
//#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct OrderRow {
    pub timestamp: Option<DateTime<Utc>>,
    pub symbol: String,
    #[serde(rename = "clOrdID")]
    pub cl_ord_id: String,
    pub side: Option<Side>,
    pub ord_status: OrderStatus,
    pub ord_type: Option<OrderType>,
    pub order_qty: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct FundingRow {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub funding_interval: DateTime<Utc>,
    pub funding_rate: f64,
    pub funding_rate_daily: f64,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookRow {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub asks: Vec<[f64; 2]>,
    pub bids: Vec<[f64; 2]>
}

impl OrderBookRow {
  pub fn first_bid(&self) -> f64 {
      self.bids.iter().map(|x| x[0]).fold(f64::NAN, f64::max)
  }
  pub fn first_ask(&self) -> f64 {
      self.asks.iter().map(|x| x[0]).fold(f64::NAN, f64::max)
  }
}
