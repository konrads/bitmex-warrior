#[macro_use]
extern crate enum_display_derive;

pub mod model;
pub mod orchestrator;
pub mod render;
pub mod sign;
pub mod ws;
pub mod ws_model;
pub mod rest;
pub mod rest_model;


#[macro_export]
macro_rules! refresh_ui {
    ($stdout:expr, $x:expr) => {
        {
            write!($stdout, "{}{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, $x, termion::cursor::Hide).unwrap();
            $stdout.flush().unwrap();
        }
    }
}

#[macro_export]
macro_rules! show_cursor {
    ($stdout:expr) => {
        {
            write!($stdout, "{}", termion::cursor::Hide).unwrap();
            $stdout.flush().unwrap();
        }
    }
}