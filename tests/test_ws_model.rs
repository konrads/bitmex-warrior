use chrono::prelude::*;
use bitmex_warrior::ws_model::*;
use serde_json;

#[cfg(test)]

const ts_str: &str = "2020-01-01T00:00:00Z";

#[test]
fn test_ws_model() {
    let ts= ts_str.parse::<DateTime<Utc>>().unwrap();
    assert_eq!(
        serde_json::from_str::<Response>(r#"{"info": "some_info", "version": "v1", "timestamp": "2020-01-01T00:00:00Z"}"#).unwrap(),
        Response::Info{ info: "some_info".to_string(), version: "v1".to_string(), timestamp: ts.clone() }
    );
    assert_eq!(
        serde_json::from_str::<Response>(r#"{"subscribe": "trade", "success": true}"#).unwrap(),
        Response::Subscribe{ subscribe: TableType::Trade, success: true }
    );
    assert_eq!(
        serde_json::from_str::<Response>(r#"{"table": "orderBookL2", "action": "update", "data": [{"timestamp": "2020-01-01T00:00:00Z", "symbol": "XBTUSD", "asks": [[1.1, 2.2], [3.3, 4.4]], "bids": [[8.8, 9.9]] }]}"#).unwrap(),
        Response::Table(Table::OrderBookL2 { action: TableAction::Update, data: vec![OrderBookRow{ timestamp: ts.clone(), symbol: "XBTUSD".to_string(), asks: vec![[1.1, 2.2], [3.3, 4.4]], bids: vec![[8.8, 9.9]] }] })
    );
}
