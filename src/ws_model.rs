use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;
use super::model::Side;


#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "op")]
pub enum Request {
    Subscribe { args: Vec<String> },
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

#[derive(Serialize, Deserialize, Debug, Display, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TableType {
    Trade,
    OrderBookL2,
    OrderBookL2_25,
    Funding,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
#[serde(tag = "table")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
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
        subscribe: TableType,
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
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct OrderRow {
    pub processed: Option<DateTime<Utc>>,
    pub symbol: String,
    pub id: u64,
    pub side: Side,
    pub size: Option<u64>,
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