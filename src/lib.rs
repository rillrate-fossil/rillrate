use std::thread;

mod macros;
pub mod protocol;
pub mod provider;
mod worker;

use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use protocol::StreamId;
use provider::DataReceiver;
pub use provider::ProviderCell;
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
    stream_id_counter: AtomicUsize,
}

impl RillState {
    fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            sender: tx,
            stream_id_counter: AtomicUsize::new(0),
        };
        (rx, this)
    }

    fn next(&self) -> StreamId {
        let id = self.stream_id_counter.fetch_add(1, Ordering::Relaxed);
        StreamId(id as u64)
    }

    fn send(&self, event: ControlEvent) {
        self.sender
            .unbounded_send(event)
            .expect("rill actors not started");
    }
}

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

pub fn bind_all(providers: &[&'static ProviderCell]) {
    for provider in providers {
        bind(provider);
    }
}

pub fn bind(provider: &'static ProviderCell) {
    if let Some(state) = RILL_STATE.get() {
        let stream_id = state.next();
        // IMPORTANT: Initialize `Provider` here to create the channel before it
        // will be used by the user.
        let rx = provider.init(stream_id);
        let event = ControlEvent::RegisterStream { provider, rx };
        state.send(event);
    }
}
