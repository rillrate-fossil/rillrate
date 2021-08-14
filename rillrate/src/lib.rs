//! `RillRate` - Dynamic UI for bots, microservices, and IoT.

#![warn(missing_docs)]

mod actors;

metacrate::meta!();

pub use rillrate_protocol::live_control::*;
pub use rillrate_protocol::live_flow::*;
pub use rillrate_protocol::range;
pub use rillrate_protocol::Link;

use actors::supervisor::NodeSupervisor;
use anyhow::Error;
use meio::thread;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static GLOBAL: Lazy<Mutex<Option<RillRateHandle>>> = Lazy::new(|| Mutex::new(None));

/// Tracks a lifetime of the `RillRate` engine.
pub struct RillRateHandle {
    _rt: thread::ScopedRuntime,
}

/// Starts the engine.
pub fn start() -> Result<(), Error> {
    let actor = NodeSupervisor::new(Default::default());
    let rt = thread::spawn(actor)?;
    let mut opt_handle = GLOBAL.lock().map_err(|err| Error::msg(err.to_string()))?;
    *opt_handle = Some(RillRateHandle { _rt: rt });
    Ok(())
}

/// Stops the engine.
pub fn stop() -> Result<(), Error> {
    let mut opt_handle = GLOBAL.lock().map_err(|err| Error::msg(err.to_string()))?;
    opt_handle.take();
    Ok(())
}
