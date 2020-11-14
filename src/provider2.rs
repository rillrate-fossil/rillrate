use crate::protocol::{EntryId, RillData};
use crate::state::{ControlEvent, RILL_STATE};
use futures::channel::mpsc;
use meio::Action;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct DataEnvelope {
    pub data: RillData,
}

impl Action for DataEnvelope {}

pub type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

/// Used to control the streams and interaction between a sender and a receiver.
#[derive(Debug)]
pub struct Joint {
    //uid?
    entry_id: EntryId,
    active: AtomicBool,
}

impl Joint {
    pub fn entry_id(&self) -> &EntryId {
        &self.entry_id
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn switch(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct Provider {
    joint: Arc<Joint>,
    sender: DataSender,
}

impl Provider {
    pub fn new(entry_id: EntryId) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let this = Joint {
            entry_id,
            active: AtomicBool::new(false),
        };
        let joint = Arc::new(this);
        let this = Provider {
            joint: joint.clone(),
            sender: tx,
        };
        let event = ControlEvent::RegisterJoint2 { joint, rx };
        let state = RILL_STATE.get().expect("rill not installed!");
        state.send(event);
        this
    }
}
