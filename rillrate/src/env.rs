use std::env::{current_exe, var};
use std::path::PathBuf;

// TODO: Refactor this module... Use proper error types with logging.

pub fn config() -> PathBuf {
    var("RILLRATE_CONFIG")
        .unwrap_or_else(|_| "rillrate.toml".into())
        .into()
}

pub fn name() -> Option<String> {
    var("RILLRATE_NAME").ok().or_else(|| {
        current_exe().ok().and_then(|buf| {
            buf.as_path()
                .file_name()
                .and_then(|path| path.to_str().map(String::from))
        })
    })
}

pub fn meta() -> bool {
    var("RILLRATE_META")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(false)
}

pub fn node() -> Option<String> {
    var("RILLRATE_NODE").ok()
}
