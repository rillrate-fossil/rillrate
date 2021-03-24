use crate::actors::worker::RillWorker;
use crate::tracers::tracer::{TracerDescription, TracerMode};
use anyhow::Error;
use futures::channel::mpsc;
use futures::lock::Mutex;
use meio::{InstantAction, InstantActionHandler, Parcel};
use once_cell::sync::Lazy;
use rill_protocol::flow::data;
use std::sync::Arc;

/// It used by tracers to register them into the state.
pub(crate) static RILL_LINK: Lazy<RillState> = Lazy::new(RillState::new);

type Sender = mpsc::UnboundedSender<Parcel<RillWorker>>;
type Receiver = mpsc::UnboundedReceiver<Parcel<RillWorker>>;

pub(crate) struct RillState {
    pub sender: Sender,
    pub receiver: Mutex<Option<Receiver>>,
}

impl RillState {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        let receiver = Mutex::new(Some(rx));
        Self {
            sender: tx,
            receiver,
        }
    }

    pub fn register_tracer<T>(
        &self,
        description: Arc<TracerDescription<T>>,
        mode: TracerMode<T>,
    ) -> Result<(), Error>
    where
        RillWorker: InstantActionHandler<RegisterTracer<T>>,
        T: data::Flow,
    {
        let msg = RegisterTracer { description, mode };
        let parcel = Parcel::pack(msg);
        self.sender
            .unbounded_send(parcel)
            .map_err(|_| Error::msg("Can't register a tracer."))
    }

    pub async fn take_receiver(&self) -> Option<Receiver> {
        self.receiver.lock().await.take()
    }
}

pub(crate) struct RegisterTracer<T: data::Flow> {
    pub description: Arc<TracerDescription<T>>,
    pub mode: TracerMode<T>,
}

impl<T: data::Flow> InstantAction for RegisterTracer<T> {}
