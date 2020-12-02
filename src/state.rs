use crate::protocol::EntryId;
use crate::provider::{DataReceiver, Joint};
use futures::channel::mpsc;
use meio::prelude::Action;
use once_cell::sync::OnceCell;
use std::sync::Arc;

pub static RILL_STATE: OnceCell<RillState> = OnceCell::new();

pub enum ControlEvent {
    RegisterJoint {
        entry_id: EntryId,
        joint: Arc<Joint>,
        rx: DataReceiver,
    },
}

impl Action for ControlEvent {}

pub type ControlSender = mpsc::UnboundedSender<ControlEvent>;
pub type ControlReceiver = mpsc::UnboundedReceiver<ControlEvent>;

pub struct RillState {
    sender: ControlSender,
}

impl RillState {
    pub fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self { sender: tx };
        (rx, this)
    }

    pub fn send(&self, event: ControlEvent) {
        self.sender
            .unbounded_send(event)
            .expect("rill actors not started");
    }
}
