use crate::tracers::tracer::{FlowReceiver, FlowSender};
use futures::channel::mpsc;
use meio::prelude::Action;
use once_cell::sync::OnceCell;
use rill_protocol::provider::Description;
use std::sync::Arc;
use tokio::sync::watch;

/// It used by tracers to register them into the state.
pub(crate) static RILL_STATE: OnceCell<RillState> = OnceCell::new();

pub(crate) enum TracerMode {
    /// Always active stream. Worker can create snapshots for that.
    Active,
    /// Lazy stream that can be activates. No snapshots available for that. Deltas only.
    Reactive {
        /// Used to to activate a `Tracer.` The value set represents the index of
        /// the stream inside `Worker` that has to be used for sending messages.
        activator: watch::Sender<bool>,
    },
}

/*
pub(crate) enum DataSource {
    Receiver { receiver: DataReceiver },
}
*/

pub(crate) struct RealtimeFlow {
    // TODO: Use broadcast::Sender here and subscribe
    // when data is necessary.
    // Active flag is not needed anymore )
    pub sender: FlowSender,
}

pub(crate) struct SnapshotFlow {
    pub receiver: FlowReceiver,
}

pub(crate) struct StorageFlow {
    pub receiver: FlowReceiver,
}

pub(crate) enum UpgradeStateEvent {
    RegisterTracer {
        description: Arc<Description>,
        realtime: Option<RealtimeFlow>,
        snapshot: Option<SnapshotFlow>,
        storage: Option<StorageFlow>,
    },
}

impl Action for UpgradeStateEvent {}

pub(crate) type ControlSender = mpsc::UnboundedSender<UpgradeStateEvent>;
pub(crate) type ControlReceiver = mpsc::UnboundedReceiver<UpgradeStateEvent>;

pub(crate) struct RillState {
    sender: ControlSender,
}

impl RillState {
    pub fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self { sender: tx };
        (rx, this)
    }

    pub fn upgrade(&self, event: UpgradeStateEvent) {
        self.sender
            .unbounded_send(event)
            .expect("rill actors not started");
    }
}
