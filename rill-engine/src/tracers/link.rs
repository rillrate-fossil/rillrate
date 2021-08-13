//! Link allows to connect to a `Recorder` to listen to incoming actions.

use crate::tracers::tracer::{self, ControlReceiver, ControlSender};
use futures::{Stream, StreamExt};
use rill_protocol::flow::core;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A link for listening events from a `Recorder`.
pub struct Link<T: core::Flow> {
    tx: ControlSender<T>,
    rx: ControlReceiver<T>,
}

impl<T: core::Flow> Link<T> {
    /// Creates a new link.
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

    /// Converts receiver into a stream of actions
    pub fn actions(self) -> impl Stream<Item = T::Action> {
        let stream = UnboundedReceiverStream::new(self.rx);
        stream.filter_map(|envelope| async move { envelope.activity.to_action() })
    }
}

/*
impl<T: core::Flow> From<Link<T>> for ControlReceiver<T> {
    fn from(link: Link<T>) -> Self {
        link.rx
    }
}
*/
