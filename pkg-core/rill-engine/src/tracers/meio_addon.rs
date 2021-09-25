//! Adds `meio` integraion to `Tracer`.

use super::tracer::Tracer;
use meio::{Action, ActionHandler, Actor, Address};
use rill_protocol::flow::core::{ActionEnvelope, Flow};

/// Tracer action for `meio` actor.
pub struct TracerAction<T: Flow> {
    envelope: ActionEnvelope<T>,
}

impl<T: Flow> Action for TracerAction<T> {}

impl<T: Flow> Tracer<T> {
    /// Forward `Tracer` events to an `Actor`.
    pub fn forward<A: Actor>(&self, addr: Address<A>)
    where
        A: ActionHandler<TracerAction<T>>,
    {
        self.async_callback(move |envelope| {
            let mut addr = addr.clone();
            async move {
                let msg = TracerAction { envelope };
                addr.act(msg).await
            }
        });
    }
}
