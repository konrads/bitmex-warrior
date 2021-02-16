#[macro_use]
extern crate enum_display_derive;

// use env_logger;
mod model;
use model::*;
use model::{OrchestratorEvent::*, PriceType::*, ExchangeAction::*};
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

const USER_GUIDE: &str = "\
.-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-.\r
|              BITMEX WARRIOR           |\r
|                                       |\r
|  z -> buy @ bid      x -> sell @ ask  |\r
|  a -> buy @ ask      s -> sell @ bid  |\r
|  + -> up qty         - -> down qty    |\r
|  o -> rotate order types              |\r
|  q -> cancel last order               |\r
|  ctrl-c -> exit                       |\r
`-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-'\r
";

/// Design:
/// 1. accept configuration describing traded instrument, trading pot
/// 2. listen for keyboard shortcuts, eg.:
///   - ctrl-z for buy @ bid
///   - ctrl-x for sell @ ask
///   - ctrl-a for buy @ ask
///   - ctrl-s for sell @ bid
/// 3. display current bid/ask as per WS feeds
/// 4. list WebSocket events, perhaps in ncurses
/// 5...∞ mutations of the above
fn main() {
    let (tx, rx) = mpsc::channel::<OrchestratorEvent>();
    let orchestrator_thread = thread::spawn(move || {
        let mut state = State::new(10.0, 2.0);
        let mut stdout = stdout().into_raw_mode().unwrap();
        // write!(stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, USER_GUIDE, termion::cursor::Hide).unwrap();
        loop {
            match rx.recv() {
                Ok(e) if e == Exit => break,
                Ok(e) => {
                    stdout.flush().unwrap();
                    if let Some(order) = process_event(&e, &mut state) {
                        // send out the order
                        write!(stdout, "{}{}effect: {:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, order, termion::cursor::Hide).unwrap();
                        stdout.flush().unwrap();
                    }
                    if state.has_refreshed {
                        let render = render_state(&state);
                        write!(stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, render, termion::cursor::Hide).unwrap();
                        stdout.flush().unwrap();
                    }
                },
                Err(err) => {
                    log::error!("mpsc channel receive error: {:?}", err);
                    break
                },
            }
        }
    });

    let stdin = stdin();
    let mut prev_key = Key::Ctrl('.');  // some random key...
    // http://ticki.github.io/blog/making-terminal-applications-in-rust-with-termion/
    for c in stdin.keys() {
        let key = c.unwrap();
        // write!(stdout(), "{}{}...{:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, key, termion::cursor::Hide).unwrap();
        match key {
            Key::Char('+') => { tx.send(UpQty).unwrap(); () },
            Key::Char('-') => { tx.send(DownQty).unwrap(); () },
            Key::Char('o') => { tx.send(RotateOrderType).unwrap(); () },
            _ if key == prev_key => (),
            Key::Char('z') => { tx.send(Buy(Bid)).unwrap();  () },
            Key::Char('x') => { tx.send(Sell(Ask)).unwrap(); () },
            Key::Char('a') => { tx.send(Buy(Ask)).unwrap();  () },
            Key::Char('s') => { tx.send(Sell(Bid)).unwrap(); () },
            Key::Char('q') => { tx.send(CancelLast).unwrap(); () },
            Key::Ctrl('c') => { tx.send(Exit).unwrap(); break},
            _other => () // write!(stdout, "{}{}...{:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, other, termion::cursor::Hide).unwrap(),
        }
        prev_key = key;
  }
  orchestrator_thread.join().unwrap();
}

fn process_event<'a>(event: &'a OrchestratorEvent, state: &'a mut State) -> Option<ExchangeAction<'a>> {  // probably need dyn...
    state.has_refreshed = false;
    match event {
        UpQty => {
            state.has_refreshed = true;
            state.qty += state.qty_increment;
            None
        }
        DownQty if state.qty <= state.qty_increment => None,  // ignore, cannot decrease < 0
        DownQty => {
            state.has_refreshed = true;
            state.qty -= state.qty_increment;
            None
        }
        RotateOrderType => {
            state.rotate_order_type();
            state.has_refreshed = true;
            None
        }
        NewBid(bid) if state.bid == *bid => None,
        NewBid(bid) if state.bid == *bid => None,
        NewBid(bid) => {
            state.has_refreshed = true;
            state.bid = *bid;
            None
        }
        NewAsk(ask) if state.ask == *ask => None,
        NewAsk(ask) => {
            state.has_refreshed = true;
            state.ask = *ask;
            None
        }
        Buy(price_type) => {
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                Bid => state.bid,
                Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("new buy order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: state.qty, price: price, side: Side::Buy, ord_type: state.order_type() };
            Some(IssueOrder(new_order))
        }
        Sell(price_type) => {
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                Bid => state.bid,
                Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("new sell order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: state.qty, price: price, side: Side::Sell, ord_type: state.order_type() };
            Some(IssueOrder(new_order))
        }
        UpdateOrder(order) => {
            state.has_refreshed = true;
            state.status = format!("updated order {} of {} @ {}", order.cl_ord_id, order.qty, order.price);
            state.order = Some((*order).clone());
            None
        }
        CancelLast if state.order.is_some() => {
            let order = state.order.as_ref().unwrap();
            state.has_refreshed = true;
            state.status = format!("cancelling order: {}", order.cl_ord_id);
            Some(CancelOrder(&order.cl_ord_id ))
        }
        _ => None
    }
}

fn render_state(state: &State) -> String {
    let recent_order_if_present = match state.order {
        Some(ref o) => format!("\r\nCURR ORDER: {} {} {:.5} @ {:.5}", o.ord_type, o.ord_status, o.qty, o.price),
        None => "".to_string()
    };
    format!("{}\r\
\r
ASK: {:.5}  BID: {:.5}\r
QTY: {:.5}\r
ORDER TYPE: {}\r
STATUS: {}{}",
            USER_GUIDE, state.ask, state.bid, state.qty, state.order_type(), state.status, recent_order_if_present)
}