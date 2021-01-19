# rillrate

Dynamic tracing system that tends to be real-time.

# How to use

Any supported language has a special description how to use the library.

# Why it's native module

This library implemented with Rust and available for other languages as native extension.
The main reason why it's not implemented as pure modules to give you as much as possible
speed of delivery of any metric and try not to affect to the performance of your app.

# Supported languages

This crate is the implementation of `rillrate` tracing system that can be used
in Rust directly or in other languages (Python, Node.js) throung special thin bindings.

All the bindings/libs will be available for languages:

Ready to use:

- Rust
- Python
- Node.js

In progress:

- Java (JVM)
- C# (.NET)
- C/C++
- Go (based on C binding)

Scheduled:

- JavaScript

# Dashboard

It shipped with embedded node that can be used to check any metrics in-place.
Just install the library and open `http://localhost:9090/` in your browser.

You will get access to the fast UI: `RillRate View` dashboard that can activate
any available stream of data in your app and visualize it.

# Prometheus

Why we used `9090` port for standalone server? You guess correct: it contains
`Prometheus` endpoint that can be activated with a configuration file. Just add
to the workdir the file `rillrate.toml` that contains:

```toml
[server]
address = "0.0.0.0"

[export.prometheus]
paths = [
    "my.counter.one",
    "my.gauge.two",
]
```

And you can get metrics from `http://<server-ip>:9090/metrics`.
