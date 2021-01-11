//! RillExport crate.

#![warn(missing_docs)]

mod actors;
mod config;

use actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use std::path::PathBuf;

metacrate::meta!();

/// The standalone server that provides access to metrics in different ways.
pub struct RillExport {
    _scoped_to_drop: meio::thread::ScopedRuntime,
}

impl RillExport {
    /// Starts an exporting server.
    pub fn start(config: Option<PathBuf>) -> Result<Self, Error> {
        let actor = EmbeddedNode::new(config);
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self {
            _scoped_to_drop: scoped,
        })
    }
}
