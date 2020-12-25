mod actors;
pub mod macros;
pub mod pathfinder;
pub mod prelude;
pub mod protocol;
pub mod providers;
mod state;

use crate::actors::supervisor::RillSupervisor;
use anyhow::Error;
use protocol::EntryId;
use state::{RillState, RILL_STATE};
use std::sync::atomic::{AtomicU16, Ordering};
use thiserror::Error;

pub static PORT: Port = Port::new(crate::protocol::PORT);

pub struct Port {
    value: AtomicU16,
}

impl Port {
    const fn new(value: u16) -> Self {
        Self {
            value: AtomicU16::new(value),
        }
    }

    pub fn set(&self, value: u16) {
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> u16 {
        self.value.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Error)]
pub enum RillError {
    #[error("alreary installed")]
    AlreadyInstalled,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    /*
    #[error("not installed")]
    NotInstalled,
    #[error("can't find termination handler")]
    NoTerminationHandler,
    #[error("termination failed")]
    TerminationFailed,
    */
}

pub struct Rill {
    scoped: meio::thread::ScopedRuntime,
}

impl Rill {
    pub fn install(name: impl Into<EntryId>) -> Result<Self, Error> {
        let (rx, state) = RillState::create();
        RILL_STATE
            .set(state)
            .map_err(|_| RillError::AlreadyInstalled)?;
        let actor = RillSupervisor::new(name.into(), rx);
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self { scoped })
    }
}
