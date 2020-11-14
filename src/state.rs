use crate::protocol::EntryId;
use crate::{provider, provider2};
use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use std::sync::Arc;

pub static RILL_STATE: OnceCell<RillState> = OnceCell::new();

pub enum ControlEvent {
    // TODO: Use the single `RegisterAllJoints` event with no `Completed` variant.
    RegisterJoint {
        joint: Box<dyn provider::Joint>,
        rx: provider::DataReceiver,
    },
    RegisterJoint2 {
        entry_id: EntryId,
        joint: Arc<provider2::Joint>,
        rx: provider2::DataReceiver,
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
