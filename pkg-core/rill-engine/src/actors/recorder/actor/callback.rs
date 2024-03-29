use super::{Group, Recorder};
use crate::tracers::tracer::{ActionReceiver, BoxedCallback};
use anyhow::Error;
use async_trait::async_trait;
use meio::{Context, IdOf, LiteTask, TaskEliminated, TaskError};
use rill_protocol::flow::core;
use rill_protocol::io::provider::Description;
use std::sync::Arc;
use tokio::sync::mpsc;

impl<T: core::Flow> Recorder<T> {
    pub(super) fn attach_callback(&mut self, callback: BoxedCallback<T>, ctx: &mut Context<Self>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.callback = Some(tx);
        let worker = CallbackWorker {
            description: self.description.clone(),
            receiver: rx,
            callback,
        };
        ctx.spawn_task(worker, (), Group::Callback);
    }

    pub(super) fn detach_callback(&mut self, _ctx: &mut Context<Self>) {
        // The `CallbackWorker` will be closed when drained
        self.callback.take();
        //ctx.terminate_group(Group::Callback);
    }
}

pub struct CallbackWorker<T: core::Flow> {
    description: Arc<Description>,
    receiver: ActionReceiver<T>,
    callback: BoxedCallback<T>,
}

#[async_trait]
impl<T: core::Flow> LiteTask for CallbackWorker<T> {
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        while let Some(envelope) = self.receiver.recv().await {
            if let Err(err) = self.callback.handle_activity(envelope).await {
                log::error!("Callback of {} failed: {}", self.description.path, err);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> TaskEliminated<CallbackWorker<T>, ()> for Recorder<T> {
    async fn handle(
        &mut self,
        _id: IdOf<CallbackWorker<T>>,
        _tag: (),
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}
