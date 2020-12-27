mod actors;

use actors::embedded_node::EmbeddedNode;
use anyhow::Error;

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
