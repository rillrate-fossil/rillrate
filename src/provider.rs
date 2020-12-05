use crate::protocol::{Path, RillData, StreamType};
use crate::state::{ControlEvent, RILL_STATE};
use anyhow::{anyhow, Error};
use derive_more::{Deref, DerefMut};
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
    // TODO: Add type of the stream...
    fn new(path: Path, stream_type: StreamType) -> Self {
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

    pub fn is_active(&self) -> bool {
        *self.active.borrow()
    }

    pub async fn when_active(&mut self) -> Result<(), Error> {
        loop {
            // TODO: Change to separate error type
            let is_active = self
                .active
                .recv()
                .await
                .ok_or_else(|| anyhow!("rill is not available"))?;
            if is_active {
                break;
            }
        }
        Ok(())
    }

    fn send(&self, data: RillData) {
        let envelope = DataEnvelope {
            idx: self.joint.index(),
            data,
        };
        if let Err(err) = self.sender.unbounded_send(envelope) {
            log::error!("Can't transfer data to sender: {}", err);
        }
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct LogProvider {
    provider: Provider,
}

impl LogProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::LogStream);
        Self { provider }
    }

    pub fn log(&self, timestamp: String, message: String) {
        let data = RillData::LogRecord { timestamp, message };
        self.provider.send(data);
    }
}
