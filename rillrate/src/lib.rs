pub mod actors;

use actors::supervisor::NodeSupervisor;
use anyhow::Error;
use meio::thread;

pub struct RillRateHandle {
    rt: thread::ScopedRuntime,
}

pub fn start_engine() -> Result<RillRateHandle, Error> {
    let actor = NodeSupervisor::new(Default::default());
    let rt = thread::spawn(actor)?;
    Ok(RillRateHandle { rt })
}
