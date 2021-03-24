use super::link;
use crate::actors::worker::{RillSender, RillWorker};
use crate::tracers::tracer::{DataEnvelope, TracerDescription, TracerMode};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{ActionHandler, Actor, Consumer, Context, InteractionHandler, InterruptedBy, StartedBy};
use rill_protocol::flow::data;
use rill_protocol::io::provider::{
    PackedFlow, PackedState, PathAction, ProviderProtocol, ProviderReqId, ProviderToServer,
};
use rill_protocol::io::transport::Direction;
use std::collections::HashSet;
use std::sync::{Arc, Weak};

pub(crate) struct Recorder<T: data::Flow> {
    description: Arc<TracerDescription<T>>,
    sender: RillSender,
    mode: TracerMode<T>,
    subscribers: HashSet<ProviderReqId>,
}

impl<T: data::Flow> Recorder<T> {
    pub fn new(
        description: Arc<TracerDescription<T>>,
        sender: RillSender,
        mode: TracerMode<T>,
    ) -> Self {
        Self {
            description,
            sender,
            mode,
            subscribers: HashSet::new(),
        }
    }

    fn get_direction(&self) -> Direction<ProviderProtocol> {
        Direction::from(&self.subscribers)
    }
}

impl<T: data::Flow> Actor for Recorder<T> {
    type GroupBy = ();

    fn name(&self) -> String {
        format!("Recorder({})", &self.description.path)
    }
}

#[async_trait]
impl<T: data::Flow> StartedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        match &mut self.mode {
            TracerMode::Push { receiver, .. } => {
                let rx = receiver
                    .take()
                    .expect("tracer hasn't attached receiver")
                    .ready_chunks(32);
                ctx.attach(rx, (), ());
                Ok(())
            }
            TracerMode::Pull { interval, .. } => {
                let heartbeat = HeartBeat::new(*interval, ctx.address().clone());
                let _task = ctx.spawn_task(heartbeat, (), ());
                // Waiting for the subscribers to spawn a heartbeat activity
                Ok(())
            }
        }
    }
}

#[async_trait]
impl<T: data::Flow> InterruptedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: data::Flow> Consumer<Vec<DataEnvelope<T>>> for Recorder<T> {
    async fn handle(
        &mut self,
        chunk: Vec<DataEnvelope<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let mut delta = Vec::new();
        for envelope in chunk.into_iter() {
            let event = envelope.into_inner();
            delta.push(event);
        }
        if !self.subscribers.is_empty() {
            let response = ProviderToServer::Data {
                // TODO: Use a reference here? (no clone?)
                delta: T::pack_delta(&delta)?,
            };
            let direction = self.get_direction();
            self.sender.response(direction, response);
        }
        if let TracerMode::Push { state, .. } = &mut self.mode {
            for event in delta {
                self.description.metric.apply(state, event);
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
impl<T: data::Flow> OnTick for Recorder<T> {
    async fn tick(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        if !self.subscribers.is_empty() {
            match &self.mode {
                TracerMode::Pull { state, .. } => {
                    if let Some(state) = Weak::upgrade(state) {
                        let state = state
                            .lock()
                            .map_err(|_| Error::msg("Can't lock state to send a state."))?
                            .clone();
                        let response = ProviderToServer::State {
                            state: T::pack_state(&state)?,
                        };
                        let direction = self.get_direction();
                        self.sender.response(direction, response);
                    } else {
                        // TODO: Consider to use a `channel` to get informed if the stream was
                        // closed.
                        ctx.shutdown();
                    }
                }
                TracerMode::Push { .. } => {
                    log::error!("Pulling tick received for the push mode.");
                }
            }
        }
        Ok(())
    }

    async fn done(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: data::Flow> ActionHandler<link::DoPathAction> for Recorder<T> {
    async fn handle(
        &mut self,
        msg: link::DoPathAction,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            let id = msg.direct_id;
            match msg.action {
                PathAction::ControlStream { active } => {
                    log::info!(
                        "Switch stream '{}' for {:?} to {:?}",
                        self.description.path,
                        id,
                        active
                    );
                    // TODO: Fix logs
                    #[allow(clippy::collapsible_if)]
                    if active {
                        if self.subscribers.insert(id) {
                            if let TracerMode::Push { state, .. } = &self.mode {
                                let state = T::pack_state(state)?;
                                let response = ProviderToServer::State { state };
                                let direction = Direction::from(msg.direct_id);
                                self.sender.response(direction, response);
                            }
                            // TODO: Send the first state for pull as well
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
                    /*
                    if self.subscribers.is_empty() {
                        // TODO: Terminate `HeartBeat`
                    } else {
                        // TODO: Spawn a `HeartBeat` state extractor
                    } PathAction
                    */
                }
                PathAction::GetSnapshot => {}
                PathAction::GetFlow => {}
            }
        } else {
            // TODO: Send `EndStream` immediately and maybe `BeginStream` before
        }
        Ok(())
    }
}

#[async_trait]
impl<T: data::Flow> ActionHandler<link::ConnectionChanged> for Recorder<T> {
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

#[async_trait]
impl<T: data::Flow> InteractionHandler<link::FetchInfo> for Recorder<T> {
    async fn handle(
        &mut self,
        msg: link::FetchInfo,
        _ctx: &mut Context<Self>,
    ) -> Result<(PackedFlow, Option<PackedState>), Error> {
        todo!()
    }
}
