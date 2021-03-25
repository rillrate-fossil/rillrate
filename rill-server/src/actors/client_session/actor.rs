use crate::actors::provider_session::{BindedProviderLink, ProviderLink};
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
use once_cell::sync::Lazy;
use rill_client::actors::broadcaster::{BroadcasterLinkForClient, PathNotification};
use rill_protocol::io::client::{ClientProtocol, ClientRequest, ClientResponse};
use rill_protocol::io::transport::{Direction, Envelope, WideEnvelope};
use tokio::sync::Mutex;

pub static PROVIDER: Lazy<Mutex<Option<ProviderLink>>> = Lazy::new(|| Mutex::new(None));

pub struct ClientSession {
    handler: WsHandler<ClientProtocol>,
    exporter: BroadcasterLinkForClient,
    provider: Option<BindedProviderLink>,
}

impl ClientSession {
    pub fn new(handler: WsHandler<ClientProtocol>, exporter: BroadcasterLinkForClient) -> Self {
        Self {
            handler,
            exporter,
            provider: None,
        }
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        //self.exporter.unsubscribe_all(ctx.address()).await.ok();
        if let Ok(provider) = self.provider() {
            provider.unsubscribe_all().await;
        }
        ctx.shutdown();
    }

    fn provider(&mut self) -> Result<&mut BindedProviderLink, Error> {
        self.provider
            .as_mut()
            .ok_or_else(|| Error::msg("Provider not binded"))
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
        ctx.spawn_task(worker, (), ());

        self.exporter
            .subscribe_to_struct_changes(ctx.address().clone())
            .await?;

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
impl TaskEliminated<WsProcessor<ClientProtocol, Self>, ()> for ClientSession {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<ClientProtocol, Self>>,
        _tag: (),
        _result: Result<TermReason, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<Envelope<ClientProtocol, ClientRequest>>> for ClientSession {
    async fn handle(
        &mut self,
        msg: WsIncoming<Envelope<ClientProtocol, ClientRequest>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Client request: {:?}", msg);
        // TODO: Return `Error` response to the client by WS
        match msg.0.data {
            ClientRequest::ControlStream { path, active } => {
                if active {
                    self.provider()?.subscribe(path, msg.0.direct_id).await?;
                } else {
                    self.provider()?.unsubscribe(path, msg.0.direct_id).await?;
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
            PathNotification::Paths { .. } => Ok(()),
            PathNotification::Name { name } => {
                // Get the provider when it connected and declared.
                let sender = self.handler.sender();
                self.provider = PROVIDER.lock().await.as_ref().map(|link| link.bind(sender));

                let response = ClientResponse::Declare(name);
                let envelope = WideEnvelope {
                    direction: Direction::broadcast(),
                    data: response,
                };
                self.handler.send(envelope);
                Ok(())
            }
        }
    }
}

/*
#[async_trait]
impl ActionHandler<ExportEvent> for ClientSession {
    async fn handle(&mut self, msg: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match msg {
            ExportEvent::BroadcastData { path, event } => {
                //let response = ClientResponse::Data(path, event);
                //self.handler.send(response);
            }
        }
        Ok(())
    }
}
*/
