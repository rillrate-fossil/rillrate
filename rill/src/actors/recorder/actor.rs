use super::link;
use crate::actors::worker::{RillSender, RillWorker};
use crate::tracers::tracer::{DataEnvelope, DataReceiver, TracerEvent, TracerState};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::provider::{
    Description, Direction, ProviderReqId, RillEvent, RillProtocol, RillToServer,
};
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
enum RecorderError {
    #[error("no receiver attached")]
    NoReceiver,
}

pub(crate) struct Recorder<T: TracerEvent> {
    description: Arc<Description>,
    sender: RillSender,
    // TODO: Change to the specific type receiver
    receiver: Option<DataReceiver<T>>,
    subscribers: HashSet<ProviderReqId>,
    state: T::State,
}

impl<T: TracerEvent> Recorder<T> {
    pub fn new(description: Arc<Description>, sender: RillSender, rx: DataReceiver<T>) -> Self {
        Self {
            description,
            sender,
            receiver: Some(rx),
            subscribers: HashSet::new(),
            state: T::State::default(),
        }
    }

    fn get_snapshot(&self) -> Vec<RillEvent> {
        self.state.make_snapshot()
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
        // TODO: Use something here?
    }

    async fn handle(
        &mut self,
        chunk: Vec<DataEnvelope<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        /*
        let mut event = None;
        for envelope in chunk {
            let DataEnvelope::Event { data, system_time } = envelope;
            // TODO: Error not allowed here
            let timestamp = system_time.duration_since(SystemTime::UNIX_EPOCH)?.into();
            event = self.state.aggregate(data, timestamp);
        }
        */
        let event = self.state.aggregate(chunk);
        // TODO: ^ Realy aggregate data and send once per loop
        if !self.subscribers.is_empty() {
            if let Some(event) = event {
                let response = RillToServer::Data {
                    event: event.to_owned(),
                };
                let direction = self.get_direction();
                self.sender.response(direction, response);
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
            #[allow(clippy::collapsible_if)]
            if msg.active {
                if self.subscribers.insert(id) {
                    let snapshot = self.get_snapshot();
                    let response = RillToServer::BeginStream { snapshot };
                    let direction = Direction::from(msg.direct_id);
                    self.sender.response(direction, response);
                } else {
                    log::warn!("Attempt to subscribe twice for <path> with id: {:?}", id);
                }
            } else {
                if self.subscribers.remove(&id) {
                    let response = RillToServer::EndStream;
                    let direction = Direction::from(msg.direct_id);
                    self.sender.response(direction, response);
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

#[async_trait]
impl<T: TracerEvent> ActionHandler<link::ConnectionChanged> for Recorder<T> {
    async fn handle(
        &mut self,
        msg: link::ConnectionChanged,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        use link::ConnectionChanged::*;
        match msg {
            Connected { sender } => {
                self.sender = sender;
            }
            Disconnected => {
                self.sender.reset();
                self.subscribers.clear();
            }
        }
        Ok(())
    }
}
