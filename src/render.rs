use model::State;

use crate::model;


/// Render the UI according to the current state.
pub fn render_state(header: &str, state: &State) -> String {
    let recent_order_if_present = match state.order {
        Some(ref o) => format!("\r\nCURR ORDER: {} {} {} {:.2} @ {:.2}", o.side.unwrap(), o.ord_type.unwrap(), o.ord_status, o.qty.unwrap(), o.price.unwrap()),
        None => "".to_string()
    };
    format!("{}\r
\r
BID: {:.2} / ASK: {:.2}\r
QTY: {:.2}\r
ORDER TYPE: {}\r
STATUS: {}{}",
            header, state.bid, state.ask, state.qty, state.order_type(), state.status, recent_order_if_present)
}