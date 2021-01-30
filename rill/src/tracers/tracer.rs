//! This module contains a generic `Tracer`'s methods.
use crate::state::{TracerMode, UpgradeStateEvent, RILL_STATE};
use anyhow::Error;
use futures::channel::mpsc;
use meio::prelude::Action;
use rill_protocol::provider::{Description, Path, RillData};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::watch;

#[derive(Debug)]
pub(crate) enum DataEnvelope {
    DataEvent {
        timestamp: SystemTime,
        data: RillData,
    },
    EndStream {
        description: Arc<Description>,
    },
}

impl Action for DataEnvelope {}

pub(crate) type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub(crate) type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Tracer`.
#[derive(Debug)]
pub struct Tracer {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<bool>,
    description: Arc<Description>,
    sender: DataSender,
}

impl Tracer {
    pub(crate) fn new(description: Description, mut active: bool) -> Self {
        log::trace!("Creating Tracer with path: {:?}", description.path);
        let opt_state = RILL_STATE.get();
        if opt_state.is_none() {
            // If there is no tracer than the `active` flag will never be true.
            active = false;
            log::warn!(
                "No rill tracer available: {} provider deactivated.",
                description.path
            );
        }
        let (tx, rx) = mpsc::unbounded();
        let (active_tx, active_rx) = watch::channel(active);
        let description = Arc::new(description);
        let this = Tracer {
            active: active_rx,
            description: description.clone(),
            sender: tx,
        };
        let mode = {
            if active {
                TracerMode::Active
            } else {
                TracerMode::Reactive {
                    activator: active_tx,
                }
            }
        };
        let event = UpgradeStateEvent::RegisterTracer {
            description,
            mode,
            rx,
        };
        if let Some(state) = opt_state {
            state.upgrade(event);
        }
        this
    }

    /// Returns a reference to a `Path` of the `Tracer`.
    pub fn path(&self) -> &Path {
        &self.description.path
    }

    pub(crate) fn send(&self, data: RillData, timestamp: Option<SystemTime>) {
        // If there is no rill tracer than it will never be active.
        if *self.active.borrow() {
            let timestamp = timestamp.unwrap_or_else(SystemTime::now);
            let envelope = DataEnvelope::DataEvent { timestamp, data };
            // And will never send an event
            if let Err(err) = self.sender.unbounded_send(envelope) {
                log::error!("Can't transfer data to sender: {}", err);
            }
        }
    }
}

impl Tracer {
    /// Returns `true` is the `Tracer` has to send data.
    pub fn is_active(&self) -> bool {
        *self.active.borrow()
    }

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
}

impl Drop for Tracer {
    fn drop(&mut self) {
        let end_stream = DataEnvelope::EndStream {
            description: self.description.clone(),
        };
        if let Err(_err) = self.sender.unbounded_send(end_stream) {
            log::error!(
                "Can't send `EndStream` to the worker actor from: {}",
                self.description.path
            );
        }
    }
}
