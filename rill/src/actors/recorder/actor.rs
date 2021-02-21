use super::link;
use crate::actors::worker::{RillWorker, RillWorkerLink};
use crate::tracers::counter::CounterDelta;
use crate::tracers::tracer::{DataEnvelope, DataReceiver, TracerEvent};
use anyhow::Error;
use async_trait::async_trait;
use futures::channel::mpsc;
use meio::prelude::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::provider::{
    Description, Direction, ProviderReqId, RillData, RillEvent, RillProtocol, RillToServer,
    Timestamp,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
enum RecorderError {
    #[error("no receiver attached")]
    NoReceiver,
}

pub struct Recorder<T: TracerEvent> {
    description: Arc<Description>,
    worker: RillWorkerLink,
    // TODO: Change to the specific type receiver
    receiver: Option<DataReceiver<T>>,
    subscribers: HashSet<ProviderReqId>,
    last_update: Option<Timestamp>,
    snapshot: T::Snapshot,
}

impl<T: TracerEvent> Recorder<T> {
    pub fn new(description: Arc<Description>, worker: RillWorkerLink, rx: DataReceiver<T>) -> Self {
        Self {
            description,
            worker,
            receiver: Some(rx),
            subscribers: HashSet::new(),
            last_update: None,
            snapshot: T::Snapshot::default(),
        }
    }

    fn get_event(&self) -> Option<RillEvent> {
        self.last_update.clone().map(|timestamp| {
            let data = T::to_data(&self.snapshot);
            RillEvent { timestamp, data }
        })
    }

    fn get_direction(&self) -> Direction<RillProtocol> {
        Direction::from(&self.subscribers)
    }
}

impl<T: TracerEvent> Actor for Recorder<T> {
    type GroupBy = ();
}

#[async_trait]
impl<T: TracerEvent> StartedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let rx = self.receiver.take().ok_or(RecorderError::NoReceiver)?;
        ctx.attach(rx, ());
        Ok(())
    }
}

#[async_trait]
impl<T: TracerEvent> InterruptedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: TracerEvent> Consumer<DataEnvelope<T>> for Recorder<T> {
    fn stream_group(&self) -> Self::GroupBy {
        ()
    }

    async fn handle(
        &mut self,
        chunk: Vec<DataEnvelope<T>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        for envelope in chunk {
            let DataEnvelope::Event { data, system_time } = envelope;
            data.aggregate(&mut self.snapshot);
            // TODO: Error allowed here?
            let timestamp = system_time.duration_since(SystemTime::UNIX_EPOCH)?.into();
            self.last_update = Some(timestamp);
        }
        if !self.subscribers.is_empty() {
            let event = self.get_event();
            if let Some(event) = event {
                let response = RillToServer::Data { event };
                let direction = self.get_direction();
                self.worker.send_response(direction, response).await?;
            }
        }
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
impl<T: TracerEvent> ActionHandler<link::ControlStream> for Recorder<T> {
    async fn handle(
        &mut self,
        msg: link::ControlStream,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            let id = msg.direct_id;
            log::info!(
                "Switch stream '{}' for {:?} to {:?}",
                self.description.path,
                msg.direct_id,
                msg.active
            );
            // TODO: Fix logs
            if msg.active {
                if self.subscribers.insert(id) {
                    let snapshot = self.get_event();
                    let response = RillToServer::BeginStream { snapshot };
                    let direction = Direction::from(msg.direct_id);
                    self.worker.send_response(direction, response).await?;
                } else {
                    log::warn!("Attempt to subscribe twice for <path> with id: {:?}", id);
                }
            } else {
                if self.subscribers.remove(&id) {
                    let response = RillToServer::EndStream;
                    let direction = Direction::from(msg.direct_id);
                    self.worker.send_response(direction, response).await?;
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
