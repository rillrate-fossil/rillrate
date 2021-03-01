use super::link;
use crate::actors::broadcaster::BroadcasterLinkForProvider;
use anyhow::Error;
use async_trait::async_trait;
use futures::channel::mpsc;
use meio::{
    ActionHandler, Actor, Context, IdOf, InstantActionHandler, InteractionHandler, InterruptedBy,
    StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::client::{ClientProtocol, ClientReqId, ClientRequest, ClientResponse};
use rill_protocol::provider::RillEvent;
use rill_protocol::transport::{Envelope, WideEnvelope};
use std::time::Duration;
use typed_slab::TypedSlab;

type Connection = WsSender<Envelope<ClientProtocol, ClientRequest>>;

pub struct RillClient {
    url: String,
    sender: Option<Connection>,
    broadcaster: BroadcasterLinkForProvider,
    directions: TypedSlab<ClientReqId, mpsc::Sender<Vec<RillEvent>>>,
}

impl RillClient {
    pub fn new(url: String, broadcaster: BroadcasterLinkForProvider) -> Self {
        Self {
            url,
            sender: None,
            broadcaster,
            directions: TypedSlab::new(),
        }
    }

    fn sender(&self) -> Result<&Connection, Error> {
        self.sender
            .as_ref()
            .ok_or_else(|| Error::msg("not connected"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    WsConnection,
}

impl Actor for RillClient {
    type GroupBy = Group;
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillClient {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::WsConnection]);

        let client = WsClient::new(
            self.url.clone(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        ctx.spawn_task(client, Group::WsConnection);

        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for RillClient {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl InstantActionHandler<WsClientStatus<ClientProtocol>> for RillClient {
    async fn handle(
        &mut self,
        status: WsClientStatus<ClientProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match status {
            WsClientStatus::Connected { sender } => {
                self.sender = Some(sender);
            }
            WsClientStatus::Failed { reason } => {
                log::error!("Connection failed: {}", reason);
                self.broadcaster.session_detached().await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<WideEnvelope<ClientProtocol, ClientResponse>>> for RillClient {
    async fn handle(
        &mut self,
        msg: WsIncoming<WideEnvelope<ClientProtocol, ClientResponse>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Incoming to exporter: {:?}", msg);
        match msg.0.data {
            ClientResponse::Declare(entry_id) => {
                self.broadcaster.session_attached(entry_id).await?;
            }
            ClientResponse::Paths(descriptions) => {
                for desc in descriptions {
                    self.broadcaster.path_declared(desc).await?;
                }
            }
            ClientResponse::Data(batch) => {}
            ClientResponse::Done => {}
        }
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<ClientProtocol, Self>> for RillClient {
    async fn handle(
        &mut self,
        _id: IdOf<WsClient<ClientProtocol, Self>>,
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<link::SubscribeToPath> for RillClient {
    async fn handle(
        &mut self,
        msg: link::SubscribeToPath,
        _ctx: &mut Context<Self>,
    ) -> Result<ClientReqId, Error> {
        log::info!("Subscribing to {}", msg.path);
        let direct_id = self.directions.insert(msg.sender);
        let data = ClientRequest::ControlStream {
            path: msg.path,
            active: true,
        };
        let envelope = Envelope { direct_id, data };
        self.sender()?.send(envelope);
        Ok(direct_id)
    }
}
