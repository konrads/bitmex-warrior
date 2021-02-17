#[macro_use]
extern crate enum_display_derive;

// use env_logger;
mod behaviour;
mod model;
mod render;
mod ws_handler;
mod ws_model;

use model::*;
use model::{OrchestratorEvent::*, PriceType::*};
use ws_model::*;
use ws_handler::Client;
use render::render_state;
use behaviour::process_event;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::sync::mpsc;
use std::thread;

const BITMEX_ADDR: &str = "wss://www.bitmex.com/realtime";

const USER_GUIDE: &str =
".-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-.\r
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
    env_logger::init();

    let (tx, rx) = mpsc::channel::<OrchestratorEvent>();
    let tx2 = tx.clone();
    let orchestrator_thread = thread::spawn(move || {
        let mut state = State::new(10.0, 2.0);
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, USER_GUIDE, termion::cursor::Hide).unwrap();
        loop {
            match rx.recv() {
                Ok(e) if e == Exit => {
                    write!(stdout, "\r\n{}", termion::cursor::Show).unwrap();
                    break
                },
                Ok(e) => {
                    if let Some(order) = process_event(&e, &mut state) {
                        // send out the order
                        write!(stdout, "{}{}effect: {:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, order, termion::cursor::Hide).unwrap();
                        stdout.flush().unwrap();
                    }
                    if state.has_refreshed {
                        let render = render_state(USER_GUIDE, &state);
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

    let ws_thread = thread::spawn(move || {
        if let Err(error) = ws::connect(BITMEX_ADDR, |out| Client::new(out, &tx2)) {
            log::error!("Failed to create WebSocket due to: {:?}", error)
        }
    });

    let stdin = stdin();
    let mut prev_key = Key::Ctrl('.');  // some random key...
    // http://ticki.github.io/blog/making-terminal-applications-in-rust-with-termion/
    for c in stdin.keys() {
        let key = c.unwrap();
        match key {
            Key::Char('+') | Key::Char('=') => { tx.send(UpQty).unwrap(); () },
            Key::Char('-') | Key::Char('_') => { tx.send(DownQty).unwrap(); () },
            Key::Char('o') => { tx.send(RotateOrderType).unwrap(); () },
            _ if key == prev_key => (),
            Key::Char('z') => { tx.send(Buy(Bid)).unwrap();  () },
            Key::Char('x') => { tx.send(Sell(Ask)).unwrap(); () },
            Key::Char('a') => { tx.send(Buy(Ask)).unwrap();  () },
            Key::Char('s') => { tx.send(Sell(Bid)).unwrap(); () },
            Key::Char('q') => { tx.send(CancelLast).unwrap(); () },
            Key::Ctrl('c') => { tx.send(Exit).unwrap(); break},
            _other => ()  // { write!(stdout(), "{}{}...{:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, _other, termion::cursor::Hide).unwrap(); () },
        }
        prev_key = key;
    }

    orchestrator_thread.join().unwrap();
    ws_thread.join().unwrap();
}