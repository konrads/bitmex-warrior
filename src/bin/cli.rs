use std::env;

use bitmex_warrior::sign;

/// Playground for manual testing
fn main() {
    let args: Vec<String> = env::args().collect();

    let s = &args[1];
    let api_secret = &args[2];

    println!("signed s: {}, api_secret: {} -> {}", s, api_secret, sign::sign(s, api_secret));
}
