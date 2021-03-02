Want to trade Bitmex like a Warrior?
====================================

Build status (master): [![Build Status](https://travis-ci.org/konrads/bitmex-warrior.svg?branch=master)](https://travis-ci.org/konrads/bitmex-warrior)


![warrior_on_the_moon](doc/image/warrior_on_the_moon.jpg?raw=true)

Get yourself some keyboard shortcuts!

Disclaimer: This is a Rust playground project, I know there are alternatives eg. Tampermonkey.

TO RUN MAIN
-----------
```
cargo run
```

TO RUN CLI
----------
(for manual testing)
```
cargo run --bin cli -- --api-secret xx yy
```

TO RUN RELEASES
---------------
```
cargo build && cargo build --release
target/debug/cli
target/debug/main
target/release/cli
target/release/main
```

TO TEST
-------
```
cargo test && cargo test --doc
```