# Release sequence

- rill-protocol
- rill-engine
- rill-client
- rill-export
- rill-server
- rillrate

# How to build?

## Linux

```bash
git clone https://github.com/rillrate/rillrate.git
cd rillrate
cargo build
```

## Mac

Install dev tools:

```bash
xcode-select --install
```

Add to your `~/.cargo/config`:

```toml
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]

[target.aarch64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```

# Windows

It should work.
