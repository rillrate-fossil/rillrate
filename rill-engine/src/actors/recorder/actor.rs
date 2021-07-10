pub mod link;

use crate::actors::connector::{RillConnector, RillSender};
use crate::tracers::tracer::{EventEnvelope, TracerMode};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{ActionHandler, Actor, Consumer, Context, InterruptedBy, StartedBy};
use rill_protocol::flow::core::{self, ActionEnvelope, Activity};
use rill_protocol::io::provider::{
    Description, FlowControl, PackedState, ProviderProtocol, ProviderReqId, ProviderToServer,
    RecorderAction, RecorderRequest,
};
use rill_protocol::io::transport::Direction;
use std::collections::HashSet;
use std::sync::{Arc, Weak};

pub(crate) struct Recorder<T: core::Flow> {
    description: Arc<Description>,
    sender: RillSender,
    mode: TracerMode<T>,
    subscribers: HashSet<ProviderReqId>,
}

impl<T: core::Flow> Recorder<T> {
    pub fn new(description: Arc<Description>, sender: RillSender, mode: TracerMode<T>) -> Self {
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

    fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        //log::warn!("Terminating: {}", self.name());
        // No more events will be received after this point.
        self.send_end(self.all_subscribers());
        self.subscribers.clear();
        ctx.shutdown();
    }
}

impl<T: core::Flow> Actor for Recorder<T> {
    type GroupBy = ();

    fn name(&self) -> String {
        format!("Recorder({})", &self.description.path)
    }
}

#[async_trait]
impl<T: core::Flow> StartedBy<RillConnector> for Recorder<T> {
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
impl<T: core::Flow> InterruptedBy<RillConnector> for Recorder<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx);
        Ok(())
    }
}

impl<T: core::Flow> Recorder<T> {
    fn send_event(
        &mut self,
        direction: Direction<ProviderProtocol>,
        event: &T::Event,
    ) -> Result<(), Error> {
        if !self.subscribers.is_empty() {
            let response = ProviderToServer::Data {
                delta: T::pack_event(event)?,
            };
            self.sender.response(direction, response);
        }
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> Consumer<Vec<EventEnvelope<T>>> for Recorder<T> {
    async fn handle(
        &mut self,
        chunk: Vec<EventEnvelope<T>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            for envelope in chunk.into_iter() {
                let EventEnvelope {
                    mut direction,
                    event,
                } = envelope;
                let apply;
                if let Some(dir) = direction.take() {
                    // Direct events not applied to the state
                    apply = false;
                    match dir {
                        Direction::Direct(direct_id) => {
                            if self.subscribers.contains(&direct_id) {
                                direction = Some(Direction::Direct(direct_id));
                            }
                        }
                        Direction::Multicast(directions) => {
                            let directions: HashSet<_> = self
                                .subscribers
                                .intersection(&directions)
                                .cloned()
                                .collect();
                            if !directions.is_empty() {
                                direction = Some(Direction::Multicast(directions));
                            }
                        }
                        Direction::Broadcast => {
                            direction = Some(Direction::Broadcast);
                        }
                    }
                } else {
                    // Multicast the event and apply it to the state
                    apply = true;
                    direction = Some(self.all_subscribers());
                }
                if let Some(direction) = direction {
                    self.send_event(direction, &event)?;
                }
                // Apply even if it has no subscribers
                if apply {
                    match &mut self.mode {
                        TracerMode::Push { state, .. } => {
                            T::apply(state, event);
                        }
                        TracerMode::Pull { .. } => {
                            log::error!(
                                "Delta received in pull mode for: {}",
                                self.description.path
                            );
                        }
                    }
                }
            }
        } else {
            // TODO: Use `ConsumerHandle` to abort the stream (or interrupt with `stop` call).
        }
        Ok(())
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx);
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> OnTick for Recorder<T> {
    async fn tick(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        if !self.subscribers.is_empty() && !ctx.is_terminating() {
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
                        "Pulling tick received in the push mode for: {}",
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

impl<T: core::Flow> Recorder<T> {
    fn send_activity(&mut self, origin: ProviderReqId, activity: Activity<T>) {
        match &mut self.mode {
            TracerMode::Push {
                control_sender: Some(sender),
                ..
            } => {
                let envelope = ActionEnvelope { origin, activity };
                // TODO: Track errors and send them back to the client?
                if let Err(err) = sender.unbounded_send(envelope) {
                    log::error!(
                        "No activity listeners in {} watcher: {}",
                        self.description.path,
                        err,
                    );
                }
            }
            TracerMode::Push {
                control_sender: None,
                ..
            } => {
                log::error!(
                    "Push sender doesn't support control actions for {}",
                    self.description.path
                );
            }
            TracerMode::Pull { .. } => {
                log::error!(
                    "Do activity request in the pull mode of {}",
                    self.description.path
                );
            }
        }
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
                            if self.subscribers.insert(id) {
                                self.send_state(id.into()).await?;
                                self.send_activity(id, Activity::Connected);
                            } else {
                                log::warn!(
                                    "Attempt to subscribe twice for <path> with id: {:?}",
                                    id
                                );
                            }
                        }
                        FlowControl::StopStream => {
                            if self.subscribers.remove(&id) {
                                self.send_activity(id, Activity::Disconnected);
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
                        self.send_flow(id.into());
                    }
                    RecorderAction::DoAction(data) => {
                        let action = T::unpack_action(&data)?;
                        let activity = Activity::Action(action);
                        self.send_activity(id, activity);
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
