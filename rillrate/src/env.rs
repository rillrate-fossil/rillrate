use std::env::{current_exe, var};
use std::path::PathBuf;

pub fn config() -> PathBuf {
    var("RILL_CONFIG")
        .unwrap_or_else(|_| "rillrate.toml".into())
        .into()
}

pub fn name() -> Option<String> {
    var("RILL_AS").ok().or_else(|| {
        current_exe().ok().and_then(|buf| {
            buf.as_path()
                .file_name()
                .and_then(|path| path.to_str().map(String::from))
        })
    })
}
