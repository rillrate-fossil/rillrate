use crate::providers::provider::DataReceiver;
use futures::channel::mpsc;
use meio::prelude::Action;
use once_cell::sync::OnceCell;
use rill_protocol::provider::Description;
use std::sync::Arc;
use tokio::sync::watch;

/// It used by providers to register them into the state.
pub(crate) static RILL_STATE: OnceCell<RillState> = OnceCell::new();

/*
pub(crate) enum ProviderMode {
    /// Always active stream. Worker can create snapshots for that.
    Active {
        // TODO: Add id that acquired from a counter
    },
    Reactive {
        activator: watch::Sender<Option<usize>>,
    },
}
*/

pub(crate) enum ProviderMode {
    /// Lazy stream that can be activates. No snapshots available for that. Deltas only.
    Reactive {
        /// Used to to activate a `Provider.` The value set represents the index of
        /// the stream inside `Worker` that has to be used for sending messages.
        activator: watch::Sender<bool>,
    },
}

pub(crate) struct RegisterProvider {
    pub description: Arc<Description>,
    pub mode: ProviderMode,
    pub rx: DataReceiver,
}

impl Action for RegisterProvider {}

pub(crate) type ControlSender = mpsc::UnboundedSender<RegisterProvider>;
pub(crate) type ControlReceiver = mpsc::UnboundedReceiver<RegisterProvider>;

pub(crate) struct RillState {
    sender: ControlSender,
}

impl RillState {
    pub fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self { sender: tx };
        (rx, this)
    }

    pub fn send(&self, event: RegisterProvider) {
        self.sender
            .unbounded_send(event)
            .expect("rill actors not started");
    }
}
