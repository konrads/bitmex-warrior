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
        Response::Subscribe{ subscribe: "trade".to_string(), success: true }
    );
    assert_eq!(
        serde_json::from_str::<Response>(r#"{"table": "orderBook10", "action": "update", "data": [{"timestamp": "2020-01-01T00:00:00Z", "symbol": "XBTUSD", "asks": [[1.1, 2.2], [3.3, 4.4]], "bids": [[8.8, 9.9]] }]}"#).unwrap(),
        Response::Table(Table::OrderBook10 { action: TableAction::Update, data: vec![OrderBookRow{ timestamp: ts.clone(), symbol: "XBTUSD".to_string(), asks: vec![[1.1, 2.2], [3.3, 4.4]], bids: vec![[8.8, 9.9]] }] })
    );
    assert_eq!(
        serde_json::from_str::<Response>(r#"{"table":"orderBook10","action":"update","keys":["symbol"],"types":{"symbol":"symbol","bids":"","asks":"","timestamp":"timestamp"},"foreignKeys":{"symbol":"instrument"},"attributes":{"symbol":"sorted"},"filter":{"symbol":"XBTUSD"},"data":[{"symbol":"XBTUSD","bids":[[51189,112760],[51188.5,2],[51187,2000],[51182,12000],[51180,700],[51178.5,88500],[51178,188],[51177,5000],[51176,1694],[51175.5,1250]],"asks":[[51189.5,903974],[51190,59762],[51190.5,1005095],[51192,10000],[51193,24953],[51193.5,200000],[51194,73398],[51195,100],[51195.5,2100],[51196,303]],"timestamp":"2020-01-01T00:00:00Z"}]}"#).unwrap(),
        Response::Table(Table::OrderBook10 { action: TableAction::Update, data: vec![OrderBookRow{ timestamp: ts.clone(), symbol: "XBTUSD".to_string(), asks: vec![[51189.5, 903974.0], [51190.0, 59762.0], [51190.5, 1005095.0], [51192.0, 10000.0], [51193.0, 24953.0], [51193.5, 200000.0], [51194.0, 73398.0], [51195.0, 100.0], [51195.5, 2100.0], [51196.0, 303.0]], bids: vec![[51189.0, 112760.0], [51188.5, 2.0], [51187.0, 2000.0], [51182.0, 12000.0], [51180.0, 700.0], [51178.5, 88500.0], [51178.0, 188.0], [51177.0, 5000.0], [51176.0, 1694.0], [51175.5, 1250.0]] }] })
    );
}
