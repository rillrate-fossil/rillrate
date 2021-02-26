//! This module contains a generic `Tracer`'s methods.
use crate::state::RILL_LINK;
use futures::channel::mpsc;
use meio::prelude::Action;
use rill_protocol::provider::{Description, Path, RillEvent, Timestamp};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::watch;

pub trait TracerState: Default + Send + 'static {
    type Item;
    fn aggregate(&mut self, items: Vec<DataEnvelope<Self::Item>>) -> Vec<RillEvent>;

    fn make_snapshot(&self) -> Vec<RillEvent>;
}

pub trait TracerEvent: Sized + Send + 'static {
    type State: TracerState<Item = Self>;
}

#[derive(Debug)]
pub enum DataEnvelope<T> {
    // TODO: Use `Timestamp` here and convert all incoiming
    // values inside `Tracers`
    Event { system_time: SystemTime, data: T },
}

impl<T> DataEnvelope<T> {
    // TODO: Remove this method
    pub fn unpack(self) -> (T, Timestamp) {
        // TODO: Fix this unwrap
        let DataEnvelope::Event { system_time, data } = self;
        let timestamp = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .into();
        (data, timestamp)
    }
}

impl<T: TracerEvent> Action for DataEnvelope<T> {}

// TODO: Remove that aliases and use raw types receivers in recorders.
pub type DataSender<T> = mpsc::UnboundedSender<DataEnvelope<T>>;
pub type DataReceiver<T> = mpsc::UnboundedReceiver<DataEnvelope<T>>;

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Tracer`.
#[derive(Debug)]
pub struct Tracer<T> {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<bool>,
    description: Arc<Description>,
    sender: DataSender<T>,
}

impl<T: TracerEvent> Tracer<T> {
    pub(crate) fn new(description: Description) -> Self {
        // TODO: Remove this active watch channel?
        let (_active_tx, active_rx) = watch::channel(true);
        log::trace!("Creating Tracer with path: {:?}", description.path);
        let (tx, rx) = mpsc::unbounded();
        let description = Arc::new(description);
        let this = Tracer {
            active: active_rx,
            description: description.clone(),
            sender: tx,
        };
        if let Err(err) = RILL_LINK.register_tracer(description, rx) {
            log::error!(
                "Can't register a Tracer. The worker can be terminated already: {}",
                err
            );
        }
        this
    }

    /// Returns a reference to a `Path` of the `Tracer`.
    pub fn path(&self) -> &Path {
        &self.description.path
    }

    pub(crate) fn send(&self, data: T, opt_system_time: Option<SystemTime>) {
        if self.is_active() {
            let system_time = opt_system_time.unwrap_or_else(SystemTime::now);
            let envelope = DataEnvelope::Event { system_time, data };
            // And will never send an event
            if let Err(err) = self.sender.unbounded_send(envelope) {
                log::error!("Can't transfer data to sender: {}", err);
            }
        }
    }
}

impl<T> Tracer<T> {
    /// Returns `true` is the `Tracer` has to send data.
    pub fn is_active(&self) -> bool {
        *self.active.borrow()
    }

    /* TODO: Remove or replace with an alternative
    /// Use this method to detect when stream had activated.
    ///
    /// It's useful if you want to spawn async coroutine that
    /// can read a batch of data, but will wait when some streams
    /// will be activated to avoid resources wasting.
    ///
    /// When the generating coroutine active you can use `is_active`
    /// method to detect when to change it to awaiting state again.
    pub async fn when_activated(&mut self) -> Result<(), Error> {
        loop {
            if self.is_active() {
                break;
            }
            self.active.changed().await?;
        }
        Ok(())
    }
    */
}
