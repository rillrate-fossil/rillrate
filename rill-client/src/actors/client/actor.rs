use super::link;
use crate::actors::broadcaster::BroadcasterLinkForProvider;
use anyhow::Error;
use async_trait::async_trait;
use futures::{channel::mpsc, SinkExt};
use meio::{
    ActionHandler, Actor, Context, IdOf, InstantActionHandler, InteractionHandler, InterruptedBy,
    StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::io::client::{ClientProtocol, ClientReqId, ClientRequest, ClientResponse};
use rill_protocol::io::provider::{Path, StreamState};
use rill_protocol::io::transport::{Direction, Envelope, WideEnvelope};
use std::time::Duration;
use typed_slab::TypedSlab;

type Connection = WsSender<Envelope<ClientProtocol, ClientRequest>>;

enum Record {
    Active {
        path: Path,
        sender: mpsc::Sender<StateOrDelta>,
    },
    AwaitingEnd,
}

pub struct RillClient {
    url: String,
    sender: Option<Connection>,
    broadcaster: BroadcasterLinkForProvider,
    directions: TypedSlab<ClientReqId, Record>,
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

impl RillClient {
    async fn distribute_event(
        &mut self,
        direction: Direction<ClientProtocol>,
        event: StateOrDelta,
    ) {
        for direction in direction.into_vec() {
            if let Some(record) = self.directions.get_mut(direction) {
                if let Record::Active { sender, .. } = record {
                    if let Err(err) = sender.send(event.clone()).await {
                        log::error!("Can't send data to {:?}: {}", direction, err);
                    }
                }
            }
        }
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
            ClientResponse::State(state) => {
                let event = StateOrDelta::State(state);
                self.distribute_event(msg.0.direction, event).await;
            }
            ClientResponse::Delta(delta) => {
                let event = StateOrDelta::Delta(delta);
                self.distribute_event(msg.0.direction, event).await;
            }
            ClientResponse::Done => {
                let directions = msg.0.direction.into_vec();
                for direction in directions {
                    self.directions.remove(direction);
                }
            }
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
        ctx: &mut Context<Self>,
    ) -> Result<link::Subscription, Error> {
        log::info!("Subscribing to {}", msg.path);
        let (tx, rx) = mpsc::channel(32);
        let record = Record::Active {
            path: msg.path.clone(),
            sender: tx,
        };
        let direct_id = self.directions.insert(record);
        let data = ClientRequest::ControlStream {
            path: msg.path,
            active: true,
        };
        let envelope = Envelope { direct_id, data };
        self.sender()?.send(envelope);
        let subscrtiption = link::Subscription::new(direct_id, rx, ctx.address().clone());
        Ok(subscrtiption)
    }
}

#[async_trait]
impl InstantActionHandler<link::UnsubscribeFromPath> for RillClient {
    async fn handle(
        &mut self,
        msg: link::UnsubscribeFromPath,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let direct_id = msg.req_id;
        let record = self.directions.get_mut(direct_id);
        if let Some(record) = record {
            let mut new_record = Record::AwaitingEnd;
            std::mem::swap(record, &mut new_record);
            if let Record::Active { path, .. } = new_record {
                let data = ClientRequest::ControlStream {
                    path,
                    active: false,
                };
                let envelope = Envelope { direct_id, data };
                self.sender()?.send(envelope);
            } else {
                log::error!("Attempt to unsubscribe twice for {:?}", direct_id);
            }
        } else {
            log::error!("Attempt to unsubscribe for non existent {:?}", direct_id);
        }
        Ok(())
    }
}

// TODO: Move somewwhere?
#[derive(Debug, Clone)]
pub enum StateOrDelta {
    State(StreamState),
    Delta(Vec<u8>),
}
