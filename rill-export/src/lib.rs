//! RillExport crate.

#![warn(missing_docs)]

mod actors;
mod config;

use actors::embedded_node::EmbeddedNode;
use anyhow::Error;

metacrate::meta!();

// TODO: Remove env vars here
mod env {
    use std::env::var;

    pub fn config() -> String {
        var("RILL_CONFIG").unwrap_or_else(|_| "rill.toml".into())
    }

    pub fn ui() -> String {
        var("RILL_UI").unwrap_or_else(|_| ".".into())
    }
}

/// The standalone server that provides access to metrics in different ways.
pub struct RillExport {
    _scoped_to_drop: meio::thread::ScopedRuntime,
}

impl RillExport {
    /// Starts an exporting server.
    pub fn start() -> Result<Self, Error> {
        //rill_protocol::PORT.set(9090);
        let actor = EmbeddedNode::new();
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self {
            _scoped_to_drop: scoped,
        })
    }
}
