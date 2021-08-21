## RillRate

### UI for Microservices, Bots and IoT devices.

**Support**: Rust, Python, Node.js _(soon)_.

<img align="center" width="400px" style="margin-left: 20px;" src="https://rillrate.com/images/dashboard.png" />

- **_It's fully custom_** - You add your own data streams with everything you want
- **_It works in real-time!_** - NOT `5 secs` real-time, it's `0.002 secs` real-time 🚀
- **_Zero-configuration_** - you don't need to install and configure any other software
- **_Web-dashboard included_** - add the library to your app and connect to the dashboard with a web browser
- **_Ferris-friendly_** - we created it using Rust only: from backed to UI 🦀

Become a [sponsor][sponsor] to see how the project is born.

Join our [reddit/rillrate][reddit] community to stay tuned about all the new features we released every day!

### How to use it?

Add a dependency to your `Cargo.toml`:

```toml
[dependencies]
rillrate = "0.37.0"
```

Install the **rillrate** engine in the `main` function:

```rust
rillrate::install("my-app");
```

And create a `Tracer` to visualize data on the embedded dashboard:

```rust
let my_tracer = Pulse::new(
    ["package", "dashboard", "group", "tracer-name"],
    None, // You can add a specification here: depth, ranges, labels, etc.
);
```

When you tracer is spawned use it to put data to it:

```rust
tracer.push(value);
```

<a href="https://github.com/sponsors/rillrate" target="_blank"><img align="right" width="300px" src="https://rillrate.com/images/book/book-only.gif" /></a>

### Do you want to know how we develop it?

Become a [sponsor][sponsor] to see how our company works inside.

**RillRate** has an open-source core that means we released the primary part of
our code under an open-source license.

We use the following frameworks to build our product:

- [Yew][yew] Framework (frontend)
- [meio][meio] actor framework (backend)

[reddit]: https://www.reddit.com/r/rillrate/
[sponsor]: https://github.com/sponsors/rillrate
[nitrogen]: https://nitrogenproject.com/
[yew]: https://github.com/yewstack/yew
[meio]: https://github.com/rillrate/meio
