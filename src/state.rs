use crate::{provider, provider2};
use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;

pub static RILL_STATE: OnceCell<RillState> = OnceCell::new();

pub enum ControlEvent {
    // TODO: Use the single `RegisterAllJoints` event with no `Completed` variant.
    RegisterJoint {
        joint: Box<dyn provider::Joint>,
        rx: provider::DataReceiver,
    },
    RegisterJoint2(provider2::RegisterJoint),
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
