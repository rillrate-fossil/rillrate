use super::link;
use crate::actors::exporter::ExporterLinkForProvider;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, IdOf, InteractionHandler, InterruptedBy, StartedBy,
    TaskEliminated, TaskError,
};
use meio_connect::{
    server::{WsHandler, WsProcessor},
    TermReason, WsIncoming,
};
use rill_protocol::provider::{
    DirectId, Direction, EntryId, Envelope, Path, ProviderReqId, RillEvent, RillProtocol,
    RillToProvider, RillToServer, WideEnvelope,
};
use std::collections::HashMap;

pub struct ProviderSession {
    handler: WsHandler<RillProtocol>,
    registered: Option<EntryId>,
    exporter: ExporterLinkForProvider,
    counter: usize,
    // TODO: Replace to `TypedSlab`
    paths: HashMap<DirectId<RillProtocol>, Path>,
}

impl ProviderSession {
    pub fn new(handler: WsHandler<RillProtocol>, exporter: ExporterLinkForProvider) -> Self {
        Self {
            handler,
            registered: None,
            exporter,
            counter: 0,
            paths: HashMap::new(),
        }
    }

    fn send_request(&mut self, direct_id: ProviderReqId, data: RillToProvider) {
        let envelope = Envelope { direct_id, data };
        log::trace!("Sending request to the server: {:?}", envelope);
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
impl StartedBy<Server> for ProviderSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
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

impl ProviderSession {
    async fn distribute_data(
        &mut self,
        direction: Direction<RillProtocol>,
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
                    "Unknown direction {:?} of the incoing data {:?}",
                    direct_id,
                    event
                );
            }
        } else {
            log::error!(
                "Not supported direction {:?} of the incoing data {:?}",
                direction,
                event
            );
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<WideEnvelope<RillProtocol, RillToServer>>> for ProviderSession {
    async fn handle(
        &mut self,
        msg: WsIncoming<WideEnvelope<RillProtocol, RillToServer>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Provider incoming message: {:?}", msg);
        match msg.0.data {
            RillToServer::Data { event } => {
                self.distribute_data(msg.0.direction, event).await?;
            }
            RillToServer::BeginStream { snapshot } => {
                // It's important to forward the snapshot, because it
                // a stream doesn't generate data too often, but the provider
                // can keep it than we can have the current value in exporters.
                if let Some(event) = snapshot {
                    log::trace!("Snapshot received: {:?}", event);
                    self.distribute_data(msg.0.direction, event).await?;
                } else {
                    log::trace!("Stream started without a snapshot.");
                }
            }
            RillToServer::EndStream => {}
            RillToServer::Declare { entry_id } => {
                ctx.not_terminating()?;
                self.exporter
                    .session_attached(entry_id.clone(), ctx.address().link())
                    .await?;
                self.registered = Some(entry_id);
                let msg = RillToProvider::Describe { active: true };
                self.send_request(0.into(), msg);
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
impl InteractionHandler<link::NewRequest> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::NewRequest,
        _ctx: &mut Context<Self>,
    ) -> Result<ProviderReqId, Error> {
        self.counter += 1;
        let direct_id = DirectId::from(self.counter);

        if let RillToProvider::ControlStream {
            ref path,
            active: true,
        } = msg.request
        {
            self.paths.insert(direct_id, path.clone());
        }

        self.send_request(direct_id, msg.request);

        Ok(direct_id)
    }
}

#[async_trait]
impl ActionHandler<link::SubRequest> for ProviderSession {
    async fn handle(
        &mut self,
        msg: link::SubRequest,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let direct_id = msg.direct_id;
        self.paths.remove(&direct_id);
        self.send_request(direct_id, msg.request);
        Ok(())
    }
}
