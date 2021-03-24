use super::link;
use crate::actors::client_session::PROVIDER;
use crate::actors::router::Router;
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
use rill_client::actors::broadcaster::BroadcasterLinkForProvider;
use rill_protocol::io::client::{ClientReqId, ClientResponse};
use rill_protocol::io::provider::{
    EntryId, PathAction, ProviderProtocol, ProviderReqId, ProviderToServer, ServerToProvider,
};
use rill_protocol::io::transport::{Direction, Envelope, WideEnvelope};
use typed_slab::TypedSlab;

pub struct ProviderSession {
    /*
    tracer: EntryTracer,
    tracer_record: Option<ProviderRecord>,
    */
    handler: WsHandler<ProviderProtocol>,
    registered: Option<EntryId>,
    exporter: BroadcasterLinkForProvider,

    directions: TypedSlab<ProviderReqId, ClientRule>,
}

enum ClientRule {
    Forward {
        sender: link::ClientSender,
        req_id: ClientReqId,
    },
    DropTillEnd,
}

impl ProviderSession {
    pub fn new(handler: WsHandler<ProviderProtocol>, exporter: BroadcasterLinkForProvider) -> Self {
        Self {
            handler,
            registered: None,
            exporter,

            directions: TypedSlab::new(),
        }
    }

    fn send_request(&mut self, direct_id: ProviderReqId, data: ServerToProvider) {
        let envelope = Envelope { direct_id, data };
        //log::trace!("Sending request to the server: {:?}", envelope);
        self.handler.send(envelope);
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        if self.registered.take().is_some() {
            self.exporter.session_detached().await.ok();
        }
        ctx.shutdown();
    }
}

#[async_trait]
impl Actor for ProviderSession {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Router> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, (), ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Router> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
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

/*
impl ProviderSession {
    async fn distribute_data(
        &mut self,
        direction: Direction<ProviderProtocol>,
        event: RillEvent,
    ) -> Result<(), Error> {
        if let Direction::Direct(direct_id) = direction {
            let path = self.paths.get(&direct_id);
            if let Some(path) = path.cloned() {
                if let Err(err) = self.exporter.data_received(path, event).await {
                    log::error!("Can't send data item to the exporter: {}", err);
                }
            } else {
                log::error!(
                    "Unknown direction {:?} of the incoming data {:?}",
                    direct_id,
                    event
                );
            }
        } else {
            log::error!(
                "Not supported direction {:?} of the incoming data {:?}",
                direction,
                event
            );
        }
        Ok(())
    }
}
*/

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
                match rule {
                    ClientRule::Forward { req_id, sender } => {
                        let envelope = WideEnvelope {
                            direction: (*req_id).into(),
                            data: resp.clone(),
                        };
                        sender.send(envelope);
                    }
                    ClientRule::DropTillEnd => {
                        log::trace!(
                            "Drop the message since the client unsubscribed from the stream."
                        );
                    }
                }
            }
        }
    }

    fn send_done_marker_for(&mut self, rule: ClientRule) {
        if let ClientRule::Forward { sender, req_id } = rule {
            let data = ClientResponse::Done;
            let envelope = WideEnvelope {
                direction: req_id.into(),
                data,
            };
            sender.send(envelope);
        }
        // Else `Stream` had finished after unsubscribing.
        // Do nothing, because `End` notification sent already.
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
        //log::trace!("Provider incoming message: {:?}", msg);
        match msg.0.data {
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
                    if let Some(rule) = self.directions.remove(*direct_id) {
                        self.send_done_marker_for(rule);
                    }
                }
            }
            /*
            ProviderToServer::Declare { entry_id } => {
                ctx.not_terminating()?;
                if self.registered.is_none() {
                    // Attach the provider
                    *PROVIDER.lock().await = Some(ctx.address().link());
                    self.exporter.session_attached(entry_id.clone()).await?;
                    self.registered = Some(entry_id);
                    // Describe paths
                    let msg = ServerToProvider::Describe { active: true };
                    self.send_request(0.into(), msg);
                } else {
                    log::error!(
                        "Attempt to register a second provider instead of {:?}",
                        self.registered
                    );
                }
            }
            ProviderToServer::Description { list } => {
                log::trace!("Paths available: {:?}", list);
                for description in list {
                    if let Err(err) = self.exporter.path_declared(description).await {
                        log::error!("Can't notify exporter about a new path: {}", err);
                    }
                }
            }
            */
            other => {
                log::warn!("Message {:?} not supported yet.", other);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<link::SubscribeToPath> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::SubscribeToPath,
        _ctx: &mut Context<Self>,
    ) -> Result<ProviderReqId, Error> {
        log::info!("Subscribing to {}", msg.path);
        let rule = ClientRule::Forward {
            sender: msg.sender,
            req_id: msg.direct_id,
        };
        let direct_id = self.directions.insert(rule);

        let action = PathAction::ControlStream { active: true };
        let request = ServerToProvider {
            path: msg.path,
            action,
        };
        self.send_request(direct_id, request);

        Ok(direct_id)
    }
}

#[async_trait]
impl ActionHandler<link::UnsubscribeFromPath> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::UnsubscribeFromPath,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("Unsubscribing from {}", msg.path);
        // But don't remove it from `directions` and wait for the `EndStream`
        // marker will be received.
        let provider_req_id = msg.direct_id;

        if let Some(rule) = self.directions.get_mut(provider_req_id) {
            let mut term_rule = ClientRule::DropTillEnd;
            std::mem::swap(rule, &mut term_rule);
            self.send_done_marker_for(term_rule);

            let action = PathAction::ControlStream { active: false };
            let request = ServerToProvider {
                path: msg.path,
                action,
            };

            self.send_request(provider_req_id, request);
        }
        Ok(())
    }
}
