//! Adds `meio` integraion to `Tracer`.

use crate::tracers::tracer::Tracer;
use meio::{Action, ActionHandler, Actor, Context};
use rill_protocol::flow::core::{ActionEnvelope, Flow};

/// Tracer action for `meio` actor.
pub struct TracerAction<T: Flow, Tag = ()> {
    /// Assigned envelope with an `Action`.
    pub envelope: ActionEnvelope<T>,
    /// Assigned tag of the action.
    pub tag: Tag,
}

impl<T: Flow, Tag: Send + 'static> Action for TracerAction<T, Tag> {}

impl<T: Flow> Tracer<T> {
    /// Forward `Tracer` events to an `Actor`.
    pub fn forward<A: Actor, Tag>(&self, tag: Tag, ctx: &mut Context<A>)
    where
        A: ActionHandler<TracerAction<T, Tag>>,
        Tag: Clone + Send + Sync + 'static,
    {
        let addr = ctx.address().clone();
        self.async_callback(move |envelope| {
            let mut addr = addr.clone();
            let tag = tag.clone();
            async move {
                let msg = TracerAction { envelope, tag };
                addr.act(msg).await
            }
        });
    }
}
