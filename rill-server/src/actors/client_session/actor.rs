use crate::actors::exporter::{ExportEvent, ExporterLinkForClient, PathNotification};
use crate::actors::router::Router;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Context, IdOf, InterruptedBy, StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    server::{WsHandler, WsProcessor},
    TermReason, WsIncoming,
};
use rill_protocol::client::{ClientProtocol, ClientRequest, ClientResponse};

pub struct ClientSession {
    handler: WsHandler<ClientProtocol>,
    //exporter: ExporterLinkForClient,
}

impl ClientSession {
    pub fn new(handler: WsHandler<ClientProtocol>, exporter: ExporterLinkForClient) -> Self {
        Self { handler }
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        //self.exporter.unsubscribe_all(ctx.address()).await.ok();
        ctx.shutdown();
    }
}

#[async_trait]
impl Actor for ClientSession {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Router> for ClientSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, ());

        /*
        self.exporter
            .subscribe_to_paths(ctx.address().clone())
            .await?;
        */

        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Router> for ClientSession {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsProcessor<ClientProtocol, Self>> for ClientSession {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<ClientProtocol, Self>>,
        _result: Result<TermReason, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<ClientRequest>> for ClientSession {
    async fn handle(
        &mut self,
        msg: WsIncoming<ClientRequest>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Client incoming message: {:?}", msg);
        match msg.0 {
            ClientRequest::ControlStream { path, active } => {
                if active {
                    // TODO: Generate a new link that tracks a subscription.
                    // TODO: And store it in the `Self`.
                    /*
                    self.exporter
                        .subscribe_to_data(path, ctx.address().clone())
                        .await?;
                    */
                } else {
                    /*
                    self.exporter
                        .unsubscribe_from_data(path, ctx.address())
                        .await?;
                    */
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
                let response = ClientResponse::Paths(descriptions);
                self.handler.send(response);
                Ok(())
            }
            PathNotification::Name { name } => {
                let response = ClientResponse::Declare(name);
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
                let response = ClientResponse::Data(path, event);
                self.handler.send(response);
            }
        }
        Ok(())
    }
}
