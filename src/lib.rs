use std::thread;

mod macros;
pub mod protocol;
pub mod provider;
mod worker;

use once_cell::sync::OnceCell;
use protocol::EntryId;
pub use provider::StaticJoint;
use provider::{ControlEvent, ControlReceiver, RillState};
use thiserror::Error;

static RILL_STATE: OnceCell<RillState> = OnceCell::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error("alreary installed")]
    AlreadyInstalled,
}

pub fn install(name: impl Into<EntryId>) -> Result<(), Error> {
    let (rx, state) = RillState::create();
    RILL_STATE.set(state).map_err(|_| Error::AlreadyInstalled)?;
    let entry_id = name.into();
    thread::spawn(move || worker::entrypoint(entry_id, rx));
    Ok(())
}

pub fn bind_all(providers: &[&'static StaticJoint]) {
    for provider in providers {
        provider.register();
    }
}
