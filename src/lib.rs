use std::thread;

mod drivers;
mod macros;
pub mod protocol;
pub mod provider;
mod worker;

use once_cell::sync::OnceCell;
pub use provider::StaticJoint;
use provider::{ControlEvent, ControlReceiver, RillState};
use thiserror::Error;

static RILL_STATE: OnceCell<RillState> = OnceCell::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error("alreary installed")]
    AlreadyInstalled,
}

pub fn install() -> Result<(), Error> {
    let (rx, state) = RillState::create();
    RILL_STATE.set(state).map_err(|_| Error::AlreadyInstalled)?;
    thread::spawn(move || worker::entrypoint(rx));
    Ok(())
}

pub fn bind_all(providers: &[&'static StaticJoint]) {
    for provider in providers {
        bind(provider);
    }
}

pub fn bind(provider: &'static StaticJoint) {
    provider.register();
}
