use crate::actors::exporter::ExporterLink;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, IdOf, InteractionHandler, InterruptedBy, Link, StartedBy,
    TaskEliminated,
};
use meio_connect::{
    server::{WsHandler, WsProcessor},
    WsIncoming,
};
use rill::protocol::{
    DirectId, EntryId, Envelope, ProviderReqId, RillProtocol, RillToProvider, RillToServer,
    WideEnvelope,
};

pub struct Session {
    handler: WsHandler<RillProtocol>,
    registered: Option<EntryId>,
    exporter: ExporterLink,
    counter: usize,
}

impl Session {
    pub fn new(handler: WsHandler<RillProtocol>, exporter: ExporterLink) -> Self {
        Self {
            handler,
            registered: None,
            exporter,
            counter: 0,
        }
    }

    fn send_request(&mut self, data: RillToProvider) {
        self.counter += 1;
        let direct_id = DirectId::from(self.counter);
        let envelope = Envelope { direct_id, data };
        log::trace!("Sending request to the server: {:?}", envelope);
        self.handler.send(envelope);
    }
}

#[async_trait]
impl Actor for Session {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Server> for Session {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Server> for Session {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsProcessor<RillProtocol, Self>> for Session {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<RillProtocol, Self>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<WideEnvelope<RillProtocol, RillToServer>>> for Session {
    async fn handle(
        &mut self,
        msg: WsIncoming<WideEnvelope<RillProtocol, RillToServer>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("WsIncoming message: {:?}", msg);
        match msg.0.data {
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
