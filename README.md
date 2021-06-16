# rillrate

Dynamic tracing system that tends to be real-time.

<img src="https://ui.rillrate.com/images/dashboard.png" width="400px">

`rillrate` is a **real, full-stack Rust app** from the backend to the web dashboard.

## How to use

The [documentation](https://rillrate.com/docs/basics/intro/) has Rust,
Python and Node.js guides for embeding a rillrate service in an
application. For exploring rillrate itself, there is a standalone
server.

### Native extension

This library is implemented with Rust and available for other languages as native extensions.
It is implemented this way to deliver the fastest possible performance and to try not to affect
the performance of the app.

### Supported languages

This crate is the implementation of the `rillrate` tracing system. It can be used
in Rust directly or in other languages (Python, Node.js) through thin bindings.

Ready to use:

- Rust
- [Python](https://github.com/rillrate/rillrate-py)
- [Node.js](https://github.com/rillrate/rillrate-js)

In progress:

- Java (JVM)
- C# (.NET)
- C/C++
- Go (based on C binding)

Scheduled:

- JavaScript

## Dashboard

`rillrate` ships with an embedded node that can be used to check any metrics in-place.
Just install the library and open `http://localhost:9090/` in your browser.

You will get access to the `RillRate View` dashboard that can activate
any available stream of data in your app and visualize it.

## Exporters

`rillrate` can export metrics to third-party services by activating them in a config file.

The list of supported systems:

- Prometheus
- Graphite

### Prometheus

Why did we use `9090` port for the inline server? You guessed it: it works as a
`Prometheus` endpoint that can be activated with a configuration file. Just add
a file called `rillrate.toml` to the working directory that contains the following:

```toml
[server]
address = "0.0.0.0"

[export.prometheus]
paths = [
    "my.counter.one",
    "my.gauge.two",
]
```

and access metrics from `http://<server-ip>:9090/metrics`.
