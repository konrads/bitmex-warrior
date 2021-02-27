use std::sync::mpsc;

use chrono::{Duration, Utc};
use tungstenite::{connect, Message};

use crate::model::{ExchangeOrder, OrchestratorEvent, OrchestratorEvent::*};
use crate::sign::sign;
use crate::ws_model::{Request, Response, Response::*, Table::*};


pub fn handle_msgs(url: &str, api_key: &str, api_secret: &str, subscriptions: Vec<String>, tx: &mpsc::Sender<OrchestratorEvent>) {
    let expires = (Utc::now() + Duration::seconds(100)).timestamp();
    let signature = sign(&format!("GET/realtime{}", expires), api_secret);
    let authenticate = Request::Authenticate(api_key.to_string(), expires, signature);
    let (mut ws_socket, _) = connect(url).expect(&format!("Failed to connect to ws_url {}", url));
    let subscribe = Request::Subscribe(subscriptions);
    ws_socket.write_message(Message::text(serde_json::to_string(&authenticate).expect(&format!("Failed to send authenticate event {:?}", &authenticate))));
    ws_socket.write_message(Message::text(serde_json::to_string(&subscribe).expect(&format!("Failed to send subscribe event {:?}", &subscribe))));

    loop {
        let msg= ws_socket.read_message().expect("Failed to read ws message");
        match msg {
            Message::Text(ref payload) => {
                match serde_json::from_str::<Response>(&payload) {
                    Ok(ws_resp) =>
                        for x in ws_resp_2_orchestrator_event(&ws_resp) {
                            tx.send(x).expect(&format!("Failed to ws send"));  // FIXME: how to pass x to err msg?
                        }
                    Err(err) =>
                        log::error!("channel error {} on payload {}", err, &payload),
                }
            }
            Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => break
        }
    }
}

fn ws_resp_2_orchestrator_event(resp: &Response) -> Vec<OrchestratorEvent> {
    match resp {
        Subscribe { subscribe, success } =>
            vec!(NewStatus(format!("Subscribed to {}: {}", subscribe, success))),
        Info { info, .. } =>
            vec!(NewStatus(format!("Info on: {}", info))),
        Error { error, .. } =>
            vec!(NewStatus(format!("Error on: {:?}", error))),
        Table(OrderBook10{ ref data, .. }) => {
            let mut events1 = data.first().iter().map(|x| NewAsk(x.first_ask())).collect::<Vec<OrchestratorEvent>>();
            let mut events2 = data.first().iter().map(|x| NewBid(x.first_bid())).collect::<Vec<OrchestratorEvent>>();
            events1.append(&mut events2);
            events1
        },
        Table(Order{ ref data, .. }) =>
            data.iter().map(|x|
                UpdateOrder(ExchangeOrder {
                    cl_ord_id:  x.cl_ord_id.to_string(),
                    ord_status: x.ord_status,
                    ord_type:   x.ord_type,
                    price:      x.price,
                    qty:        x.order_qty,
                    side:       x.side
                })).collect(),
        e @ Table { .. } => {
            log::info!("ignoring other table: {:?}", e);
            Vec::new()
        }
    }
}
