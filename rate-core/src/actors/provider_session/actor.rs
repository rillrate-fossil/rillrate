use super::link;
use crate::actors::client_session::ClientSender;
use crate::actors::router::Router;
use crate::actors::supervisor::Supervisor;
use crate::registry::{Occupied, ProviderEntry, Registry, WasEmpty};
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Context, IdOf, InteractionHandler, InterruptedBy, StartedBy,
    TaskEliminated, TaskError,
};
use meio_connect::{
    server::{WsHandler, WsProcessor},
    TermReason, WsIncoming,
};
use rill_protocol::io::client::{ClientReqId, ClientResponse};
use rill_protocol::io::provider::{
    FlowControl, ProviderProtocol, ProviderReqId, ProviderToServer, RecorderRequest,
    ServerToProvider,
};
use rill_protocol::io::transport::{Direction, Envelope, ServiceEnvelope, WideEnvelope};
use typed_slab::TypedSlab;

/// This gate used to cut active stream imediatelly to
/// avoid flooding of messages if the tracer went out of the control.
struct ClientGate {
    req_id: ClientReqId,
    forward_to: Option<ClientSender>,
    /// Provider send `EndStream`
    drained: bool,
    /// Client unsubscribed
    unsubscribed: bool,
}

impl ClientGate {
    /// Prevents any other messages
    fn stop(&mut self) {
        if let Some(sender) = self.forward_to.take() {
            let data = ClientResponse::Done;
            let envelope = Envelope {
                direct_id: self.req_id,
                data,
            };
            let service_envelope = ServiceEnvelope::Envelope(envelope);
            sender.send(service_envelope);
        }
    }
}

pub struct ProviderSession {
    handler: WsHandler<ProviderProtocol>,
    registry: Registry,
    entry: Option<ProviderEntry>,
    directions: TypedSlab<ProviderReqId, ClientGate>,
    /// Allow to add provider with subpaths if it's duplacated
    duplications_limit: usize,
}

impl ProviderSession {
    pub fn new(handler: WsHandler<ProviderProtocol>, registry: Registry) -> Self {
        Self {
            handler,
            registry,
            entry: None,
            directions: TypedSlab::new(),
            duplications_limit: 10,
        }
    }

    fn send_request(&mut self, direct_id: ProviderReqId, data: ServerToProvider) {
        let envelope = Envelope { direct_id, data };
        self.handler.send(envelope);
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        if let Some(entry) = self.entry.take() {
            if let Err(WasEmpty { path }) = entry.unregister_provider().await {
                log::error!("Can't unergister provider: {}", path);
            }
        }
        ctx.shutdown();
    }
}

#[async_trait]
impl Actor for ProviderSession {
    type GroupBy = ();

    fn name(&self) -> String {
        format!("ProviderSession({})", self.handler.addr())
    }
}

#[async_trait]
impl<T: Supervisor> StartedBy<Router<T>> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, (), ());
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> InterruptedBy<Router<T>> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

impl ProviderSession {
    fn distribute_response(
        &mut self,
        direction: Direction<ProviderProtocol>,
        resp: ClientResponse,
    ) {
        let ids = direction.into_vec();
        // TODO: Send whole batch
        for direct_id in &ids {
            if let Some(rule) = self.directions.get(*direct_id) {
                if let Some(sender) = rule.forward_to.as_ref() {
                    let envelope = Envelope {
                        direct_id: rule.req_id,
                        data: resp.clone(),
                    };
                    let service_envelope = ServiceEnvelope::Envelope(envelope);
                    sender.send(service_envelope);
                } else {
                    log::trace!(
                        "Drop the message since the client unsubscribed from the stream: {:?}",
                        rule.req_id
                    );
                }
            }
        }
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<WideEnvelope<ProviderProtocol, ProviderToServer>>>
    for ProviderSession
{
    async fn handle(
        &mut self,
        msg: WsIncoming<WideEnvelope<ProviderProtocol, ProviderToServer>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match msg.0.data {
            // TODO: Split into streaming part of the protocol
            ProviderToServer::Data { delta } => {
                let resp = ClientResponse::Delta(delta);
                self.distribute_response(msg.0.direction, resp);
            }
            ProviderToServer::State { state } => {
                let resp = ClientResponse::State(state);
                self.distribute_response(msg.0.direction, resp);
            }
            ProviderToServer::EndStream => {
                // `distribute_last_response`
                let ids = msg.0.direction.into_vec();
                for direct_id in &ids {
                    // Provider doesn't have to release it on stream ending, because client can still use it!!!
                    let direct_id = *direct_id;
                    if let Some(rule) = self.directions.get_mut(direct_id) {
                        log::debug!(
                            "REL-ID: {} (CLIENT:{})",
                            usize::from(direct_id),
                            usize::from(rule.req_id)
                        );
                        log::info!(
                            "FORWARD[{} -x {}]",
                            usize::from(rule.req_id),
                            usize::from(direct_id)
                        );
                        rule.stop();
                        rule.drained = true;
                        if rule.unsubscribed {
                            self.directions.remove(direct_id);
                        }
                    } else {
                        log::error!(
                            "Attempt to remove direction from the provider twice for: {:?}",
                            direct_id
                        );
                    }
                }
            }

            // TODO: Split into actoins part of the protocol
            ProviderToServer::Flow { description } => {
                let resp = ClientResponse::Flow(description);
                self.distribute_response(msg.0.direction.clone(), resp);
                let ids = msg.0.direction.into_vec();
                for direct_id in &ids {
                    self.directions.remove(*direct_id);
                }
            }

            // TODO: Move to `wide` part of the procotol
            // TODO: Maybe split WideEnvelope to `Broadcast` and `Unicast` sections.
            // TODO: Or broadcast when there is no items in a list.
            ProviderToServer::Declare { description } => {
                ctx.not_terminating()?;
                let mut path = description.path.clone();
                if self.entry.is_none() {
                    log::info!("Provider connected: {:?}", description);
                    let mut counter = 0;
                    loop {
                        let res = self
                            .registry
                            .register_provider(
                                path.clone(),
                                description.clone(),
                                ctx.address().link(),
                            )
                            .await;
                        match res {
                            Ok(entry) => {
                                self.entry = Some(entry);
                                break;
                            }
                            Err(Occupied { .. }) => {
                                log::error!("Entry {} is already occupied by a provider.", path);
                                if counter <= self.duplications_limit {
                                    counter += 1;
                                    let mut entries: Vec<_> = description.path.clone().into();
                                    entries.push(format!("sub-{}", counter).into());
                                    path = entries.into();
                                    log::warn!("Trying with path: {}", path);
                                    continue;
                                } else {
                                    // TODO: Send an error to the provider
                                    self.graceful_shutdown(ctx).await;
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    log::error!("Provider tried to declare itself twice: {}", path);
                    self.graceful_shutdown(ctx).await;
                }
            }
            ProviderToServer::Error { reason } => {
                log::error!("Request failed with: {}", reason);
                let resp = ClientResponse::Error(reason);
                self.distribute_response(msg.0.direction, resp);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsProcessor<ProviderProtocol, Self>, ()> for ProviderSession {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<ProviderProtocol, Self>>,
        _tag: (),
        _result: Result<TermReason, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<link::SubscribeToPath> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::SubscribeToPath,
        ctx: &mut Context<Self>,
    ) -> Result<link::SubscriptionLink, Error> {
        let path = msg.path;
        log::info!("Subscribing to {}", path);
        let rule = ClientGate {
            forward_to: Some(msg.sender),
            req_id: msg.direct_id,
            drained: false,
            unsubscribed: false,
        };
        let direct_id = self.directions.insert(rule);
        log::debug!(
            "ACQ-ID: {} (CLIENT:{})",
            usize::from(direct_id),
            usize::from(msg.direct_id)
        );
        log::info!(
            "FORWARD[{} -> {}] {}",
            usize::from(msg.direct_id),
            usize::from(direct_id),
            path
        );

        let control = FlowControl::StartStream;
        let request = RecorderRequest::ControlStream(control);
        let request = ServerToProvider {
            path: path.clone(),
            request,
        };
        self.send_request(direct_id, request);

        let link = link::SubscriptionLink {
            address: ctx.address().to_owned(),
            path,
            req_id: direct_id,
        };

        Ok(link)
    }
}

#[async_trait]
impl InteractionHandler<link::UnsubscribeFromPath> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::UnsubscribeFromPath,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("Unsubscribing from {}", msg.path);
        let direct_id = msg.req_id;
        if let Some(rule) = self.directions.get_mut(direct_id) {
            if let Some(_) = rule.forward_to.take() {
                rule.stop();
                rule.unsubscribed = true;
                if !rule.drained {
                    let control = FlowControl::StopStream;
                    let request = RecorderRequest::ControlStream(control);
                    let request = ServerToProvider {
                        path: msg.path,
                        request,
                    };
                    self.send_request(direct_id, request);
                } else {
                    self.directions.remove(direct_id);
                }
                Ok(())
            } else {
                Err(Error::msg("Unsubscribing in progress."))
            }
        } else {
            Err(Error::msg("Never subscribed."))
        }
    }
}

#[async_trait]
impl ActionHandler<link::ActionOnPath> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::ActionOnPath,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path;
        log::info!("Action on {}", path);
        let rule = ClientGate {
            forward_to: Some(msg.sender),
            req_id: msg.direct_id,
            drained: true,
            unsubscribed: true,
        };

        let direct_id = self.directions.insert(rule);
        // TODO: WARNINGS! Ids leaks here!!!
        // It removed only on `Flow` response, but not for
        // `GetSnapshot` or `DoAction`.
        //self.directions.remove(direct_id);

        let request = RecorderRequest::Action(msg.action);
        let request = ServerToProvider { path, request };
        self.send_request(direct_id, request);
        Ok(())
    }
}
