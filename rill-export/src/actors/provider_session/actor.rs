use super::link;
use crate::actors::exporter::ExporterLinkForProvider;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, IdOf, InterruptedBy, StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    server::{WsHandler, WsProcessor},
    TermReason, WsIncoming,
};
use rill_protocol::provider::{
    DirectId, Direction, EntryId, Envelope, Path, ProviderReqId, RillProtocol, RillToProvider,
    RillToServer, WideEnvelope,
};
use std::collections::HashMap;
use typed_slab::TypedSlab;

pub struct ProviderSession {
    handler: WsHandler<RillProtocol>,
    registered: Option<EntryId>,
    exporter: ExporterLinkForProvider,
    counter: usize,
    // TODO: Replace to `TypedSlab`
    paths: HashMap<DirectId<RillProtocol>, Path>,
    active_requests: TypedSlab<ProviderReqId, ()>,
}

impl ProviderSession {
    pub fn new(handler: WsHandler<RillProtocol>, exporter: ExporterLinkForProvider) -> Self {
        Self {
            handler,
            registered: None,
            exporter,
            counter: 0,
            paths: HashMap::new(),
            active_requests: TypedSlab::new(),
        }
    }

    fn send_request(&mut self, data: RillToProvider) {
        self.counter += 1;
        let direct_id = DirectId::from(self.counter);
        if let RillToProvider::ControlStream {
            ref path,
            active: true,
        } = data
        {
            self.paths.insert(direct_id.clone(), path.clone());
        }
        let envelope = Envelope { direct_id, data };
        log::trace!("Sending request to the server: {:?}", envelope);
        self.handler.send(envelope);
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        self.exporter.session_detached().await.ok();
        ctx.shutdown();
    }
}

#[async_trait]
impl Actor for ProviderSession {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Server> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.exporter.session_attached(ctx.address().link()).await?;
        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Server> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsProcessor<RillProtocol, Self>> for ProviderSession {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<RillProtocol, Self>>,
        _result: Result<TermReason, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<WideEnvelope<RillProtocol, RillToServer>>> for ProviderSession {
    async fn handle(
        &mut self,
        msg: WsIncoming<WideEnvelope<RillProtocol, RillToServer>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Provider incoming message: {:?}", msg);
        match msg.0.data {
            RillToServer::Data { timestamp, data } => {
                if let Direction::Direct(direct_id) = msg.0.direction {
                    let path = self.paths.get(&direct_id);
                    if let Some(path) = path.cloned() {
                        if let Err(err) = self.exporter.data_received(path, timestamp, data).await {
                            log::error!("Can't send data item to the exporter: {}", err);
                        }
                    } else {
                        log::error!(
                            "Unknown direction {:?} of the incoing data {:?}",
                            direct_id,
                            data
                        );
                    }
                } else {
                    log::error!(
                        "Not supported direction {:?} of the incoing data {:?}",
                        msg.0.direction,
                        data
                    );
                }
            }
            RillToServer::BeginStream => {}
            RillToServer::EndStream => {}
            RillToServer::Declare { entry_id } => {
                self.registered = Some(entry_id);
                let msg = RillToProvider::Describe { active: true };
                self.send_request(msg);
            }
            RillToServer::Description { list } => {
                log::trace!("Paths available: {:?}", list);
                for description in list {
                    if let Err(err) = self.exporter.path_declared(description).await {
                        log::error!("Can't notify exporter about a new path: {}", err);
                    }
                }
            }
            other => {
                log::warn!("Message {:?} not supported yet.", other);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::ForwardRequest> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::ForwardRequest,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Return optional id here!
        self.send_request(msg.request);
        Ok(())
    }
}
