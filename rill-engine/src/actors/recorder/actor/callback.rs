use super::Recorder;
use crate::tracers::tracer::BoxedCallback;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Context, IdOf, LiteTask, TaskEliminated, TaskError};
use rill_protocol::flow::core::{self, ActionEnvelope, Activity};
use rill_protocol::io::provider::{Description, ProviderReqId};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

impl<T: core::Flow> Recorder<T> {
    pub(super) fn spawn_callback_worker(&mut self, ctx: &mut Context<Self>) {
        let (tx, rx) = mpsc::channel(64);
        if let Some(callback) = self.callback.callback.take() {
            let worker = CallbackWorker {
                description: self.description.clone(),
                active_connections: HashSet::new(),
                callback,
                receiver: rx,
            };
            self.callback.sender = Some(tx);
            ctx.spawn_task(worker, (), ());
        }
    }

    pub(super) async fn send_activity(&mut self, origin: ProviderReqId, activity: Activity<T>) {
        if let Some(sender) = self.callback.sender.as_mut() {
            let envelope = ActionEnvelope { origin, activity };
            if let Err(err) = sender.send(envelope).await {
                log::error!("Can't send action to a callback worker: {:?}", err);
            }
        }
    }
}

pub(crate) struct CallbackHolder<T: core::Flow> {
    pub callback: Option<BoxedCallback<T>>,
    pub sender: Option<mpsc::Sender<ActionEnvelope<T>>>,
}

pub struct CallbackWorker<T: core::Flow> {
    description: Arc<Description>,
    active_connections: HashSet<ProviderReqId>,
    // TODO: Count connected to call lifetime methods of the callback
    callback: BoxedCallback<T>,
    receiver: mpsc::Receiver<ActionEnvelope<T>>,
}

#[async_trait]
impl<T: core::Flow> LiteTask for CallbackWorker<T> {
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        while let Some(envelope) = self.receiver.recv().await {
            match envelope.activity {
                Activity::Connected => {
                    let awake = self.active_connections.is_empty();
                    self.active_connections.insert(envelope.origin);
                    if awake {
                        self.callback.awake().await;
                    }
                }
                Activity::Disconnected => {
                    self.active_connections.remove(&envelope.origin);
                    let suspend = self.active_connections.is_empty();
                    if suspend {
                        self.callback.suspend().await;
                    }
                }
                Activity::Action(_) => {}
            }
            let res = self
                .callback
                .handle_activity(envelope.origin, envelope.activity)
                .await;
            if let Err(err) = res {
                log::error!(
                    "Callback of a tracer {} failed: {}",
                    self.description.path,
                    err
                );
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
