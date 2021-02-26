use chrono::prelude::*;
use serde_json;

use bitmex_warrior::model::*;
use bitmex_warrior::rest_model;
use bitmex_warrior::rest_model::Order;
use bitmex_warrior::ws_model;
use bitmex_warrior::ws_model::{OrderBookRow, OrderRow, Table, TableAction};

#[cfg(test)]

const ts_str: &str = "2020-01-01T00:00:00Z";

#[test]
fn test_ws_model() {
    let ts= ts_str.parse::<DateTime<Utc>>().unwrap();
    assert_eq!(
        serde_json::from_str::<ws_model::Response>(r#"{"info": "some_info", "version": "v1", "timestamp": "2020-01-01T00:00:00Z"}"#).unwrap(),
        ws_model::Response::Info{ info: "some_info".to_string(), version: "v1".to_string(), timestamp: ts.clone() }
    );
    assert_eq!(
        serde_json::from_str::<ws_model::Response>(r#"{"subscribe": "trade", "success": true}"#).unwrap(),
        ws_model::Response::Subscribe{ subscribe: "trade".to_string(), success: true }
    );
    assert_eq!(
        serde_json::from_str::<ws_model::Response>(r#"{"table": "orderBook10", "action": "update", "data": [{"timestamp": "2020-01-01T00:00:00Z", "symbol": "XBTUSD", "asks": [[1.1, 2.2], [3.3, 4.4]], "bids": [[8.8, 9.9]] }]}"#).unwrap(),
        ws_model::Response::Table(Table::OrderBook10 { action: TableAction::Update, data: vec![OrderBookRow{ timestamp: ts.clone(), symbol: "XBTUSD".to_string(), asks: vec![[1.1, 2.2], [3.3, 4.4]], bids: vec![[8.8, 9.9]] }] })
    );
    assert_eq!(
        serde_json::from_str::<ws_model::Response>(r#"{"table":"orderBook10","action":"update","keys":["symbol"],"types":{"symbol":"symbol","bids":"","asks":"","timestamp":"timestamp"},"foreignKeys":{"symbol":"instrument"},"attributes":{"symbol":"sorted"},"filter":{"symbol":"XBTUSD"},"data":[{"symbol":"XBTUSD","bids":[[51189,112760],[51188.5,2],[51187,2000],[51182,12000],[51180,700],[51178.5,88500],[51178,188],[51177,5000],[51176,1694],[51175.5,1250]],"asks":[[51189.5,903974],[51190,59762],[51190.5,1005095],[51192,10000],[51193,24953],[51193.5,200000],[51194,73398],[51195,100],[51195.5,2100],[51196,303]],"timestamp":"2020-01-01T00:00:00Z"}]}"#).unwrap(),
        ws_model::Response::Table(Table::OrderBook10 { action: TableAction::Update, data: vec![OrderBookRow{ timestamp: ts.clone(), symbol: "XBTUSD".to_string(), asks: vec![[51189.5, 903974.0], [51190.0, 59762.0], [51190.5, 1005095.0], [51192.0, 10000.0], [51193.0, 24953.0], [51193.5, 200000.0], [51194.0, 73398.0], [51195.0, 100.0], [51195.5, 2100.0], [51196.0, 303.0]], bids: vec![[51189.0, 112760.0], [51188.5, 2.0], [51187.0, 2000.0], [51182.0, 12000.0], [51180.0, 700.0], [51178.5, 88500.0], [51178.0, 188.0], [51177.0, 5000.0], [51176.0, 1694.0], [51175.5, 1250.0]] }] })
    );
    assert_eq!(
        serde_json::from_str::<ws_model::Response>(r#"{"table":"order","action":"insert","data":[{"orderID":"8743ca01-d400-4799-a229-6e9ee72dd2b5","clOrdID":"xxx","clOrdLinkID":"","account":1502286,"symbol":"XBTUSD","side":"Buy","simpleOrderQty":null,"orderQty":10,"price":50097.5,"displayQty":null,"stopPx":null,"pegOffsetValue":null,"pegPriceType":"","currency":"USD","settlCurrency":"XBt","ordType":"Limit","timeInForce":"GoodTillCancel","execInst":"ParticipateDoNotInitiate","contingencyType":"","exDestination":"XBME","ordStatus":"New","triggered":"","workingIndicator":true,"ordRejReason":"","simpleLeavesQty":null,"leavesQty":10,"simpleCumQty":null,"cumQty":0,"avgPx":null,"multiLegReportingType":"SingleSecurity","text":"Submission from www.bitmex.com","transactTime":"2020-01-01T00:00:00Z","timestamp":"2020-01-01T00:00:00Z"}]}"#).unwrap(),
        ws_model::Response::Table(Table::Order { action: TableAction::Insert, data: vec![OrderRow { timestamp: Some(ts), symbol: "XBTUSD".to_string(), cl_ord_id: "xxx".to_string(), side: Some(Side::Buy), ord_status: OrderStatus::New, ord_type: Some(OrderType::Limit), order_qty: Some(10.0), price: Some(50097.5) }] })
    );
}

#[test]
fn test_rest_model() {
    assert_eq!(
        serde_json::from_str::<rest_model::Response>(r#"{"orderID":"e4f3f392-c2d0-4e4d-8e69-a57268431ea7","clOrdID":"4b2322e7-1e50-409c-80d7-ce894b7a9139","clOrdLinkID":"","account":299045,"symbol":"XBTUSD","side":"Buy","simpleOrderQty":null,"orderQty":100,"price":51170.5,"displayQty":null,"stopPx":null,"pegOffsetValue":null,"pegPriceType":"","currency":"USD","settlCurrency":"XBt","ordType":"Limit","timeInForce":"GoodTillCancel","execInst":"","contingencyType":"","exDestination":"XBME","ordStatus":"Filled","triggered":"","workingIndicator":false,"ordRejReason":"","simpleLeavesQty":null,"leavesQty":0,"simpleCumQty":null,"cumQty":100,"avgPx":51124.7444,"multiLegReportingType":"SingleSecurity","text":"Submitted via API.","transactTime":"2021-02-24T12:21:59.150Z","timestamp":"2021-02-24T12:21:59.150Z"}"#).unwrap(),
        rest_model::Response::Order(Order { cl_ord_id: "4b2322e7-1e50-409c-80d7-ce894b7a9139".to_string(), symbol: "XBTUSD".to_string(), side: Side::Buy, order_qty: 100.0, ord_status: OrderStatus::Filled, ord_type: Some(OrderType::Limit), price: 51170.5 })
    );
}