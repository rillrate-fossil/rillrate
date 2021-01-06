use crate::actors::exporter::{ExportEvent, ExporterLinkForClient, PathNotification};
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
        self.exporter
            .subscribe_to_paths(ctx.address().clone())
            .await?;
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
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Client incoming message: {:?}", msg);
        match msg.0 {
            ViewRequest::ControlStream { path, active } => {
                if active {
                    self.exporter
                        .subscribe_to_data(path, ctx.address().clone())
                        .await?;
                } else {
                    self.exporter
                        .unsubscribe_from_data(path, ctx.address())
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<PathNotification> for ClientSession {
    async fn handle(
        &mut self,
        msg: PathNotification,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let response = ViewResponse::Paths(msg.descriptions);
        self.handler.send(response);
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<ExportEvent> for ClientSession {
    async fn handle(&mut self, msg: ExportEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match msg {
            ExportEvent::BroadcastData {
                path,
                data,
                timestamp,
            } => {
                let response = ViewResponse::Data(path, data);
                self.handler.send(response);
            }
        }
        Ok(())
    }
}
