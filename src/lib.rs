use std::thread;

pub mod protocol;
pub mod provider;
mod worker;

use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use protocol::StreamId;
use provider::{DataReceiver, ProviderCell};
use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;

enum ControlEvent {
    RegisterStream {
        provider: &'static ProviderCell,
        rx: DataReceiver,
    },
}

impl Action for ControlEvent {}

type ControlSender = mpsc::UnboundedSender<ControlEvent>;
type ControlReceiver = mpsc::UnboundedReceiver<ControlEvent>;

struct RillState {
    sender: ControlSender,
    counter: AtomicUsize,
}

impl RillState {
    fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            sender: tx,
            counter: AtomicUsize::new(0),
        };
        (rx, this)
    }

    fn next(&self) -> StreamId {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        StreamId(id as u64)
    }

    fn send(&self, event: ControlEvent) {
        self.sender.unbounded_send(event);
    }
}

static RILL: OnceCell<RillState> = OnceCell::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error("alreary installed")]
    AlreadyInstalled,
}

pub fn install() -> Result<(), Error> {
    let (rx, state) = RillState::create();
    RILL.set(state).map_err(|_| Error::AlreadyInstalled)?;
    thread::spawn(move || worker::entrypoint(rx));
    Ok(())
}

pub fn bind_all(providers: &[&'static ProviderCell]) {
    for provider in providers {
        bind(provider);
    }
}

pub fn bind(provider: &'static ProviderCell) {
    if let Some(state) = RILL.get() {
        let stream_id = state.next();
        // IMPORTANT: Initialize `Provider` here to create the channel before it
        // will be used by the user.
        let rx = provider.init(stream_id);
        let event = ControlEvent::RegisterStream { provider, rx };
        state.send(event);
    }
}
