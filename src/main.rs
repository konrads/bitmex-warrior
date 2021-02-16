#[macro_use]
extern crate enum_display_derive;

// use env_logger;
mod behaviour;
mod model;
mod render;

use model::*;
use model::{OrchestratorEvent::*, PriceType::*};
use render::render_state;
use behaviour::process_event;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::sync::mpsc;
use std::thread;

const USER_GUIDE: &str =
r".-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-.
|                                       |
|              BITMEX WARRIOR           |
|                                       |
|  z -> buy @ bid      x -> sell @ ask  |
|  a -> buy @ ask      s -> sell @ bid  |
|  + -> up qty         - -> down qty    |
|  o -> rotate order types              |
|  q -> cancel last order               |
|  ctrl-c -> exit                       |
|                                       |
`-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-'
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