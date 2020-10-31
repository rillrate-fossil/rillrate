use std::thread;

mod connector;
pub mod protocol;
pub mod provider;
mod worker;

use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use provider::ProviderCell;
use thiserror::Error;

enum ControlEvent {
    RegisterStream { provider: &'static ProviderCell },
}

impl Action for ControlEvent {}

type ControlSender = mpsc::UnboundedSender<ControlEvent>;
type ControlReceiver = mpsc::UnboundedReceiver<ControlEvent>;

static RILL: OnceCell<ControlSender> = OnceCell::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error("alreary installed")]
    AlreadyInstalled,
}

pub fn install() -> Result<(), Error> {
    let (tx, rx) = mpsc::unbounded();
    RILL.set(tx).map_err(|_| Error::AlreadyInstalled)?;
    thread::spawn(move || worker::entrypoint(rx));
    Ok(())
}

pub fn bind(provider: &'static ProviderCell) {
    if let Some(sender) = RILL.get() {
        let event = ControlEvent::RegisterStream { provider };
        sender.unbounded_send(event);
    }
}
