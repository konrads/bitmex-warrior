#[macro_use]
extern crate enum_display_derive;

// use env_logger;
mod model;
mod behaviour;

use model::*;
use model::{OrchestratorEvent::*, PriceType::*, ExchangeAction::*};
use behaviour::process_event;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::sync::mpsc;
use std::thread;

const USER_GUIDE: &str = "\
.-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-.\r
|                                       |\r
|              BITMEX WARRIOR           |\r
|                                       |\r
|  z -> buy @ bid      x -> sell @ ask  |\r
|  a -> buy @ ask      s -> sell @ bid  |\r
|  + -> up qty         - -> down qty    |\r
|  o -> rotate order types              |\r
|  q -> cancel last order               |\r
|  ctrl-c -> exit                       |\r
|                                       |\r
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
        write!(stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, USER_GUIDE, termion::cursor::Hide).unwrap();
        loop {
            match rx.recv() {
                Ok(e) if e == Exit => {
                    write!(stdout, "{}", termion::cursor::Show).unwrap();
                    break
                },
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

fn render_state(state: &State) -> String {
    let recent_order_if_present = match state.order {
        Some(ref o) => format!("\r\nCURR ORDER: {} {} {:.5} @ {:.5}", o.ord_type, o.ord_status, o.qty, o.price),
        None => "".to_string()
    };
    format!("{}\r\
\r
BID: {:.5} / ASK: {:.5}\r
QTY: {:.5}\r
ORDER TYPE: {}\r
STATUS: {}{}",
            USER_GUIDE, state.bid, state.ask, state.qty, state.order_type(), state.status, recent_order_if_present)
}