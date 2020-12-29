mod actors;

use actors::embedded_node::EmbeddedNode;
use anyhow::Error;

metacrate::meta!();

mod env {
    use std::env::var;

    pub fn config() -> String {
        var("RILL_CONFIG").unwrap_or_else(|_| "rill.toml".into())
    }

    pub fn ui() -> String {
        var("RILL_UI").unwrap_or_else(|_| ".".into())
    }
}

pub struct RillExport {
    _scoped_to_drop: meio::thread::ScopedRuntime,
}

impl RillExport {
    pub fn start() -> Result<Self, Error> {
        rill::PORT.set(9090);
        let actor = EmbeddedNode::new();
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self {
            _scoped_to_drop: scoped,
        })
    }
}
