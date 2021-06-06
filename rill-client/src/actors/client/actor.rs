use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Context, IdOf, InstantActionHandler, InterruptedBy, StartedBy,
    TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::io::client::{
    ClientProtocol, ClientReqId, ClientRequest, ClientResponse, ClientServiceRequest,
    ClientServiceResponse,
};
use rill_protocol::io::transport::{Envelope, ServiceEnvelope};
use std::time::Duration;

type WsOutgoing = WsSender<ServiceEnvelope<ClientProtocol, ClientRequest, ClientServiceResponse>>;

pub struct RillClient {
    url: String,
    sender: Option<WsOutgoing>,
}

impl RillClient {
    pub fn new(url: String) -> Self {
        Self { url, sender: None }
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
        // TODO: Use `strum` here
        ctx.termination_sequence(vec![Group::WsConnection]);

        let client = WsClient::new(
            self.url.clone(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        ctx.spawn_task(client, (), Group::WsConnection);

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
                self.sender.take();
            }
        }
        Ok(())
    }
}

#[async_trait]
impl
    ActionHandler<WsIncoming<ServiceEnvelope<ClientProtocol, ClientResponse, ClientServiceRequest>>>
    for RillClient
{
    async fn handle(
        &mut self,
        msg: WsIncoming<ServiceEnvelope<ClientProtocol, ClientResponse, ClientServiceRequest>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Incoming to exporter: {:?}", msg);
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<ClientProtocol, Self>, ()> for RillClient {
    async fn handle(
        &mut self,
        _id: IdOf<WsClient<ClientProtocol, Self>>,
        _tag: (),
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}
