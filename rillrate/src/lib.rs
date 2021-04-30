//! Dynamic tracing system that tends to be real-time.

#![warn(missing_docs)]

mod config;
mod env;
mod supervisor;
mod tracers;

pub use rill_engine as engine;
pub use rill_protocol as protocol;
pub use tracers::*;

use anyhow::Error;
use engine::tracers::meta::ReadyBoardTracer;
use meio::thread::ScopedRuntime;
use once_cell::sync::Lazy;
use protocol::flow::meta::ready_board::READY_BOARDS;
use protocol::io::provider::Path;
use std::sync::Mutex;

static INSTALLED: Lazy<Mutex<Option<RillRate>>> = Lazy::new(|| Mutex::new(None));

/// The tracer.
pub struct RillRate {
    _scoped: ScopedRuntime,
    ready_board_flow: ReadyBoardTracer,
}

impl RillRate {
    /// Creates an instance of `RillRate` tracer using environment vars.
    pub fn from_env(app_name: impl ToString) -> Result<Self, Error> {
        use supervisor::RillRate;
        let actor = RillRate::new(app_name.to_string());
        let _scoped = meio::thread::spawn(actor)?;
        let path = READY_BOARDS.root();
        let ready_board_flow = ReadyBoardTracer::new(path);
        Ok(Self {
            _scoped,
            ready_board_flow,
        })
    }

    /// Adds ReadyBoard to the provider.
    pub fn add_board(&mut self, name: String, paths: Vec<Path>, description: Option<String>) {
        self.ready_board_flow
            .add_board(name, paths.into_iter().collect(), description);
    }
}

/// Create and install an instance of `RillRate` into the global cell.
/// The provider will be alive and available until the `uninstall` method call.
pub fn install(app_name: impl ToString) -> Result<(), Error> {
    let instance = RillRate::from_env(app_name)?;
    // TODO: Metod like `Option::swap` instad?
    let mut cell = INSTALLED
        .lock()
        .map_err(|err| Error::msg(err.to_string()))?;
    *cell = Some(instance);
    Ok(())
}

/// Uninstalling of the installed `RillRate` instance.
pub fn uninstall() -> Result<(), Error> {
    INSTALLED
        .lock()
        .map_err(|err| Error::msg(err.to_string()))?
        .take();
    Ok(())
}
