#[macro_use]
extern crate enum_display_derive;
#[macro_use]
extern crate lazy_static;

use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::thread;

use log4rs;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use bitmex_warrior::{refresh_ui, show_cursor};
use model::*;
use model::{OrchestratorEvent::*, PriceType::*};

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
#[allow(unused_must_use)]
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
        refresh_ui!(stdout, USER_GUIDE);
        loop {
            match rx.recv() {
                Ok(Exit) => {
                    println!();
                    show_cursor!(stdout);
                    break
                },
                Ok(e) => {
                    if let Some(cmd) = orchestrator::process_event(&e, &mut state) {
                        let rest_resp = match cmd {
                            ExchangeCmd::CancelOrder(cl_ord_id) =>
                                rest::cancel_order(&CFG.http_url, &CFG.api_key, &CFG.api_secret, cl_ord_id),
                            ExchangeCmd::IssueOrder(order) =>
                                rest::issue_order(&CFG.http_url, &CFG.api_key, &CFG.api_secret, &CFG.symbol.as_str(), &order)
                        };
                        rest_resp.map(|x| tx3.send(x).expect("Failed to send event"));  // could also add .ok() to supress warnings, as per https://stackoverflow.com/questions/53368303/why-am-i-getting-unused-result-which-must-be-used-result-may-be-an-err-vari
                    };
                    if state.has_refreshed {
                        let rendered = render::render_state(USER_GUIDE, &state);
                        refresh_ui!(stdout, rendered);
                    }
                },
                Err(err) => {
                    log::error!("mpsc channel receive error: {:?}", err);
                    break
                }
            }
        }
    });

    let _ws_thread = thread::spawn(move || {
        ws::handle_msgs(&CFG.wss_url, &CFG.api_key, &CFG.api_secret, CFG.wss_subscriptions.clone(), &tx2);
    });

    let stdin = stdin();
    // http://ticki.github.io/blog/making-terminal-applications-in-rust-with-termion/
    for c in stdin.keys() {
        let key = c.unwrap();
        match key {
            Key::Char('+') | Key::Char('=') => tx.send(UpQty).unwrap(),
            Key::Char('-') | Key::Char('_') => tx.send(DownQty).unwrap(),
            Key::Char('o') => tx.send(RotateOrderType).unwrap(),
            Key::Char('z') => tx.send(Buy(Bid)).unwrap(),
            Key::Char('x') => tx.send(Sell(Ask)).unwrap(),
            Key::Char('a') => tx.send(Buy(Ask)).unwrap(),
            Key::Char('s') => tx.send(Sell(Bid)).unwrap(),
            Key::Char('c') => tx.send(CancelLast).unwrap(),
            Key::Ctrl('c') => {
                tx.send(Exit).unwrap();
                break
            },
            _other => ()
        }
    }

    orchestrator_thread.join().unwrap();
}