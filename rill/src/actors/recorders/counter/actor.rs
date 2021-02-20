use crate::actors::worker::{RillWorker, RillWorkerLink};
use crate::tracers::counter::CounterDelta;
use crate::tracers::tracer::{DataEnvelope, DataReceiver};
use anyhow::Error;
use async_trait::async_trait;
use futures::channel::mpsc;
use meio::prelude::{Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::provider::ProviderReqId;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
enum RecorderError {
    #[error("no receiver attached")]
    NoReceiver,
}

pub struct CounterRecorder {
    worker: RillWorkerLink,
    // TODO: Change to the specific type receiver
    receiver: Option<DataReceiver<CounterDelta>>,
    subscribers: HashSet<ProviderReqId>,
    snapshot: u64,
}

impl CounterRecorder {
    pub fn new(worker: RillWorkerLink, rx: DataReceiver<CounterDelta>) -> Self {
        Self {
            worker,
            receiver: Some(rx),
            subscribers: HashSet::new(),
            snapshot: 0,
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
        for envelope in chunk {}
        // TODO: Maintain a snapshot
        // TODO: Send update to subscribers
        Ok(())
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
