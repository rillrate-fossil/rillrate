use crate::protocol::{Path, RillData, StreamType};
use crate::state::{ControlEvent, RILL_STATE};
use anyhow::Error;
use futures::channel::mpsc;
use meio::prelude::Action;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
// TODO: Move to user featrues part
//use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

#[derive(Debug)]
pub struct DataEnvelope {
    pub idx: usize,
    pub data: RillData,
}

impl Action for DataEnvelope {}

pub type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

/// Used to control the streams and interaction between a sender and a receiver.
#[derive(Debug, Default)]
pub(crate) struct Joint {
    /// The index of the binding in the `Worker`.
    idx: AtomicUsize,
}

impl Joint {
    pub fn assign(&self, idx: usize) {
        self.idx.store(idx, Ordering::Relaxed);
    }

    pub fn index(&self) -> usize {
        self.idx.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct Provider {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<bool>,
    joint: Arc<Joint>,
    sender: DataSender,
}

impl Provider {
    pub(crate) fn new(path: Path, stream_type: StreamType) -> Self {
        log::trace!("Creating Provider with path: {:?}", path);
        let (tx, rx) = mpsc::unbounded();
        let (active_tx, active_rx) = watch::channel(false);
        let joint = Arc::new(Joint::default());
        let this = Provider {
            active: active_rx,
            joint: joint.clone(),
            sender: tx,
        };
        let event = ControlEvent::RegisterJoint {
            path,
            stream_type,
            joint,
            active: active_tx,
            rx,
        };
        let state = RILL_STATE.get().expect("rill not installed!");
        state.send(event);
        this
    }

    pub(crate) fn send(&self, data: RillData) {
        let envelope = DataEnvelope {
            idx: self.joint.index(),
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
            // TODO: Change to separate error type
            let is_active = self
                .active
                .recv()
                .await
                .ok_or_else(|| Error::msg("rill is not available"))?;
            if is_active {
                break;
            }
        }
        Ok(())
    }
}
