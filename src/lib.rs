use std::thread;

pub mod protocol;
pub mod provider;
mod worker;

use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use provider::{Data, ProviderCell};
use thiserror::Error;

enum ControlEvent {
    RegisterStream {
        provider: &'static ProviderCell,
        initial_receiver: mpsc::UnboundedReceiver<Data>,
    },
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

pub fn bind_all(providers: &[&'static ProviderCell]) {
    for provider in providers {
        bind(provider);
    }
}

pub fn bind(provider: &'static ProviderCell) {
    if let Some(sender) = RILL.get() {
        let initial_receiver = provider.init();
        let event = ControlEvent::RegisterStream {
            provider,
            initial_receiver,
        };
        sender.unbounded_send(event);
    }
}
