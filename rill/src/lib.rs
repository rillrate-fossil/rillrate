use std::thread;

mod actors;
mod exporters;
pub mod macros;
pub mod pathfinder;
pub mod prelude;
pub mod protocol;
pub mod providers;
mod state;

use actors::runtime::term;
use protocol::EntryId;
use state::{RillState, RILL_STATE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("alreary installed")]
    AlreadyInstalled,
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
    sender: Option<term::Sender>,
}

impl Rill {
    pub fn install(name: impl Into<EntryId>) -> Result<Self, Error> {
        let (term_tx, term_rx) = term::channel();
        let (rx, state) = RillState::create();
        RILL_STATE.set(state).map_err(|_| Error::AlreadyInstalled)?;
        let entry_id = name.into();
        thread::spawn(move || actors::runtime::entrypoint(entry_id, rx, term_rx));
        Ok(Self {
            sender: Some(term_tx),
        })
    }

    /* TODO: I'm not sure the methods below are necessary. Keep it commented.
    pub fn exempt(mut self) {
        self.no_wait();
        // And let it `Drop`.
    }

    /// Don't wait for the `Worker` termination on drop of this instance.
    pub fn no_wait(&mut self) {
        self.sender.take();
    }
    */

    fn terminate(&mut self) {
        if let Some(sender) = self.sender.take() {
            if let Err(_) = sender.notifier_tx.send(()) {
                log::error!("Can't send termination signal to the rill state.");
                return;
            }
            if let Err(_) = sender.blocker.lock() {
                log::error!("Can't wait for termination of the rill state.");
            }
        }
    }
}

impl Drop for Rill {
    fn drop(&mut self) {
        self.terminate();
    }
}
