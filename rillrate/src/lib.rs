//! `RillRate` - Dynamic UI for bots, microservices, and IoT.

#![warn(missing_docs)]

mod actors;

metacrate::meta!();

// TODO: Use packs separately (later).
pub use rrpack_prime::live_control::*;
pub use rrpack_prime::live_flow::*;
pub use rrpack_prime::range;
pub use rrpack_prime::Link;

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
pub fn install(_name: impl ToString) -> Result<(), Error> {
    let actor = NodeSupervisor::new(Default::default());
    let rt = thread::spawn(actor)?;
    let mut opt_handle = GLOBAL.lock().map_err(|err| Error::msg(err.to_string()))?;
    *opt_handle = Some(RillRateHandle { _rt: rt });
    Ok(())
}

/// Stops the engine.
pub fn uninstall() -> Result<(), Error> {
    let mut opt_handle = GLOBAL.lock().map_err(|err| Error::msg(err.to_string()))?;
    opt_handle.take();
    Ok(())
}
