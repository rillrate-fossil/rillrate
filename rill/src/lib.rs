//! Rill crate.

#![warn(missing_docs)]

mod actors;
mod config;
pub mod macros;
pub mod prelude;
mod protocol;
pub mod tracers;

use crate::actors::supervisor::RillSupervisor;
use crate::actors::worker::RillLink;
use anyhow::Error;
use config::RillConfig;
use once_cell::sync::OnceCell;
use rill_protocol::provider::EntryId;
use std::time::{Duration, Instant};
use thiserror::Error;

metacrate::meta!();

/// It used by tracers to register them into the state.
static RILL_LINK: OnceCell<RillLink> = OnceCell::new();

#[derive(Debug, Error)]
enum RillError {
    /* TODO: Use
    #[error("not installed")]
    NotInstalled,
    #[error("alreary installed")]
    AlreadyInstalled,
    */
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
        // TODO: Prevent it be called twice
        let config = RillConfig::new(host, name.into(), with_meta);
        let actor = RillSupervisor::new(config);
        let scoped = meio::thread::spawn(actor)?;

        // TODO: Refactor that below
        let when = Instant::now();
        let how_long = Duration::from_secs(10);
        loop {
            if RILL_LINK.get().is_some() {
                break;
            }
            if when.elapsed() > how_long {
                return Err(Error::msg("rillrate still not started..."));
            }
        }

        Ok(Self { _scoped: scoped })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ordinary usage.
    #[test]
    fn test_install() -> Result<(), Error> {
        let _rill = Rill::install("127.0.0.1:1636".into(), "rill", false);
        let counter = tracers::CounterTracer::new("counter".parse()?, false);
        counter.inc(1.0, None);
        let active_counter = tracers::CounterTracer::new("active_counter".parse()?, true);
        active_counter.inc(1.0, None);
        Ok(())
    }

    /// `Rill` provider is not exists here, but tracers must not panic.
    #[test]
    fn test_provider_without_tracer() -> Result<(), Error> {
        let counter = tracers::CounterTracer::new("counter".parse()?, false);
        let active_counter = tracers::CounterTracer::new("active_counter".parse()?, true);
        for _ in 0..1_000_000 {
            counter.inc(1.0, None);
            active_counter.inc(1.0, None);
        }
        Ok(())
    }
}
