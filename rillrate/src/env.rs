use std::env::var;
use std::path::PathBuf;

pub fn config() -> Option<PathBuf> {
    var("RILLRATE_CONFIG").map(PathBuf::from).ok()
}
