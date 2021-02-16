#[macro_use]
extern crate enum_display_derive;
use std::fmt::Display;

// use env_logger;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

#[derive(Debug, Display, PartialEq, Clone)]
enum OrderType {
    Limit,
    StopLimit
}

#[derive(Debug, Display, PartialEq, Clone)]
enum OrderStatus {
    NotYetIssued,
    New,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected
}

#[derive(Debug, Display, PartialEq, Clone)]
enum PriceType {
    Bid,
    Ask
}

#[derive(Debug, Display, PartialEq, Clone)]
enum Side {
    Buy,
    Sell
}

#[derive(Debug, PartialEq, Clone)]
struct ExchangeOrder {
    cl_ord_id: String,
    ord_status: OrderStatus,
    ord_type: OrderType,
    price: f64,
    qty: f64,
    side: Side,
}

#[derive(Debug, PartialEq)]
enum OrchestratorEvent {
    Buy(PriceType),  // from user
    Sell(PriceType), // from user
    CancelLast,      // from user
    UpQty,           // from user
    DownQty,         // from user
    NewBid(f64),     // from WS
    NewAsk(f64),     // from WS
    UpdateOrder(ExchangeOrder),  // from WS/Rest
    Exit  // from user
}

#[derive(Debug)]
enum ExchangeAction<'a> {
    IssueOrder(ExchangeOrder),
    CancelOrder(&'a str)
}

#[derive(Debug, PartialEq)]
struct State {
    bid: f64,
    ask: f64,
    qty: f64,
    qty_increment: f64,
    order: Option<ExchangeOrder>,
    status: String,
    has_refreshed: bool
}

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
        let mut state = State { bid: 0.0, ask: 0.0, order: None, qty: 10.0, qty_increment: 2.0, has_refreshed: false, status: "".to_string() };
        let mut stdout = stdout().into_raw_mode().unwrap();
        loop {
            match rx.recv() {
                Ok(e) if e == OrchestratorEvent::Exit => break,
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
    // let mut stdout = stdout().into_raw_mode().unwrap();
    // setting up stdout and going into raw mode
    // printing welcoming message, clearing the screen and going to left top corner with the cursor
    // write!(stdout, "{}{}ctrl-z: buy@bid, ctrl-x: sell@ask, ctrl-a: buy@ask, ctl-s: sell@bid", termion::cursor::Goto(1, 1), termion::clear::All).unwrap();

    let mut prev_key = Key::Ctrl('.');  // some random key...
    // http://ticki.github.io/blog/making-terminal-applications-in-rust-with-termion/
    // detecting keydown events, pass to Orchestrator
    for c in stdin.keys() {
        let key = c.unwrap();
        // write!(stdout(), "{}{}...{:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, key, termion::cursor::Hide).unwrap();
        match key {
            Key::Char('+') => { tx.send(OrchestratorEvent::UpQty).unwrap(); () },
            Key::Char('-') => { tx.send(OrchestratorEvent::DownQty).unwrap(); () },
            _ if key == prev_key => (),
            Key::Char('z') => { tx.send(OrchestratorEvent::Buy(PriceType::Bid)).unwrap();  () },
            Key::Char('x') => { tx.send(OrchestratorEvent::Sell(PriceType::Ask)).unwrap(); () },
            Key::Char('a') => { tx.send(OrchestratorEvent::Buy(PriceType::Ask)).unwrap();  () },
            Key::Char('s') => { tx.send(OrchestratorEvent::Sell(PriceType::Bid)).unwrap(); () },
            Key::Char('q') => { tx.send(OrchestratorEvent::CancelLast).unwrap(); () },
            Key::Ctrl('c') => { tx.send(OrchestratorEvent::Exit).unwrap(); break},
            other => () // write!(stdout, "{}{}...{:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, other, termion::cursor::Hide).unwrap(),
        }
        prev_key = key;
  }
  orchestrator_thread.join().unwrap();
}

fn process_event<'a>(event: &'a OrchestratorEvent, state: &'a mut State) -> Option<ExchangeAction<'a>> {  // probably need dyn...
    state.has_refreshed = false;
    match event {
        OrchestratorEvent::UpQty => {
            state.has_refreshed = true;
            state.qty += state.qty_increment;
            None
        }
        OrchestratorEvent::DownQty if state.qty <= state.qty_increment => None,  // ignore, cannot decrease < 0
        OrchestratorEvent::DownQty => {
            state.has_refreshed = true;
            state.qty -= state.qty_increment;
            None
        }
        OrchestratorEvent::NewBid(bid) if state.bid == *bid => None,
        OrchestratorEvent::NewBid(bid) if state.bid == *bid => None,
        OrchestratorEvent::NewBid(bid) => {
            state.has_refreshed = true;
            state.bid = *bid;
            None
        }
        OrchestratorEvent::NewAsk(ask) if state.ask == *ask => None,
        OrchestratorEvent::NewAsk(ask) => {
            state.has_refreshed = true;
            state.ask = *ask;
            None
        }
        OrchestratorEvent::Buy(price_type) => {
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                PriceType::Bid => state.bid,
                PriceType::Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("new buy order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: state.qty, price: price, side: Side::Buy, ord_type: OrderType::Limit };
            Some(ExchangeAction::IssueOrder(new_order))
        }
        OrchestratorEvent::Sell(price_type) => {
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                PriceType::Bid => state.bid,
                PriceType::Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("new sell order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: state.qty, price: price, side: Side::Sell, ord_type: OrderType::Limit };
            Some(ExchangeAction::IssueOrder(new_order))
        }
        OrchestratorEvent::UpdateOrder(order) => {
            state.has_refreshed = true;
            state.status = format!("updated order {} of {} @ {}", order.cl_ord_id, order.qty, order.price);
            state.order = Some((*order).clone());
            None
        }
        OrchestratorEvent::CancelLast if state.order.is_some() => {
            let order = state.order.as_ref().unwrap();
            state.has_refreshed = true;
            state.status = format!("cancelling order: {}", order.cl_ord_id);
            Some(ExchangeAction::CancelOrder(&order.cl_ord_id ))
        }
        _ => None
    }
}

fn render_state(state: &State) -> String {
    // write!(stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, e, termion::cursor::Hide).unwrap();
    let recent_order_if_present = match state.order {
        Some(ref o) => format!("\r\nCURR ORDER: {} {} {:.5} @ {:.5}", o.ord_type, o.ord_status, o.qty, o.price),
        None => "".to_string()
    };
    format!("\r
.-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-.\r
|              BITMEX WARRIOR           |\r
|                                       |\r
|  z -> buy @ bid      x -> sell @ ask  |\r
|  a -> buy @ ask      s -> sell @ bid  |\r
|  + -> up qty         - -> down qty    |\r
|  q -> cancel last order               |\r
|  ctrl-c -> exit                       |\r
`-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-'\r
\r
ASK: {:.5}  BID: {:.5}\r
QTY: {:.5}\r
STATUS: {}{}",
            state.ask, state.bid, state.qty, state.status, recent_order_if_present)
}