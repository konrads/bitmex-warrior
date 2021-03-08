Want to trade Bitmex like a Warrior?
====================================

Build status (master): [![Build Status](https://travis-ci.org/konrads/bitmex-warrior.svg?branch=master)](https://travis-ci.org/konrads/bitmex-warrior)


![warrior_on_the_moon](doc/image/warrior_on_the_moon.jpg?raw=true)

Get yourself some keyboard shortcuts!

Disclaimer: This is a Rust playground project, I know there are alternatives eg. Tampermonkey.

Run via cargo
-------------
```
cp app.toml.template app.toml              # replace <API_KEY> and <API_SECRET> with whose supplied via https://www.bitmex.com/app/apiKeys
cargo run                                  # run main
cargo run --bin cli -- --api-secret xx yy  # run munal testing via cli
```

Build and run deliverables 
--------------------------
```
cargo build && cargo build --release
target/debug/main
target/debug/cli --api-secret xx yy
target/release/main
target/release/cli --api-secret xx yy
```

Run tests
---------
```
cargo test && cargo test --doc
```