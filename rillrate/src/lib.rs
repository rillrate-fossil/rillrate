//! `RillRate` - Dynamic UI for bots, microservices, and IoT.

#![warn(missing_docs)]

mod actors;

metacrate::meta!();

// TODO: Use packs separately (later).
pub use rrpack_prime::live_control::*;
pub use rrpack_prime::live_flow::*;
pub use rrpack_prime::range;

use actors::supervisor::NodeSupervisor;
use anyhow::Error;
use meio::thread;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static GLOBAL: Lazy<Mutex<Option<RillRate>>> = Lazy::new(|| Mutex::new(None));

/// Tracks a lifetime of the `RillRate` engine.
pub struct RillRate {
    _rt: thread::ScopedRuntime,
}

impl RillRate {
    /// Starts the engine.
    pub fn start(_name: impl ToString) -> Result<Self, Error> {
        let actor = NodeSupervisor::new(Default::default());
        let rt = thread::spawn(actor)?;
        Ok(RillRate { _rt: rt })
    }

    /// Pin the engine globally. Not needed to keep the handle in the scope.
    pub fn pin(self) -> Result<(), Error> {
        let mut opt_handle = GLOBAL.lock().map_err(|err| Error::msg(err.to_string()))?;
        *opt_handle = Some(self);
        Ok(())
    }

    /// Installs the engine globally.
    pub fn install(name: impl ToString) -> Result<(), Error> {
        Self::start(name)?.pin()
    }

    /// Uninstall the engine from the global scope.
    pub fn uninstall() -> Result<(), Error> {
        let mut opt_handle = GLOBAL.lock().map_err(|err| Error::msg(err.to_string()))?;
        opt_handle.take();
        Ok(())
    }
}

/// Install the engine.
pub fn install(name: impl ToString) -> Result<(), Error> {
    RillRate::install(name)
}

/// Uninstall the engine.
pub fn uninstall() -> Result<(), Error> {
    RillRate::uninstall()
}
