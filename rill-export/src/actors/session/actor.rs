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
    EntryId, Envelope, ProviderReqId, RillProtocol, RillToProvider, RillToServer, WideEnvelope,
};

pub struct Session {
    handler: WsHandler<RillProtocol>,
    registered: Option<EntryId>,
}

impl Session {
    pub fn new(handler: WsHandler<RillProtocol>) -> Self {
        Self {
            handler,
            registered: None,
        }
    }

    fn send_envelope(&mut self, envelope: Envelope<RillProtocol, RillToProvider>) {
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
            }
            other => {
                log::warn!("Message {:?} not supported yet.", other);
            }
        }
        Ok(())
    }
}
