## RillRate

### Real-time UI for bots, microservices, and IoT

RillRate is a library that embeds a live web dashboard to your app.

Fast, embedded, with auto-layout and controls. No configuration is needed.
**Support**: [Rust][rillrate-rs], [Python][rillrate-py]. _Soon_: Node.js, Java, C#.

<img align="center" width="400px" src="https://rillrate.com/images/dashboard.png" />

- **_It's fully custom_** - You add your own data streams with everything you want
- **_It works in real-time!_** - NOT `5 secs` real-time, it's `0.002 secs` real-time 🚀
- **_Zero-configuration_** - you don't need to install and configure any other software
- **_Web-dashboard included_** - add the library to your app and connect to the dashboard with a web browser
- **_Ferris-friendly_** - we created it using Rust only: from backed to UI 🦀

Become a [sponsor][sponsor] to see how the project is born.  Sponsors also get
access to sources of the UI dashboard that made with the [Yew][yew] Framework.

Join our [reddit/rillrate][reddit] community to stay tuned about all the new features we released every day!

Follow us on [Twitter][twitter] and watch or participate weekly competitions.

### How to use it?

Add a dependency to your `Cargo.toml`:

```toml
[dependencies]
rillrate = "0.40.0-rc.1"
```

Install the **rillrate** engine in the `main` function:

```rust
rillrate::install("my-app");
```

And create a `Tracer` to visualize data on the embedded dashboard:

```rust
let my_tracer = Pulse::new(
    "package.dashboard.group.tracer-name",
    FlowMode::Realtime,
    PulseOpts::default().min(0).max(50).higher(true)
);
```

When you tracer is spawned use it to put data to it:

```rust
tracer.push(value);
```

### Packs

RillRate provides packs of components for different purposes.

Released:

- **Prime** - basic elements

In progress:

- **APM** - components for performance monitoring
- **Charts** - all basic charts
- **Trade** - live components for trading (order books, charts, etc.)

### Project structure

The project consists of the following parts:

- `pkg-core` (backend) - core components and the engine
- `pkg-dashboard` (frontend) - the dashboard app and rendering routines
- `pkg-packs` - tracers for different data stream types
- `rillrate` - the main library that joins all the parts above
- `demo` - the demo app

### Frameworks

We use the following frameworks to build our product:

- [Yew][yew] Framework (frontend)
- [meio][meio] actor framework (backend)

The original idea was inspired by [Nitrogen][nitrogen] Web Framework (Erlang).

<hr>

### License

**RillRate** is provided under the _Apache-2.0_ license. See [LICENSE](LICENSE).

The project is the Full-stack Rust app: both frontend and backend made with Rust.

[reddit]: https://reddit.com/r/rillrate/
[twitter]: https://twitter.com/rillrate/
[sponsor]: https://github.com/sponsors/rillrate
[nitrogen]: https://nitrogenproject.com/
[yew]: https://github.com/yewstack/yew
[meio]: https://github.com/rillrate/meio
[rillrate-rs]: https://github.com/rillrate/rillrate
[rillrate-py]: https://github.com/rillrate/rillrate-py
