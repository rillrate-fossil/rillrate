//! Rill crate.

#![warn(missing_docs)]

mod actors;
mod config;
pub mod macros;
pub mod prelude;
mod protocol;
mod state;
pub mod tracers;

use crate::actors::supervisor::RillSupervisor;
use anyhow::Error;
use config::RillConfig;
use rill_protocol::provider::EntryId;
use state::{RillState, RILL_STATE};
use thiserror::Error;

metacrate::meta!();

#[derive(Debug, Error)]
enum RillError {
    /*
    #[error("not installed")]
    NotInstalled,
    */
    #[error("alreary installed")]
    AlreadyInstalled,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
}

/// The provider instance that can be configured.
#[derive(Debug)]
pub struct Rill {
    _scoped: meio::thread::ScopedRuntime,
}

impl Rill {
    /// Initializes provider system and all created `Tracer`s will be attached to it.
    pub fn install(host: String, name: impl Into<EntryId>, with_meta: bool) -> Result<Self, Error> {
        let (rx, state) = RillState::create();
        // IMPORTANT! Set the state before any worker/supervisor will be spawned,
        // because `meta` tracers also uses the same state for registering themselves.
        RILL_STATE
            .set(state)
            .map_err(|_| RillError::AlreadyInstalled)?;
        let config = RillConfig::new(host, name.into(), with_meta);
        let actor = RillSupervisor::new(config, rx);
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self { _scoped: scoped })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install() -> Result<(), Error> {
        let _rill = Rill::install("127.0.0.1:1636".into(), "rill");
        let counter = tracers::CounterTracer::new("counter".parse()?);
        counter.inc(1.0, None);
        Ok(())
    }

    #[test]
    fn test_provider_without_tracer() -> Result<(), Error> {
        // It must not panic.
        let counter = tracers::CounterTracer::new("counter".parse()?);
        for _ in 0..1_000_000 {
            counter.inc(1.0, None);
        }
        Ok(())
    }
}
