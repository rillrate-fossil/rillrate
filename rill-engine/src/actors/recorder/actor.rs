use super::link;
use crate::actors::worker::{RillSender, RillWorker};
use crate::tracers::tracer::{time_to_ts, DataEnvelope, TracerMode};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy, TaskAddress};
use rill_protocol::flow::core::{self, TimedEvent, ToEvent};
use rill_protocol::io::provider::{
    Description, FlowControl, PackedState, ProviderProtocol, ProviderReqId, ProviderToServer,
    RecorderAction, RecorderRequest,
};
use rill_protocol::io::transport::Direction;
use std::collections::HashSet;
use std::sync::{Arc, Weak};
use std::time::Instant;

pub(crate) struct Recorder<T: core::Flow> {
    drained_at: Option<Instant>,
    description: Arc<Description>,
    sender: RillSender,
    mode: TracerMode<T>,
    subscribers: HashSet<ProviderReqId>,
    heartbeat: Option<TaskAddress<HeartBeat>>,
}

impl<T: core::Flow> Recorder<T> {
    pub fn new(description: Arc<Description>, sender: RillSender, mode: TracerMode<T>) -> Self {
        Self {
            drained_at: None,
            description,
            sender,
            mode,
            subscribers: HashSet::new(),
            heartbeat: None,
        }
    }

    /// `Direction` to all subscribers.
    fn all_subscribers(&self) -> Direction<ProviderProtocol> {
        Direction::from(&self.subscribers)
    }

    fn send_flow(&mut self, direction: Direction<ProviderProtocol>) {
        let description = Description::clone(&self.description);
        let response = ProviderToServer::Flow { description };
        self.sender.response(direction, response);
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

    /// No more messages from the flow will be received (no more messages expected).
    fn drained(&mut self, ctx: &mut Context<Self>) {
        // No more events will be received after this point.
        self.drained_at = Some(Instant::now());
        if self.subscribers.is_empty() {
            // There are subscribers. Wait when they will be terminated.
            ctx.shutdown();
        } else {
            self.send_end(self.all_subscribers());
            // TODO: Start a heartbeat to limit waiting for close events
        }
    }
}

// TODO: Use `strum` here
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Group {
    HeartBeat,
    Receiver,
}

impl<T: core::Flow> Actor for Recorder<T> {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("Recorder({})", &self.description.path)
    }
}

#[async_trait]
impl<T: core::Flow> StartedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::HeartBeat, Group::Receiver]);
        match &mut self.mode {
            TracerMode::Push { receiver, .. } => {
                let rx = receiver
                    .take()
                    .expect("tracer hasn't attached receiver")
                    .ready_chunks(32);
                ctx.attach(rx, (), Group::Receiver);
                Ok(())
            }
            TracerMode::Pull { interval, .. } => {
                let heartbeat = HeartBeat::new(*interval, ctx.address().clone());
                let task = ctx.spawn_task(heartbeat, (), Group::HeartBeat);
                self.heartbeat = Some(task);
                // Waiting for the subscribers to spawn a heartbeat activity
                Ok(())
            }
        }
    }
}

#[async_trait]
impl<T: core::Flow> InterruptedBy<RillWorker> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

impl<T: core::Flow> Recorder<T> {
    fn send_delta(&mut self, delta: &[TimedEvent<T::Event>]) -> Result<(), Error> {
        if !self.subscribers.is_empty() {
            let response = ProviderToServer::Data {
                delta: T::pack_delta(&delta)?,
            };
            let direction = self.all_subscribers();
            self.sender.response(direction, response);
        }
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> Consumer<Vec<DataEnvelope<T>>> for Recorder<T> {
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

        self.send_delta(&delta)?;

        match &mut self.mode {
            TracerMode::Push { state, .. } => {
                for event in delta {
                    T::apply(state, event);
                }
            }
            TracerMode::Pull { .. } => {
                log::error!("Delta received in pull mode for: {}", self.description.path);
            }
        }
        Ok(())
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.drained(ctx);
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> OnTick for Recorder<T> {
    async fn tick(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        let mut stop = false;
        if !self.subscribers.is_empty() {
            match &self.mode {
                TracerMode::Pull { .. } => {
                    let direction = self.all_subscribers();
                    if let Err(_err) = self.send_state(direction).await {
                        self.drained(ctx);
                        stop = true;
                    }
                }
                TracerMode::Push { .. } => {
                    log::error!(
                        "Pulling tick received in the push mode for: {}",
                        self.description.path
                    );
                }
            }
        }
        if stop {
            let task = self
                .heartbeat
                .take()
                .ok_or_else(|| Error::msg("No heartbeat address"))?;
            task.stop()?;
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
impl<T: core::Flow> ActionHandler<link::DoRecorderRequest> for Recorder<T> {
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
                            if self.drained_at.is_none() {
                                if self.subscribers.insert(id) {
                                    self.send_state(id.into()).await?;
                                } else {
                                    log::warn!(
                                        "Attempt to subscribe twice for <path> with id: {:?}",
                                        id
                                    );
                                }
                                // TODO: Start heartbeat if the first subscriber appeared
                                // for Pull mode only
                            } else {
                                // TODO: Send Error
                            }
                        }
                        FlowControl::StopStream => {
                            if self.subscribers.remove(&id) {
                                self.send_end(id.into());
                            } else {
                                log::warn!("Can't remove subscriber of <path> by id: {:?}", id);
                            }
                            if self.drained_at.is_some() {
                                if self.subscribers.is_empty() {
                                    ctx.shutdown();
                                }
                            }
                            // TODO: Stop heartbeat here
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
                RecorderRequest::Action(action) => {
                    match action {
                        RecorderAction::GetSnapshot => {
                            self.send_state(id.into()).await?;
                        }
                        RecorderAction::GetFlow => {
                            self.send_flow(id.into());
                        }
                        RecorderAction::DoAction(data) => {
                            if self.drained_at.is_none() {
                                let action = T::unpack_action(&data)?;
                                match &mut self.mode {
                                    TracerMode::Push {
                                        state,
                                        control_sender,
                                        ..
                                    } => {
                                        // TODO: Track errors and send them back to the client
                                        let opt_event = action.to_event();
                                        let timestamp = time_to_ts(None)?;
                                        if let Err(err) = control_sender.send(action) {
                                            log::error!(
                                                "No action listeners in {} watcher: {}",
                                                self.description.path,
                                                err,
                                            );
                                        }
                                        if let Some(event) = opt_event {
                                            let timed_event = TimedEvent { timestamp, event };
                                            T::apply(state, timed_event.clone());
                                            self.send_delta(&[timed_event])?;
                                        }
                                    }
                                    TracerMode::Pull { .. } => {
                                        log::error!(
                                            "Do action request in the pull mode of {}",
                                            self.description.path
                                        );
                                    }
                                }
                            } else {
                                // TODO: Send Error?
                            }
                        }
                    }
                }
            }
        } else {
            // TODO: Send `EndStream` immediately and maybe `BeginStream` before
        }
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> ActionHandler<link::ConnectionChanged> for Recorder<T> {
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
