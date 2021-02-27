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
use rill_protocol::provider::Envelope;
use rill_protocol::view::{ViewProtocol, ViewResponse};
use std::time::Duration;

pub struct RillClient {
    url: String,
}

impl RillClient {
    pub fn new(url: String) -> Self {
        Self { url }
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
impl InstantActionHandler<WsClientStatus<ViewProtocol>> for RillClient {
    async fn handle(
        &mut self,
        status: WsClientStatus<ViewProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<ViewResponse>> for RillClient {
    async fn handle(
        &mut self,
        msg: WsIncoming<ViewResponse>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<ViewProtocol, Self>> for RillClient {
    async fn handle(
        &mut self,
        _id: IdOf<WsClient<ViewProtocol, Self>>,
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}
