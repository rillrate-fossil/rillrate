# How to build?

## Linux

No issues. Just compile it with Rust compiler.

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
