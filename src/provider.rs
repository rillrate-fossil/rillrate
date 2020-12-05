use crate::protocol::{Path, RillData};
use crate::state::{ControlEvent, RILL_STATE};
use futures::channel::mpsc;
use meio::prelude::Action;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
// TODO: Move to user featrues part
//use std::time::{SystemTime, UNIX_EPOCH};

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
    /// The flag that used to activate/deactivate streams.
    active: AtomicBool,
}

impl Joint {
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn switch(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    pub fn assign(&self, idx: usize) {
        self.idx.store(idx, Ordering::Relaxed);
    }

    pub fn index(&self) -> usize {
        self.idx.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct Provider {
    joint: Arc<Joint>,
    sender: DataSender,
}

impl Provider {
    // TODO: Add type of the stream...
    pub fn new(path: Path) -> Self {
        log::trace!("Creating Provider with path: {:?}", path);
        let (tx, rx) = mpsc::unbounded();
        let joint = Arc::new(Joint::default());
        let this = Provider {
            joint: joint.clone(),
            sender: tx,
        };
        let event = ControlEvent::RegisterJoint { path, joint, rx };
        let state = RILL_STATE.get().expect("rill not installed!");
        state.send(event);
        this
    }

    pub fn is_active(&self) -> bool {
        self.joint.is_active()
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

    /*
    pub fn log(&self, message: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let data = RillData::LogRecord {
            timestamp: now as i64, //TODO: Change to u128 instead?
            message,
        };
        self.send(data);
    }
    */
}

#[derive(Debug)]
pub struct LogProvider {
    provider: Provider,
}

impl LogProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path);
        Self { provider }
    }

    pub fn is_active(&self) -> bool {
        self.provider.is_active()
    }

    pub fn log(&self, timestamp: String, message: String) {
        let data = RillData::LogRecord { timestamp, message };
        self.provider.send(data);
    }
}
