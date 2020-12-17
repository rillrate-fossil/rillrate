use std::thread;

pub mod macros;
pub mod pathfinder;
pub mod prelude;
pub mod protocol;
pub mod providers;
mod state;
mod term;
mod worker;

use once_cell::sync::OnceCell;
use protocol::EntryId;
use state::{RillState, RILL_STATE};
use std::sync::Mutex;
use thiserror::Error;

static INSTANCE: OnceCell<Mutex<Option<term::Sender>>> = OnceCell::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error("alreary installed")]
    AlreadyInstalled,
    #[error("not installed")]
    NotInstalled,
    #[error("can't find termination handler")]
    NoTerminationHandler,
    #[error("termination failed")]
    TerminationFailed,
}

pub fn install(name: impl Into<EntryId>) -> Result<(), Error> {
    let (term_tx, term_rx) = term::channel();
    let term_sender = Mutex::new(Some(term_tx));
    INSTANCE
        .set(term_sender)
        .map_err(|_| Error::AlreadyInstalled)?;
    let (rx, state) = RillState::create();
    RILL_STATE.set(state).map_err(|_| Error::AlreadyInstalled)?;
    let entry_id = name.into();
    thread::spawn(move || worker::entrypoint(entry_id, rx, term_rx));
    Ok(())
}

/* TODO: Consider what to do?
pub fn awake(provider: &Lazy<Provider>) {
    Lazy::force(provider);
}
*/

pub fn terminate() -> Result<(), Error> {
    let mutex = INSTANCE.get().ok_or(Error::NotInstalled)?;
    let term_sender = mutex
        .lock()
        .map_err(|_| Error::NoTerminationHandler)?
        .take()
        .ok_or(Error::NoTerminationHandler)?;
    drop(mutex);
    term_sender
        .notifier_tx
        .send(())
        .map_err(|_| Error::TerminationFailed)?;
    let blocker = term_sender
        .blocker
        .lock()
        .map_err(|_| Error::TerminationFailed)?;
    drop(blocker);
    Ok(())
}
