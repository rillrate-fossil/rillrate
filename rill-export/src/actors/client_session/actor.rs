use crate::actors::exporter::ExporterLinkForClient;
use crate::actors::provider_session::ProviderSessionLink;
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
use rill_protocol::view::{ViewProtocol, ViewRequest, ViewResponse};

pub struct ClientSession {
    handler: WsHandler<ViewProtocol>,
    exporter: ExporterLinkForClient,
    provider: Option<ProviderSessionLink>,
}

impl ClientSession {
    pub fn new(handler: WsHandler<ViewProtocol>, exporter: ExporterLinkForClient) -> Self {
        Self {
            handler,
            exporter,
            provider: None,
        }
    }
}

#[async_trait]
impl Actor for ClientSession {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Server> for ClientSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Server> for ClientSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsProcessor<ViewProtocol, Self>> for ClientSession {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<ViewProtocol, Self>>,
        _result: Result<TermReason, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<ViewRequest>> for ClientSession {
    async fn handle(
        &mut self,
        msg: WsIncoming<ViewRequest>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Client incoming message: {:?}", msg);
        match msg.0 {
            ViewRequest::GetAvailablePaths => {
                let paths = self.exporter.get_paths().await?;
                let response = ViewResponse::Paths(paths);
                self.handler.send(response);
            }
            ViewRequest::Subscribe(path) => {
                todo!();
            }
            ViewRequest::Unsubscribe => {
                todo!();
            }
        }
        Ok(())
    }
}
