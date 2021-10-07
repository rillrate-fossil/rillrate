//! Extension for `meio`.

use crate::tracers::tracer::Tracer;
use anyhow::Error;
use async_trait::async_trait;
use meio::handlers::{Handler, Priority};
use meio::{Actor, Context};
use rill_protocol::flow::core::{ActionEnvelope, Activity, Flow};

// TODO: Add `custom_act` method the the `Address`.

trait FlowAction: Send + 'static {}

#[async_trait]
trait FlowActionHandler<I: FlowAction>: Actor {
    async fn handle(&mut self, input: I, _ctx: &mut Context<Self>) -> Result<(), Error>;
}

struct FlowActionHandlerImpl<I> {
    input: Option<I>,
}

#[async_trait]
impl<A, I> Handler<A> for FlowActionHandlerImpl<I>
where
    A: FlowActionHandler<I>,
    I: FlowAction,
{
    fn priority(&self) -> Priority {
        Priority::Normal
    }

    async fn handle(&mut self, actor: &mut A, ctx: &mut Context<A>) -> Result<(), Error> {
        let input = self.input.take().expect("action handler called twice");
        actor.handle(input, ctx).await
    }
}

/// Tracer action for `meio` actor.
pub struct TracerAction<T: Flow, Tag = ()> {
    /// Assigned envelope with an `Action`.
    pub envelope: ActionEnvelope<T>,
    /// Assigned tag of the action.
    pub tag: Tag,
}

impl<T, Tag> FlowAction for TracerAction<T, Tag>
where
    T: Flow,
    Tag: Send + 'static,
{
}

/// Handles incoming events.
#[async_trait]
pub trait FlowHandler<T: Flow, Tag>: Actor {
    /// Status events.
    async fn status(&mut self, _activity: Activity, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
    /// Actions.
    async fn action(&mut self, _action: T::Action, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl<A, T, Tag> FlowActionHandler<TracerAction<T, Tag>> for A
where
    A: FlowHandler<T, Tag>,
    T: Flow,
    Tag: Send + 'static,
{
    async fn handle(
        &mut self,
        mut input: TracerAction<T, Tag>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if let Some(action) = input.envelope.action.take() {
            FlowHandler::action(self, action, ctx).await
        } else {
            FlowHandler::status(self, input.envelope.activity, ctx).await
        }
    }
}

impl<T: Flow> Tracer<T> {
    /// Forward `Tracer` events to an `Actor`.
    pub fn forward<A: Actor, Tag>(&self, tag: Tag, ctx: &mut Context<A>)
    where
        A: FlowHandler<T, Tag>,
        Tag: Clone + Send + Sync + 'static,
    {
        let addr = ctx.address().clone();
        self.async_callback(move |envelope| {
            let addr = addr.clone();
            let tag = tag.clone();
            async move {
                let msg = TracerAction { envelope, tag };
                // TODO: Improve this! `send_event` shold accept external actions directly.
                let handler = FlowActionHandlerImpl { input: Some(msg) };
                addr.send_event(handler).await
            }
        });
    }
}
