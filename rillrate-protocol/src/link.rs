use rill_engine::tracers::tracer::{self, ControlReceiver, ControlSender};
use rill_protocol::flow::core;

/// A link for listening events from a `Recorder`.
pub struct Link<T: core::Flow> {
    tx: ControlSender<T>,
    rx: ControlReceiver<T>,
}

impl<T: core::Flow> Link<T> {
    pub fn new() -> Self {
        let (tx, rx) = tracer::channel();
        Self { tx, rx }
    }

    /// Clones a sender.
    pub fn sender(&self) -> ControlSender<T> {
        self.tx.clone()
    }

    /// Takes a receiver.
    pub fn receiver(self) -> ControlReceiver<T> {
        self.rx
    }
}

/*
impl<T: core::Flow> From<Link<T>> for ControlReceiver<T> {
    fn from(link: Link<T>) -> Self {
        link.rx
    }
}
*/
