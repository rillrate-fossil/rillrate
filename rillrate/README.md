# rillrate

[![Crates.io][crates-badge]][crates-url]
[![Released API docs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/rillrate.svg
[crates-url]: https://crates.io/crates/rillrate
[docs-badge]: https://docs.rs/rillrate/badge.svg
[docs-url]: https://docs.rs/rillrate

Dynamic tracing system that tends to be real-time.

The original Rust version of the library.

# How to use

Add it you your `Cargo.toml`:

```
cargo add -s rillrate
```

Create an instance of `RillRate` tracer and `install` it providing
the name of your app:

```
use anyhow::Error;
use rillrate::RillRate;

fn main() -> Result<(), Error> {
    let _rillrate = RillRate::install("my-app");
    // Start your app routines here
    Ok(())
}
```

Add to your functions and modules all necessary metrics:

```
use rillrate::{Counter, Gauge, Logger};

fn my_routine() -> Result<(), Error> {
    let couter = Counter::new("my-counter");
    let gauge = Gauge::new("my-gauge");
    let logger = Logger::new("my-logger");

    // Usage in your code
    counter.inc(1.0);
    gauge.set(123.0);
    logger.log("my event");
}
```

Start your app and try to connect to `http://localhost:9090`.
