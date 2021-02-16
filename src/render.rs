use crate::model;
use model::{OrchestratorEvent, ExchangeAction, ExchangeOrder, OrderStatus, Side, State, OrchestratorEvent::*, PriceType::*, ExchangeAction::*};


pub fn render_state(header: &str, state: &State) -> String {
    let recent_order_if_present = match state.order {
        Some(ref o) => format!("\r\nCURR ORDER: {} {} {:.5} @ {:.5}", o.ord_type, o.ord_status, o.qty, o.price),
        None => "".to_string()
    };
    format!(r"{}

BID: {:.5} / ASK: {:.5}
QTY: {:.5}
ORDER TYPE: {}
STATUS: {}{}",
            header, state.bid, state.ask, state.qty, state.order_type(), state.status, recent_order_if_present)
}