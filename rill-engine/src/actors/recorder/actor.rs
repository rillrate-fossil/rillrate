use super::link;
use crate::actors::worker::{RillSender, RillWorker};
use crate::tracers::tracer::{DataEnvelope, DataReceiver};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::data::{self, Delta, State};
use rill_protocol::io::provider::{
    Description, ProviderProtocol, ProviderReqId, ProviderToServer, StreamState,
};
use rill_protocol::io::transport::Direction;
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
enum RecorderError {
    #[error("no receiver attached")]
    NoReceiver,
}

pub(crate) struct Recorder<T: data::Event> {
    description: Arc<Description>,
    sender: RillSender,
    // TODO: Change to the specific type receiver
    receiver: Option<DataReceiver<T>>,
    subscribers: HashSet<ProviderReqId>,
    state: T::State,
}

impl<T: data::Event> Recorder<T> {
    pub fn new(description: Arc<Description>, sender: RillSender, rx: DataReceiver<T>) -> Self {
        Self {
            description,
            sender,
            receiver: Some(rx),
            subscribers: HashSet::new(),
            state: T::State::default(),
        }
    }

    fn get_snapshot(&self) -> StreamState {
        self.state.clone().into()
    }

    fn get_direction(&self) -> Direction<ProviderProtocol> {
        Direction::from(&self.subscribers)
    }
}

impl<T: data::Event> Actor for Recorder<T> {
    type GroupBy = ();
}

#[async_trait]
impl<T: data::Event> StartedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let rx = self
            .receiver
            .take()
            .ok_or(RecorderError::NoReceiver)?
            .ready_chunks(32);
        ctx.attach(rx, ());
        Ok(())
    }
}

#[async_trait]
impl<T: data::Event> InterruptedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: data::Event> Consumer<Vec<DataEnvelope<T>>> for Recorder<T> {
    async fn handle(
        &mut self,
        chunk: Vec<DataEnvelope<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let mut delta = T::Delta::default();
        for envelope in chunk.into_iter() {
            let event = envelope.into_inner();
            delta.push(event);
        }
        if !self.subscribers.is_empty() {
            let response = ProviderToServer::Data {
                delta: delta.clone().into(),
            };
            let direction = self.get_direction();
            self.sender.response(direction, response);
        }
        self.state.apply(delta);
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
impl<T: data::Event> ActionHandler<link::ControlStream> for Recorder<T> {
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
                    let state = self.get_snapshot();
                    let response = ProviderToServer::BeginStream { state };
                    let direction = Direction::from(msg.direct_id);
                    self.sender.response(direction, response);
                } else {
                    log::warn!("Attempt to subscribe twice for <path> with id: {:?}", id);
                }
            } else {
                if self.subscribers.remove(&id) {
                    let response = ProviderToServer::EndStream;
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
impl<T: data::Event> ActionHandler<link::ConnectionChanged> for Recorder<T> {
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
