use chrono::{Duration, Utc};
use thiserror::Error;
use reqwest::StatusCode;

use crate::model::{ExchangeOrder, OrchestratorEvent, OrchestratorEvent::*};
use crate::model::OrderType::{Limit, Market};
use crate::rest_model::{Order, Response};
use crate::sign::sign;

const API_ORDER_PATH: &str = "/api/v1/order";


/// Issue an Order.
#[tokio::main]
pub async fn issue_order(root_url: &str, api_key: &str, api_secret: &str, symbol: &str, order: &ExchangeOrder) -> Result<OrchestratorEvent, RestError> {
    let url_params = match order {
        ExchangeOrder { cl_ord_id, ord_type, price, qty, side, .. } if ord_type.map_or_else(|| false, |x| x == Limit) => {
            vec![("symbol",  symbol.to_string()),
             ("ordType",     "Limit".to_string()),
             ("timeInForce", "GoodTillCancel".to_string()),
             // ("execInst",    "ParticipateDoNotInitiate".to_string()),
             ("orderQty",    qty.unwrap().to_string()),
             ("side",        side.unwrap().to_string()),
             ("price",       price.unwrap().to_string()),
             ("clOrdID",     cl_ord_id.to_string())]
            //format!("symbol={}&ordType={}&timeInForce=GoodTillCancel&orderQty={}&side={}&price={}&clOrdID={}", symbol, *ord_type, qty, side, price, cl_ord_id)
        }
        ExchangeOrder { cl_ord_id, ord_type, qty, side, .. } if ord_type.map_or_else(|| false, |x| x == Market) => {
            //let qty_str = qty.to_string();
            vec![("symbol",  symbol.to_string()),
             ("ordType",     "Market".to_string()),
             ("timeInForce", "GoodTillCancel".to_string()),
             ("orderQty",    qty.unwrap().to_string()),
             ("side",        side.unwrap().to_string()),
             ("clOrdID",     cl_ord_id.to_string())]
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
            match serde_json::from_str::<Response>(&resp_body)? {
                Response::Order(Order { cl_ord_id, ord_status, ord_type,  price, order_qty, side, .. }) => {
                    Ok(
                        UpdateOrder(ExchangeOrder {
                            cl_ord_id,
                            ord_status,
                            ord_type,
                            price: Some(price),
                            qty: Some(order_qty),
                            side: Some(side)
                        }))
                }
            }
        }
        status => {
            Ok(NewStatus(format!("Received unexpected http response status {}: {:?}\nreq: {:?}", status, res.text().await?, url_params_str)))
        }
    }
}

/// Cancel an Order.
#[tokio::main]
pub async fn cancel_order(root_url: &str, api_key: &str, api_secret: &str, cl_ord_id: &str) -> Result<OrchestratorEvent, RestError> {
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

    Ok(NewStatus(res.text().await?))
}


#[derive(Error, Debug)]
pub enum RestError {
    #[error("http response parse error: {0:?}")]
    HttpError(#[from] reqwest::Error),
    #[error("json parse error: {0:?}")]
    ParseError(#[from] serde_json::Error),
}
