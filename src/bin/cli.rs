use getopts::Options;
use std::env;

use bitmex_warrior::sign;

/// Playground for manual testing
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.reqopt("a", "api-secret", "api-secret", "CREDENTIALS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    let api_secrets = matches.opt_str("api-secret").unwrap();
    let s = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    };
    println!("signed s: {}, api_secrets: {} -> {}", s, &api_secrets, sign::sign(&s, &api_secrets));
}
