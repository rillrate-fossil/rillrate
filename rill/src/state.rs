use crate::actors::worker::{RegisterTracer, RillLink, RillWorker};
use crate::tracers::tracer::{DataReceiver, TracerEvent};
use anyhow::Error;
use futures::channel::mpsc;
use futures::lock::Mutex;
use meio::prelude::{InstantActionHandler, Parcel};
use once_cell::sync::OnceCell;
use rill_protocol::provider::Description;
use std::sync::Arc;

/// It used by tracers to register them into the state.
pub(crate) static RILL_LINK: OnceCell<RillState> = OnceCell::new();

type Sender = mpsc::UnboundedSender<Parcel<RillWorker>>;
type Receiver = mpsc::UnboundedReceiver<Parcel<RillWorker>>;

pub(crate) struct RillState {
    pub link: RillLink,
    pub sender: Sender,
    pub receiver: Mutex<Option<Receiver>>,
}

impl RillState {
    pub fn new(link: RillLink) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let receiver = Mutex::new(Some(rx));
        Self {
            link,
            sender: tx,
            receiver,
        }
    }

    pub fn register_tracer<T>(
        &self,
        description: Arc<Description>,
        receiver: DataReceiver<T>,
    ) -> Result<(), Error>
    where
        RillWorker: InstantActionHandler<RegisterTracer<T>>,
        T: TracerEvent,
    {
        let msg = RegisterTracer {
            description,
            receiver,
        };
        let parcel = Parcel::new(msg);
        self.sender
            .unbounded_send(parcel)
            .map_err(|_| Error::msg("Can't register a tracer."))
    }

    pub async fn take_receiver(&self) -> Option<Receiver> {
        self.receiver.lock().await.take()
    }
}
