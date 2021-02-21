use super::link;
use crate::actors::worker::{RillWorker, RillWorkerLink};
use crate::tracers::counter::CounterDelta;
use crate::tracers::tracer::{DataEnvelope, DataReceiver};
use anyhow::Error;
use async_trait::async_trait;
use futures::channel::mpsc;
use meio::prelude::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::provider::ProviderReqId;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
enum RecorderError {
    #[error("no receiver attached")]
    NoReceiver,
}

pub struct CounterRecorder {
    // TODO: Keep path here
    worker: RillWorkerLink,
    // TODO: Change to the specific type receiver
    receiver: Option<DataReceiver<CounterDelta>>,
    subscribers: HashSet<ProviderReqId>,
    counter: f64,
}

impl CounterRecorder {
    pub fn new(worker: RillWorkerLink, rx: DataReceiver<CounterDelta>) -> Self {
        Self {
            worker,
            receiver: Some(rx),
            subscribers: HashSet::new(),
            counter: 0.0,
        }
    }
}

impl Actor for CounterRecorder {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillWorker> for CounterRecorder {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let rx = self.receiver.take().ok_or(RecorderError::NoReceiver)?;
        ctx.attach(rx, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillWorker> for CounterRecorder {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Consumer<DataEnvelope<CounterDelta>> for CounterRecorder {
    fn stream_group(&self) -> Self::GroupBy {
        ()
    }

    async fn handle(
        &mut self,
        chunk: Vec<DataEnvelope<CounterDelta>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        for envelope in chunk {
            let DataEnvelope::Event { data, .. } = envelope;
            let CounterDelta::Increment(delta) = data;
            self.counter += delta;
        }
        // TODO: Send update to subscribers
        Ok(())
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Send `EndStream` to all subscribers
        // TODO: Remove all subscribers
        ctx.shutdown();
        // TODO: Maybe send an instant `StopList` event and avoid shutdown for a while
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::ControlStream> for CounterRecorder {
    async fn handle(
        &mut self,
        msg: link::ControlStream,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            // TODO: Fix logs
            let id = msg.direct_id;
            if msg.active {
                if self.subscribers.insert(id) {
                    // TODO: Send `BeginStream` with a snapshot
                } else {
                    log::warn!("Attempt to subscribe twice for <path> with id: {:?}", id);
                }
            } else {
                if self.subscribers.remove(&id) {
                    // TODO: Send `EndStream`
                } else {
                    log::warn!("Can't remove subscriber of <path> by id: {:?}", id);
                }
            }
        } else {
            // TODO: Send `EndStream` immediately and maybe `BeginStream` before
        }
        Ok(())
    }
}
