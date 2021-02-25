use std::sync::mpsc;

use crate::sign::sign;
use chrono::{Duration, Utc};
use reqwest;
use reqwest::StatusCode;

use crate::model::{Side, OrderStatus, OrderType, ExchangeOrder, ExchangeCmd, ExchangeCmd::*, OrchestratorEvent, OrchestratorEvent::*};
use crate::model::OrderType::{Limit, Market};
use crate::rest_model::{Response, Order};
use std::collections::HashMap;


const API_ORDER_PATH: &str = "/api/v1/order";


#[tokio::main]
pub async fn issue_order(root_url: &str, api_key: &str, api_secret: &str, symbol: &str, order: &ExchangeOrder, tx: &mut mpsc::Sender<OrchestratorEvent>) -> Result<(), reqwest::Error> {
    // let mut params = HashMap::new();
    let url_params = match order {
        ExchangeOrder { cl_ord_id, ord_type, price, qty, side, .. } if ord_type.map_or_else(|| false, |x| x == Limit) => {
            vec![("symbol".to_string(),  symbol.to_string()),
             ("ordType".to_string(),     "Limit".to_string()),
             ("timeInForce".to_string(), "GoodTillCancel".to_string()),
             // ("execInst".to_string(),    "ParticipateDoNotInitiate".to_string()),
             ("orderQty".to_string(),    qty.unwrap().to_string()),
             ("side".to_string(),        side.unwrap().to_string()),
             ("price".to_string(),       price.unwrap().to_string()),
             ("clOrdID".to_string(),     cl_ord_id.to_string())]
            //format!("symbol={}&ordType={}&timeInForce=GoodTillCancel&orderQty={}&side={}&price={}&clOrdID={}", symbol, *ord_type, qty, side, price, cl_ord_id)
        }
        ExchangeOrder { cl_ord_id, ord_type, price, qty, side, .. } if ord_type.map_or_else(|| false, |x| x == Market) => {
            //let qty_str = qty.to_string();
            vec![("symbol".to_string(),  symbol.to_string()),
             ("ordType".to_string(),     "Limit".to_string()),
             ("timeInForce".to_string(), "GoodTillCancel".to_string()),
             ("orderQty".to_string(),    qty.unwrap().to_string()),
             ("side".to_string(),        side.unwrap().to_string()),
             ("clOrdID".to_string(),     cl_ord_id.to_string())]
            //format!("symbol={}&ordType={}&timeInForce=GoodTillCancel&orderQty={}&side={}&clOrdID={}", symbol, *ord_type, qty, side, cl_ord_id)
        }
        other =>
            panic!("Unexpected ExchangeOrder: {:?}", other)
    };


    let expires = (Utc::now() + Duration::seconds(100)).timestamp();
    let url_params_str = url_params.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("&");
    let signature = sign(&format!("POST{}{}{}", API_ORDER_PATH, expires, &url_params_str), api_secret);

    let client = reqwest::Client::new();
    let req = client
        .post(&format!("{}{}", root_url, API_ORDER_PATH))
        // .header("Content-Type", "application/x-www-form-urlencoded")
        .header("api-expires", expires)
        .header("api-key", api_key)
        .header("api-signature", signature)
        // .body(url_params_str);
        .form(&url_params);

    let res = req.send().await?;
    match res.status() {
        StatusCode::OK => {
            let resp_body = res.text().await?;
            match serde_json::from_str::<Response>(&resp_body) {
                Ok(Response::Order(Order { cl_ord_id, ord_status, ord_type,  price, order_qty, side, .. })) => {
                    tx.send(
                        UpdateOrder(ExchangeOrder {
                            cl_ord_id: cl_ord_id.to_string(),
                            ord_status: ord_status.clone(),
                            ord_type: ord_type.clone(),
                            price: Some(price.clone()),
                            qty: Some(order_qty.clone()),
                            side: Some(side)}.clone()
                        ));
                }
                Err(err) =>
                    panic!("Error {} for unknown rest model: {:?}", resp_body, err)
            }
        }
        _ => {
            tx.send(NewStatus(format!("Received unexpected http response: {:?}", res)));
        }
    }
    Ok(())
}

#[tokio::main]
pub async fn cancel_order(root_url: &str, api_key: &str, api_secret: &str, cl_ord_id: &str, tx: &mut mpsc::Sender<OrchestratorEvent>) -> Result<(), reqwest::Error> {
    let url_params = format!("clOrdID={}", cl_ord_id);
    let expires = (Utc::now() + Duration::seconds(100)).timestamp();
    let signature = sign(&format!("DELETE{}{}{}", API_ORDER_PATH, expires, &url_params), api_secret);

    let client = reqwest::Client::new();
    let res = client
        .delete(&format!("{}{}", root_url, API_ORDER_PATH))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("api-expires", expires.to_string())
        .header("api-key", api_key)
        .header("api-signature", signature)
        .body(url_params)
        .send()
        .await?;

    tx.send(NewStatus(res.text().await?.to_string()));
    Ok(())
}