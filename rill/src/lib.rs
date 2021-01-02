mod actors;
pub mod macros;
pub mod prelude;
pub mod protocol;
pub mod providers;
mod state;

use crate::actors::supervisor::RillSupervisor;
use anyhow::Error;
use rill_protocol::EntryId;
use state::{RillState, RILL_STATE};
use thiserror::Error;

metacrate::meta!();

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
    _scoped: meio::thread::ScopedRuntime,
}

impl Rill {
    pub fn install(name: impl Into<EntryId>) -> Result<Self, Error> {
        let (rx, state) = RillState::create();
        RILL_STATE
            .set(state)
            .map_err(|_| RillError::AlreadyInstalled)?;
        let actor = RillSupervisor::new(name.into(), rx);
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self { _scoped: scoped })
    }
}
