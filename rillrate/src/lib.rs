pub mod actors;

pub use rillrate_protocol::live_data::counter::*;

use actors::supervisor::NodeSupervisor;
use anyhow::Error;
use meio::thread;

pub struct RillRateHandle {
    rt: thread::ScopedRuntime,
}

pub fn start() -> Result<RillRateHandle, Error> {
    let actor = NodeSupervisor::new(Default::default());
    let rt = thread::spawn(actor)?;
    Ok(RillRateHandle { rt })
}
