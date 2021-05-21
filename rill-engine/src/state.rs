use crate::actors::connector::RillConnector;
use crate::tracers::tracer::TracerMode;
use anyhow::Error;
use futures::channel::mpsc;
use futures::lock::Mutex;
use meio::{Actor, InstantAction, InstantActionHandler, Parcel};
use once_cell::sync::Lazy;
use rill_protocol::flow::core;
use rill_protocol::io::provider::Description;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Reserved receiver already taken.")]
pub struct AlreadyTaken;

/// It used by tracers to register them into the state.
pub(crate) static RILL_LINK: Lazy<ParcelDistributor<RillConnector>> =
    Lazy::new(ParcelDistributor::new);

pub(crate) struct ParcelDistributor<A: Actor> {
    pub sender: mpsc::UnboundedSender<Parcel<A>>,
    pub receiver: Mutex<Option<mpsc::UnboundedReceiver<Parcel<A>>>>,
}

impl<A: Actor> ParcelDistributor<A> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        let receiver = Mutex::new(Some(rx));
        Self {
            sender: tx,
            receiver,
        }
    }

    pub async fn take_receiver(&self) -> Result<mpsc::UnboundedReceiver<Parcel<A>>, AlreadyTaken> {
        self.receiver.lock().await.take().ok_or(AlreadyTaken)
    }
}

pub(crate) struct RegisterTracer<T: core::Flow> {
    pub description: Arc<Description>,
    pub mode: TracerMode<T>,
}

impl<T: core::Flow> InstantAction for RegisterTracer<T> {}

impl ParcelDistributor<RillConnector> {
    pub fn register_tracer<T>(
        &self,
        description: Arc<Description>,
        mode: TracerMode<T>,
    ) -> Result<(), Error>
    where
        RillConnector: InstantActionHandler<RegisterTracer<T>>,
        T: core::Flow,
    {
        let msg = RegisterTracer { description, mode };
        let parcel = Parcel::pack(msg);
        self.sender
            .unbounded_send(parcel)
            .map_err(|_| Error::msg("Can't register a tracer."))
    }
}
