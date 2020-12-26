mod actors;
mod exporters;

use actors::embedded_node::EmbeddedNode;
use anyhow::Error;

pub struct RillExport {
    scoped: meio::thread::ScopedRuntime,
}

impl RillExport {
    pub fn start() -> Result<Self, Error> {
        rill::PORT.set(9090);
        let actor = EmbeddedNode::new();
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self { scoped })
    }
}
