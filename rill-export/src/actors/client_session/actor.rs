use crate::actors::exporter::{ExportEvent, ExporterLinkForClient, PathNotification};
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
use rill_protocol::provider::Path;
use rill_protocol::view::{ViewProtocol, ViewRequest, ViewResponse};
use std::collections::HashSet;
use tokio::sync::broadcast;

pub struct ClientSession {
    handler: WsHandler<ViewProtocol>,
    exporter: ExporterLinkForClient,
    provider: Option<ProviderSessionLink>,
    available_paths: HashSet<Path>,
}

impl ClientSession {
    pub fn new(handler: WsHandler<ViewProtocol>, exporter: ExporterLinkForClient) -> Self {
        Self {
            handler,
            exporter,
            provider: None,
            available_paths: HashSet::new(),
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
            ViewRequest::GetAvailablePaths => {
                let paths = self.available_paths.clone();
                let response = ViewResponse::Paths(paths);
                self.handler.send(response);
            }
            ViewRequest::Subscribe(path) => {
                self.exporter
                    .subscribe_to_data(path, ctx.address().clone())
                    .await?;
            }
            ViewRequest::Unsubscribe => {
                todo!();
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
        let paths = msg.descriptions.into_iter().map(|d| d.path);
        self.available_paths.extend(paths);
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
                todo!("filter and forward");
            }
        }
        Ok(())
    }
}
