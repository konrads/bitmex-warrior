#[macro_use]
extern crate config;
#[macro_use]
extern crate enum_display_derive;
#[macro_use]
extern crate lazy_static;

use std::io::{stdin, stdout, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use log4rs;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use bitmex_warrior::{refresh_ui, show_cursor};
use model::*;
use model::{OrchestratorEvent::*, PriceType::*};
use orchestrator::process_event;
use render::render_state;
use ws::handle_msg;

mod orchestrator;
mod model;
mod render;
mod sign;
mod ws;
mod ws_model;
mod rest;
mod rest_model;

const USER_GUIDE: &str =
".-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-.\r
|                                       |\r
|             BITMEX WARRIOR            |\r
|                                       |\r
|  z -> buy @ bid      x -> sell @ ask  |\r
|  a -> buy @ ask      s -> sell @ bid  |\r
|  + -> up qty         - -> down qty    |\r
|  o -> rotate order types              |\r
|  c -> cancel last order               |\r
|  ctrl-c -> exit                       |\r
|                                       |\r
`-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-'\r
";

lazy_static! {
    static ref CFG: AppConfig = AppConfig::new("app").unwrap();
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
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!(r"
 ▄▄▄▄    ██▓▄▄▄█████▓ ███▄ ▄███▓▓█████ ▒██   ██▒       █     █░ ▄▄▄       ██▀███   ██▀███   ██▓ ▒█████   ██▀███         ▐██▌
▓█████▄ ▓██▒▓  ██▒ ▓▒▓██▒▀█▀ ██▒▓█   ▀ ▒▒ █ █ ▒░      ▓█░ █ ░█░▒████▄    ▓██ ▒ ██▒▓██ ▒ ██▒▓██▒▒██▒  ██▒▓██ ▒ ██▒       ▐██▌
▒██▒ ▄██▒██▒▒ ▓██░ ▒░▓██    ▓██░▒███   ░░  █   ░      ▒█░ █ ░█ ▒██  ▀█▄  ▓██ ░▄█ ▒▓██ ░▄█ ▒▒██▒▒██░  ██▒▓██ ░▄█ ▒       ▐██▌
▒██░█▀  ░██░░ ▓██▓ ░ ▒██    ▒██ ▒▓█  ▄  ░ █ █ ▒       ░█░ █ ░█ ░██▄▄▄▄██ ▒██▀▀█▄  ▒██▀▀█▄  ░██░▒██   ██░▒██▀▀█▄         ▓██▒
░▓█  ▀█▓░██░  ▒██▒ ░ ▒██▒   ░██▒░▒████▒▒██▒ ▒██▒      ░░██▒██▓  ▓█   ▓██▒░██▓ ▒██▒░██▓ ▒██▒░██░░ ████▓▒░░██▓ ▒██▒       ▒▄▄
░▒▓███▀▒░▓    ▒ ░░   ░ ▒░   ░  ░░░ ▒░ ░▒▒ ░ ░▓ ░      ░ ▓░▒ ▒   ▒▒   ▓▒█░░ ▒▓ ░▒▓░░ ▒▓ ░▒▓░░▓  ░ ▒░▒░▒░ ░ ▒▓ ░▒▓░       ░▀▀▒
▒░▒   ░  ▒ ░    ░    ░  ░      ░ ░ ░  ░░░   ░▒ ░        ▒ ░ ░    ▒   ▒▒ ░  ░▒ ░ ▒░  ░▒ ░ ▒░ ▒ ░  ░ ▒ ▒░   ░▒ ░ ▒░       ░  ░
 ░    ░  ▒ ░  ░      ░      ░      ░    ░    ░          ░   ░    ░   ▒     ░░   ░   ░░   ░  ▒ ░░ ░ ░ ▒    ░░   ░           ░
 ░       ░                  ░      ░  ░ ░    ░            ░          ░  ░   ░        ░      ░      ░ ░     ░            ░
      ░
    ");


    let (tx, rx) = mpsc::channel::<OrchestratorEvent>();
    let tx2 = tx.clone();
    let tx3 = tx.clone();
    let orchestrator_thread = thread::spawn(move || {
        let mut state = State::new(CFG.init_qty, CFG.qty_inc);
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, USER_GUIDE, termion::cursor::Hide).unwrap();
        loop {
            match rx.recv() {
                Ok(Exit) => {
                    println!();
                    show_cursor!(stdout);
                    break
                },
                Ok(e) => {
                    if let Some(cmd) = process_event(&e, &mut state) {
                        match cmd {
                            ExchangeCmd::CancelOrder(cl_ord_id) => {
                                rest::cancel_order(CFG.http_url.as_str(), CFG.api_key.as_str(), CFG.api_secret.as_str(), cl_ord_id, &mut tx3.clone());
                            }
                            ExchangeCmd::IssueOrder(order) => {
                                rest::issue_order(CFG.http_url.as_str(), CFG.api_key.as_str(), CFG.api_secret.as_str(), CFG.symbol.as_str(), &order, &mut tx3.clone());
                            }
                        }
                    }
                    if state.has_refreshed {
                        let render = render_state(USER_GUIDE, &state);
                        refresh_ui!(stdout, render);
                    }
                },
                Err(err) => {
                    log::error!("mpsc channel receive error: {:?}", err);
                    break
                },
            }
        }
    });

    let _ws_thread = thread::spawn(move || {
        handle_msg(
            CFG.wss_url.as_str(),
            CFG.api_key.as_str(),
            CFG.api_secret.as_str(),
            CFG.wss_subscriptions.clone(),
            &tx2);
    });

    let stdin = stdin();
    // http://ticki.github.io/blog/making-terminal-applications-in-rust-with-termion/
    for c in stdin.keys() {
        let key = c.unwrap();
        match key {
            Key::Char('+') | Key::Char('=') => { tx.send(UpQty).unwrap(); () },
            Key::Char('-') | Key::Char('_') => { tx.send(DownQty).unwrap(); () },
            Key::Char('o') => { tx.send(RotateOrderType).unwrap(); () },
            Key::Char('z') => { tx.send(Buy(Bid)).unwrap();  () },
            Key::Char('x') => { tx.send(Sell(Ask)).unwrap(); () },
            Key::Char('a') => { tx.send(Buy(Ask)).unwrap();  () },
            Key::Char('s') => { tx.send(Sell(Bid)).unwrap(); () },
            Key::Char('c') => { tx.send(CancelLast).unwrap(); () },
            Key::Ctrl('c') => {
                tx.send(Exit).unwrap();
                // ws_socket.close(None);
                break
            },
            _other => ()  // { write!(stdout(), "{}{}...{:?}{}", termion::cursor::Goto(1, 1), termion::clear::All, _other, termion::cursor::Hide).unwrap(); () },
        }
    }

    orchestrator_thread.join().unwrap();
}