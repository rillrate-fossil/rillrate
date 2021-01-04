use crate::actors::exporter::{ExportEvent, ExporterLinkForClient};
use crate::actors::provider_session::ProviderSessionLink;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, IdOf, InterruptedBy, StartedBy, TaskEliminated, TaskError,
    TryConsumer,
};
use meio_connect::{
    server::{WsHandler, WsProcessor},
    TermReason, WsIncoming,
};
use rill_protocol::view::{ViewProtocol, ViewRequest, ViewResponse};
use tokio::sync::broadcast;

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
            .grasp_export_stream(ctx.address().clone())
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
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Client incoming message: {:?}", msg);
        match msg.0 {
            ViewRequest::GetAvailablePaths => {
                // TODO: Send error in case of fail
                let paths = self.exporter.get_paths().await?;
                let response = ViewResponse::Paths(paths);
                self.handler.send(response);
            }
            ViewRequest::Subscribe(path) => {
                // TODO: Send error in case of fail
                let mut session = self.exporter.get_provider_session().await?;
                // TODO: Use address as well
                session.subscribe(path).await?;
                todo!();
            }
            ViewRequest::Unsubscribe => {
                todo!();
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TryConsumer<ExportEvent> for ClientSession {
    type Error = broadcast::RecvError;

    async fn handle(&mut self, event: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ExportEvent::SetInfo { .. } => {}
            ExportEvent::BroadcastData {
                path,
                data,
                timestamp,
            } => {
                todo!("filter and forward");
            }
        }
        Ok(())
    }

    async fn error(&mut self, err: Self::Error, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::error!(
            "Broadcasting stream failed. Not possible to continue: {}",
            err
        );
        ctx.shutdown();
        Ok(())
    }
}
