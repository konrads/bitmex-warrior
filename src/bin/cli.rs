use clap::{App, AppSettings, Arg};
use bitmex_warrior::sign;

/// Playground for manual testing
fn main() {
    let app = App::new("cli")
        .version("0.0.1")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("sign")
                .arg(Arg::new("api-secret").short('a').long("api-secret").takes_value(true).required(true))
                .arg(Arg::new("to-be-signed").index(1).required(true)),
        )
        .subcommand(
            App::new("threads")
        );
    let arg_matches = app.get_matches();

    match arg_matches.subcommand() {
        Some(("sign", sign_matches)) => {
            let api_secret = sign_matches.value_of("api-secret").unwrap();
            let to_be_signed = sign_matches.value_of("to-be-signed").unwrap();
            println!("signed to-be-signed: {}, api_secrets: {} -> {}", to_be_signed, api_secret, sign::sign(&to_be_signed, &api_secret));
        }
        Some(("threads", _threads_matches)) => {
            println!("...threads")
        }
        _ => unreachable!()  // thanks to AppSettings::SubcommandRequiredElseHelp
    };
}
