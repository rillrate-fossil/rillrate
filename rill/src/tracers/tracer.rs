//! This module contains a generic `Tracer`'s methods.
use crate::state::{
    RealtimeFlow, SnapshotFlow, StorageFlow, TracerMode, UpgradeStateEvent, RILL_STATE,
};
use anyhow::Error;
use futures::channel::mpsc;
use meio::prelude::Action;
use rill_protocol::provider::{Description, Path, RillData};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum DataEnvelope {
    DataEvent {
        system_time: SystemTime,
        data: RillData,
    },
    EndStream {
        description: Arc<Description>,
    },
}

impl Action for DataEnvelope {}

pub(crate) type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub(crate) type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

pub(crate) type FlowSender = broadcast::Sender<DataEnvelope>;
pub(crate) type FlowReceiver = broadcast::Receiver<DataEnvelope>;

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Tracer`.
#[derive(Debug)]
pub struct Tracer {
    description: Arc<Description>,
    sender: broadcast::Sender<DataEnvelope>,
    // The receiver that used to activate/deactivate streams.
    //active: watch::Receiver<bool>,
    //sender: DataSender,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TracerType {
    Realtime,
    Snapshot,
    Storage,
}

impl Tracer {
    pub(crate) fn new(description: Description, tracer_type: &[TracerType]) -> Self {
        log::trace!("Creating Tracer with path: {:?}", description.path);
        let opt_state = RILL_STATE.get();
        let description = Arc::new(description);
        let (tx, rx) = broadcast::channel(128);
        if let Some(state) = opt_state {
            let realtime = tracer_type
                .contains(&TracerType::Realtime)
                .then(|| RealtimeFlow { sender: tx.clone() });
            let snapshot = tracer_type
                .contains(&TracerType::Snapshot)
                .then(|| SnapshotFlow {
                    receiver: tx.subscribe(),
                });
            let storage = tracer_type
                .contains(&TracerType::Storage)
                .then(|| StorageFlow {
                    receiver: tx.subscribe(),
                });
            let event = UpgradeStateEvent::RegisterTracer {
                description: description.clone(),
                realtime,
                snapshot,
                storage,
            };
            state.upgrade(event);
        }
        Self {
            description,
            sender: tx,
        }
        /*
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
        let source = DataSource::Receiver { receiver: rx };
        let event = UpgradeStateEvent::RegisterTracer {
            description,
            mode,
            source,
        };
        if let Some(state) = opt_state {
            state.upgrade(event);
        }
        this
        */
    }

    /// Returns a reference to a `Path` of the `Tracer`.
    pub fn path(&self) -> &Path {
        &self.description.path
    }

    pub(crate) fn send(&self, data: RillData, opt_system_time: Option<SystemTime>) {
        // If there is no rill tracer than it will never be active.
        if self.is_active() {
            let system_time = opt_system_time.unwrap_or_else(SystemTime::now);
            let envelope = DataEnvelope::DataEvent { system_time, data };
            // And will never send an event
            if let Err(send_err) = self.sender.send(envelope) {
                // TODO: Do something with unsent data dependent on strategy:
                // DROP or RETRY maybe RETRY_COUNTER or SWITCH_TO_PULLING
                //log::error!("Can't transfer data to sender: {}", err);
                // TODO: If can't send start batching or aggregating
            }
        }
    }
}

impl Tracer {
    pub fn is_active(&self) -> bool {
        self.sender.receiver_count() > 0
    }

    /*
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
    */
}

impl Drop for Tracer {
    fn drop(&mut self) {
        /*
        let end_stream = DataEnvelope::EndStream {
            description: self.description.clone(),
        };
        if let Err(_err) = self.sender.unbounded_send(end_stream) {
            log::error!(
                "Can't send `EndStream` to the worker actor from: {}",
                self.description.path
            );
        }
        */
    }
}
