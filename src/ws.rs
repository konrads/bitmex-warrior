use std::sync::mpsc;

use chrono::{Duration, Utc};
use hex::encode as hexify;
use ring::hmac;
use tungstenite::{connect, Message};

use crate::sign::sign;
use crate::model::{Side, OrderStatus, OrderType, ExchangeOrder, OrchestratorEvent, OrchestratorEvent::*};
use crate::ws_model::{Request, Response, Response::*, Table::*};


pub fn handle_msg(url: &str, api_key: &str, api_secret: &str, subscriptions: Vec<String>, tx: &mpsc::Sender<OrchestratorEvent>) {
    let expires = (Utc::now() + Duration::seconds(100)).timestamp();
    let signature = sign(&format!("GET/realtime{}", expires), api_secret);
    let authenticate = Request::Authenticate(api_key.to_string(), expires, signature);
    let (mut ws_socket, _) = connect(url).unwrap();
    let subscribe = Request::Subscribe(subscriptions);
    ws_socket.write_message(Message::text(serde_json::to_string(&authenticate).unwrap()));
    ws_socket.write_message(Message::text(serde_json::to_string(&subscribe).unwrap()));

    loop {
        let msg= ws_socket.read_message().unwrap();
        match msg {
            Message::Text(ref payload) => {
                match serde_json::from_str::<Response>(&payload) {
                    Ok(Subscribe { subscribe, success }) => {
                        tx.send(NewStatus(format!("Subscribed to {}: {}", subscribe, success)));
                    }
                    Ok(Info { info, .. }) => {
                        tx.send(NewStatus(format!("Info on: {}", info)));
                    }
                    Ok(Error { error, .. }) => {
                        tx.send(NewStatus(format!("Error on: {:?}", error)));
                    }
                    Ok(Table(OrderBook10{ ref data, .. })) => {
                        data.first().map(|x| tx.send(NewAsk(x.first_ask())));
                        data.first().map(|x| tx.send(NewBid(x.first_bid())));
                    }
                    Ok(Table(Order{ ref data, .. })) => {
                        for x in data {
                            tx.send(
                                UpdateOrder(ExchangeOrder {
                                    cl_ord_id: x.cl_ord_id.to_string(),
                                    ord_status: x.ord_status.clone(),
                                    ord_type: x.ord_type.unwrap_or_else(|| OrderType::Market).clone(), // FIXME
                                    price: x.price.unwrap_or_else(|| -99.99),                          // FIXME should be opt?
                                    qty: x.order_qty.unwrap_or_else(|| -99.99),                        // FIXME should be opt?
                                    side: x.side.unwrap_or_else(|| Side::Buy).clone()  // FIXME
                                }));
                        }
                    }
                    Ok(e @ Table { .. }) =>
                        log::info!("other table: {:?}", e),
                    Err(err) =>
                        log::error!("channel error {} on payload {}", err, &payload),
                }
            }
            Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => break
        }
    }
}
