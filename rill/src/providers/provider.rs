//! This module contains a generic `Provider`'s methods.
use crate::state::{ProviderMode, RegisterProvider, RILL_STATE};
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
        idx: usize,
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
/// for checking the activitiy status of the `Provider`.
#[derive(Debug)]
pub struct Provider {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<Option<usize>>,
    description: Arc<Description>,
    sender: DataSender,
}

impl Provider {
    pub(crate) fn new(description: Description) -> Self {
        log::trace!("Creating Provider with path: {:?}", description.path);
        let (tx, rx) = mpsc::unbounded();
        let (active_tx, active_rx) = watch::channel(None);
        let description = Arc::new(description);
        let this = Provider {
            active: active_rx,
            description: description.clone(),
            sender: tx,
        };
        let mode = ProviderMode { active: active_tx };
        let event = RegisterProvider {
            description,
            mode,
            rx,
        };
        let state = RILL_STATE.get().expect("rill is not installed!");
        state.send(event);
        this
    }

    /// Returns a reference to a `Path` of the `Provider`.
    pub fn path(&self) -> &Path {
        &self.description.path
    }

    pub(crate) fn send(&self, data: RillData, timestamp: Option<SystemTime>) {
        if let Some(idx) = *self.active.borrow() {
            let timestamp = timestamp.unwrap_or_else(SystemTime::now);
            let envelope = DataEnvelope::DataEvent {
                idx,
                timestamp,
                data,
            };
            if let Err(err) = self.sender.unbounded_send(envelope) {
                log::error!("Can't transfer data to sender: {}", err);
            }
        }
    }
}

impl Provider {
    /// Returns `true` is the `Provider` has to send data.
    pub fn is_active(&self) -> bool {
        self.active.borrow().is_some()
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

impl Drop for Provider {
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
