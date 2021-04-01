use super::link;
use crate::actors::worker::{RillSender, RillWorker};
use crate::tracers::tracer::{DataEnvelope, TracerDescription, TracerMode};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::flow::data;
use rill_protocol::io::provider::{
    FlowControl, PackedState, ProviderProtocol, ProviderReqId, ProviderToServer, RecorderAction,
    RecorderRequest,
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

    /// `Direction` to all subscribers.
    fn all_subscribers(&self) -> Direction<ProviderProtocol> {
        Direction::from(&self.subscribers)
    }

    fn send_flow(&mut self, direction: Direction<ProviderProtocol>) -> Result<(), Error> {
        let description = self.description.to_description()?;
        let response = ProviderToServer::Flow { description };
        self.sender.response(direction, response);
        Ok(())
    }

    async fn pack_state(&self) -> Result<PackedState, Error> {
        match &self.mode {
            TracerMode::Push { state, .. } => T::pack_state(state),
            TracerMode::Pull { state, .. } => {
                if let Some(state) = Weak::upgrade(state) {
                    let state = state
                        .lock()
                        .map_err(|_| Error::msg("Can't lock state to send a state."))?;
                    T::pack_state(&state)
                } else {
                    Err(Error::msg("Can't upgrade weak reference to the state."))
                }
            }
        }
    }

    async fn send_state(&mut self, direction: Direction<ProviderProtocol>) -> Result<(), Error> {
        let state = self.pack_state().await?;
        let response = ProviderToServer::State { state };
        self.sender.response(direction, response);
        Ok(())
    }

    fn send_end(&mut self, direction: Direction<ProviderProtocol>) {
        let response = ProviderToServer::EndStream;
        self.sender.response(direction, response);
    }

    fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        // No more events will be received after this point.
        self.send_end(self.all_subscribers());
        self.subscribers.clear();
        ctx.shutdown();
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
        self.graceful_shutdown(ctx);
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
                delta: T::pack_delta(&delta)?,
            };
            let direction = self.all_subscribers();
            self.sender.response(direction, response);
        }
        match &mut self.mode {
            TracerMode::Push { state, .. } => {
                for event in delta {
                    self.description.metric.apply(state, event);
                }
            }
            TracerMode::Pull { .. } => {
                log::error!(
                    "Delta received for Pull mode for: {}",
                    self.description.path
                );
            }
        }
        Ok(())
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx);
        Ok(())
    }
}

#[async_trait]
impl<T: data::Flow> OnTick for Recorder<T> {
    async fn tick(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        if !self.subscribers.is_empty() {
            match &self.mode {
                TracerMode::Pull { .. } => {
                    let direction = self.all_subscribers();
                    if let Err(_err) = self.send_state(direction).await {
                        // Stop the actor if the data can't be pulled.
                        self.graceful_shutdown(ctx);
                    }
                }
                TracerMode::Push { .. } => {
                    log::error!(
                        "Pulling tick received for the push mode for: {}",
                        self.description.path
                    );
                }
            }
        }
        Ok(())
    }

    async fn done(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        // This can happen only if the `InterruptedBy` handler called and
        // all shutdown routine (sending `End` responses) was already performed.
        Ok(())
    }
}

#[async_trait]
impl<T: data::Flow> ActionHandler<link::DoRecorderRequest> for Recorder<T> {
    async fn handle(
        &mut self,
        msg: link::DoRecorderRequest,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            let id = msg.direct_id;
            match msg.request {
                RecorderRequest::ControlStream(control) => {
                    log::info!(
                        "Switch stream '{}' for {:?} to {:?}",
                        self.description.path,
                        id,
                        control,
                    );
                    match control {
                        FlowControl::StartStream => {
                            if self.subscribers.insert(id) {
                                self.send_state(id.into()).await?;
                            } else {
                                log::warn!(
                                    "Attempt to subscribe twice for <path> with id: {:?}",
                                    id
                                );
                            }
                        }
                        FlowControl::StopStream => {
                            if self.subscribers.remove(&id) {
                                self.send_end(id.into());
                            } else {
                                log::warn!("Can't remove subscriber of <path> by id: {:?}", id);
                            }
                        }
                    }
                    /*
                    if self.subscribers.is_empty() {
                        // TODO: Terminate `HeartBeat`
                    } else {
                        // TODO: Spawn a `HeartBeat` state extractor
                    }
                    */
                }
                RecorderRequest::Action(action) => match action {
                    RecorderAction::GetSnapshot => {
                        self.send_state(id.into()).await?;
                    }
                    RecorderAction::GetFlow => {
                        self.send_flow(id.into())?;
                    }
                },
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
