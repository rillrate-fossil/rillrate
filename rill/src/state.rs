use crate::tracers::counter::CounterDelta;
use crate::tracers::gauge::GaugeUpdate;
use crate::tracers::logger::LogRecord;
use crate::tracers::tracer::DataReceiver;
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

// TODO: Consider combining with StreamType
// TODO: Refactor that
#[derive(Debug)]
pub enum TracerFlow {
    Counter {
        receiver: DataReceiver<CounterDelta>,
    },
    Gauge {
        receiver: DataReceiver<GaugeUpdate>,
    },
    Log {
        receiver: DataReceiver<LogRecord>,
    },
}

pub trait ForFlow<T> {
    fn for_flow(rx: DataReceiver<T>) -> Self;
}

impl ForFlow<CounterDelta> for TracerFlow {
    fn for_flow(rx: DataReceiver<CounterDelta>) -> Self {
        Self::Counter { receiver: rx }
    }
}

impl ForFlow<LogRecord> for TracerFlow {
    fn for_flow(rx: DataReceiver<LogRecord>) -> Self {
        Self::Log { receiver: rx }
    }
}

impl ForFlow<GaugeUpdate> for TracerFlow {
    fn for_flow(rx: DataReceiver<GaugeUpdate>) -> Self {
        Self::Gauge { receiver: rx }
    }
}

pub(crate) enum DataSource {
    Receiver { receiver: DataReceiver<()> },
}

#[derive(Debug)]
pub(crate) enum UpgradeStateEvent {
    RegisterTracer {
        description: Arc<Description>,
        flow: TracerFlow,
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
