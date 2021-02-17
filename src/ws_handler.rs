use crate::model::{Side, OrchestratorEvent::*};
use crate::ws_model::{TableType, Request, Response, Response::*};
use std::sync::mpsc;


pub struct Client<'a, T> {
    ws_sender: ws::Sender,
    tx: &'a mpsc::Sender<T>,
}

impl <'a, T> Client<'a, T> {
    pub fn new(out: ws::Sender, tx: &'a mpsc::Sender<T>) -> Self {
        Client { ws_sender: out, tx: tx }
    }
}


// impl<'a, T: serde::de::DeserializeOwned> ws::Handler for Client<'a, T> {
//     fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
//         let subscribe = Request::Subscribe {
//             args: vec![
//                 TableType::Trade,
//                 TableType::Funding,
//                 TableType::OrderBookL2,
//             ],
//         };
//
//         let ser = serde_json::to_string(&subscribe).unwrap();  // FIXME: apply error conversion
//         log::info!("Sending subscribe command: {:?}", ser);
//         self.ws_sender.send(ser)
//     }
//
//     fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
//         let payload: String = msg.into_text().unwrap();
//         match serde_json::from_str::<Response>(&payload) {
//             Ok(Subscribe { subscribe, success }) =>
//                 log::info!("Subscribed: {}: success: {}", subscribe, success),
//             Ok(i @ Info { .. }) =>
//                 log::info!("{:?}", i),
//             Ok(e @ Error { .. }) =>
//                 log::info!("{:?}", e),
//             Ok(TableData{ table, ref data, .. }) if table == Table::OrderBookL2 => {
//                 let latest_buy = data.iter().filter(|x| (**(x as &&OrderBookEntry)).side == Side::Buy).max_by_key(|x| (**x).timestamp);
//                 let latest_sell = data.iter().filter(|x| (**(x as &&OrderBookEntry)).side == Side::Buy).max_by_key(|x| (**x).timestamp);
//                 if let Some(buy) = latest_buy {
//                     self.tx.send(NewBid(buy.price));
//                 }
//                 if let Some(sell) = latest_sell {
//                     self.tx.send(NewAsk(sell.price));
//                 }
//             }
//             Err(err) =>
//                 log::error!("{}", err),
//         }
//         Ok(()) // never fail
//     }
//
//     fn on_error(&mut self, err: ws::Error) {
//         log::error!("On Error, {}", err)
//     }
// }
