use std::env::{current_exe, var};
//use std::net::{SocketAddr, Ipv4Addr};
use std::path::PathBuf;

// TODO: Refactor this module... Use proper error types with logging.

pub fn config() -> PathBuf {
    var("RILL_CONFIG")
        .unwrap_or_else(|_| "rillrate.toml".into())
        .into()
}

pub fn name() -> Option<String> {
    var("RILL_NAME").ok().or_else(|| {
        current_exe().ok().and_then(|buf| {
            buf.as_path()
                .file_name()
                .and_then(|path| path.to_str().map(String::from))
        })
    })
}

/* TODO: Use this
pub fn node() -> SocketAddr {
    var("RILL_NODE").ok().and_then(|s| {
        s.parse::<SocketAddr>()
            .map_err(|err| {
                // TODO: Return error here
                log::error!("Can't parse address: {}", err);
            })
            .ok()
    })
    .unwrap_or_else(|| SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), rill_protocol::PORT.get()))
}
*/

pub fn standalone() -> bool {
    var("RILL_EXPORT").is_ok()
}
