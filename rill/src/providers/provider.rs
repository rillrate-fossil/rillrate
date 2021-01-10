//! This module contains a generic `Provider`'s methods.
use crate::state::{ControlEvent, RILL_STATE};
use anyhow::Error;
use futures::channel::mpsc;
use meio::prelude::Action;
use rill_protocol::provider::{Description, Path, RillData};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::watch;

#[derive(Debug)]
pub(crate) struct DataEnvelope {
    pub idx: usize,
    pub timestamp: SystemTime,
    pub data: RillData,
}

impl Action for DataEnvelope {}

pub(crate) type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub(crate) type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

/// Used to control the streams and interaction between a sender and a receiver.
#[derive(Debug)]
pub(crate) struct Joint {
    /// The index of the binding in the `Worker`.
    idx: AtomicUsize,
    description: Description,
}

impl Joint {
    fn new(description: Description) -> Self {
        Self {
            idx: AtomicUsize::new(0),
            description,
        }
    }

    pub fn assign(&self, idx: usize) {
        self.idx.store(idx, Ordering::Relaxed);
    }

    pub fn index(&self) -> usize {
        self.idx.load(Ordering::Relaxed)
    }

    pub fn description(&self) -> &Description {
        &self.description
    }
}

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Provider`.
#[derive(Debug)]
pub struct Provider {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<bool>,
    joint: Arc<Joint>,
    sender: DataSender,
}

impl Provider {
    pub(crate) fn new(description: Description) -> Self {
        log::trace!("Creating Provider with path: {:?}", description.path);
        let (tx, rx) = mpsc::unbounded();
        let (active_tx, active_rx) = watch::channel(false);
        let joint = Arc::new(Joint::new(description));
        let this = Provider {
            active: active_rx,
            joint: joint.clone(),
            sender: tx,
        };
        let event = ControlEvent::RegisterProvider {
            joint,
            active: active_tx,
            rx,
        };
        let state = RILL_STATE.get().expect("rill not installed!");
        state.send(event);
        this
    }

    /// Returns a reference to a `Path` of the `Provider`.
    pub fn path(&self) -> &Path {
        &self.joint.description().path
    }

    pub(crate) fn send(&self, data: RillData, timestamp: Option<SystemTime>) {
        let timestamp = timestamp.unwrap_or_else(SystemTime::now);
        let envelope = DataEnvelope {
            idx: self.joint.index(),
            timestamp,
            data,
        };
        if let Err(err) = self.sender.unbounded_send(envelope) {
            log::error!("Can't transfer data to sender: {}", err);
        }
    }
}

impl Provider {
    /// Returns `true` is the `Provider` has to send data.
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
            if *self.active.borrow() {
                break;
            }
            self.active.changed().await?;
        }
        Ok(())
    }
}
