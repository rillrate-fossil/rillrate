pub mod actors;

metacrate::meta!();

pub use rillrate_protocol::link::Link;
pub use rillrate_protocol::live_control::*;
pub use rillrate_protocol::live_flow::*;

// TODO: Add wrappers for tracers?

use actors::supervisor::NodeSupervisor;
use anyhow::Error;
use meio::thread;

pub struct RillRateHandle {
    _rt: thread::ScopedRuntime,
}

pub fn start() -> Result<RillRateHandle, Error> {
    let actor = NodeSupervisor::new(Default::default());
    let rt = thread::spawn(actor)?;
    Ok(RillRateHandle { _rt: rt })
}
