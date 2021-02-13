use crate::actors::exporter::{ExportEvent, ExporterLinkForClient, PathNotification};
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
}

impl ClientSession {
    pub fn new(handler: WsHandler<ViewProtocol>, exporter: ExporterLinkForClient) -> Self {
        Self { handler, exporter }
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        self.exporter.unsubscribe_all(ctx.address()).await.ok();
        ctx.shutdown();
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
        self.graceful_shutdown(ctx).await;
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
        self.graceful_shutdown(ctx).await;
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
                    // TODO: Generate a new link that tracks a subscription.
                    // TODO: And store it in the `Self`.
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
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match msg {
            PathNotification::Paths { descriptions } => {
                let response = ViewResponse::Paths(descriptions);
                self.handler.send(response);
                Ok(())
            }
            PathNotification::Name { name } => {
                let response = ViewResponse::Declare(name);
                self.handler.send(response);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl ActionHandler<ExportEvent> for ClientSession {
    async fn handle(&mut self, msg: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match msg {
            ExportEvent::BroadcastData { path, event } => {
                let response = ViewResponse::Data(path, event);
                self.handler.send(response);
            }
        }
        Ok(())
    }
}
