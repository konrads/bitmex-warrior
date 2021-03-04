use std::sync::mpsc;

use chrono::{Duration, Utc};
use tungstenite::{connect, Message};

use crate::model::{ExchangeOrder, OrchestratorEvent, OrchestratorEvent::*};
use crate::sign::sign;
use crate::ws_model::{Request, Response, Response::*, Table::*};


/// Listen to WS messages and push them to supplied channel.
pub fn handle_msgs(url: &str, api_key: &str, api_secret: &str, subscriptions: Vec<String>, tx: &mpsc::Sender<OrchestratorEvent>) {
    let expires = (Utc::now() + Duration::seconds(100)).timestamp();
    let signature = sign(&format!("GET/realtime{}", expires), api_secret);
    let authenticate = Request::Authenticate(api_key.to_string(), expires, signature);
    let (mut ws_socket, _) = connect(url).unwrap_or_else(|_| panic!("Failed to connect to ws_url {}", url));
    let subscribe = Request::Subscribe(subscriptions);
    ws_socket.write_message(Message::text(serde_json::to_string(&authenticate).unwrap_or_else(|_| panic!("Failed to parse authenticate event {:?}", &authenticate)))).unwrap_or_else(|_| panic!("Failed to send authenticate event {:?}", &authenticate));
    ws_socket.write_message(Message::text(serde_json::to_string(&subscribe).unwrap_or_else(|_| panic!("Failed to parse subscribe event {:?}", &subscribe)))).unwrap_or_else(|_| panic!("Failed to send subscribe event {:?}", &subscribe));

    loop {
        let msg= ws_socket.read_message().expect("Failed to read ws message");
        match msg {
            Message::Text(ref payload) => {
                match serde_json::from_str::<Response>(&payload) {
                    Ok(ws_resp) =>
                        for x in ws_resp_2_orchestrator_event(&ws_resp) {
                            tx.send(x).expect("Failed to ws send");  // FIXME: how to pass x to err msg?
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

/// Convert WS Response to OrchestratorEvent
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


/// Example of internal tests, allows for testing non-public fns.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::OrderStatus;
    use crate::ws_model::{TableAction::Insert, OrderRow};

    #[test]
    #[allow(non_snake_case)]
    fn test__ws_resp_2_orchestrator_event() {
        assert_eq!(
            ws_resp_2_orchestrator_event(
                &Table(Order{ action: Insert, data: vec!(OrderRow {
                    timestamp: None,
                    symbol: "XBTUSD".to_string(),
                    cl_ord_id: "12345".to_string(),
                    side: None,
                    ord_status: OrderStatus::New,
                    ord_type: None,
                    order_qty: None,
                    price: None
                })})),
                vec!(UpdateOrder(ExchangeOrder { cl_ord_id: "12345".to_string(), ord_status: OrderStatus::New, ord_type: None, price: None, qty: None, side: None }))
        );
    }
}
