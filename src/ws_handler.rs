use crate::model::{OrchestratorEvent, OrchestratorEvent::*};
use crate::ws_model::{Request, Response, Response::*, Table::*};
use std::sync::mpsc;


pub struct Client<'a> {
    ws_sender: ws::Sender,
    tx: &'a mpsc::Sender<OrchestratorEvent>,
}

impl <'a> Client<'a> {
    pub fn new(out: ws::Sender, tx: &'a mpsc::Sender<OrchestratorEvent>) -> Self {
        Client { ws_sender: out, tx: tx }
    }
}


impl<'a> ws::Handler for Client<'a> {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        let subscribe = Request::Subscribe {
            args: vec![
                "trade:XBTUSD".to_string(),
                // "funding:XBTUSD".to_string(),
                "orderBook10:XBTUSD".to_string(),
            ],
        };

        let ser = serde_json::to_string(&subscribe).unwrap();  // FIXME: apply error conversion
        log::info!("Sending subscribe command: {:?}", ser);
        self.ws_sender.send(ser)
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let payload: String = msg.into_text().unwrap();
        match serde_json::from_str::<Response>(&payload) {
            Ok(Subscribe { subscribe, success }) =>
                log::info!("Subscribed: {}: success: {}", subscribe, success),
            Ok(i @ Info { .. }) =>
                log::info!("info: {:?}", i),
            Ok(e @ Error { .. }) =>
                log::info!("response error: {:?} on payload {}", e, &payload),
            Ok(Table(OrderBook10{ ref data, .. })) => {
                data.first().map(|x| self.tx.send(NewAsk(x.first_ask())));
                data.first().map(|x| self.tx.send(NewBid(x.first_bid())));
            }
            Ok(e @ Table { .. }) =>
                log::info!("other table: {:?}", e),
            Err(err) =>
                log::error!("channel error {} on payload {}", err, &payload),
        }
        Ok(()) // never fail
    }

    fn on_error(&mut self, err: ws::Error) {
        log::error!("On Error, {}", err)
    }
}
