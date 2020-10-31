use crate::protocol::{RillServerProtocol, RillToProvider, RillToServer};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, InteractionHandler, LiteTask};
use meio_ws::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use std::time::Duration;

pub struct Connector {
    url: String,
    sender: Option<WsSender<RillToServer>>,
}

impl Connector {
    pub fn new(url: String) -> Self {
        Self { url, sender: None }
    }
}

#[async_trait]
impl Actor for Connector {
    fn name(&self) -> String {
        format!("Connector({})", self.url)
    }

    async fn initialize(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let client = WsClient::new(
            self.url.clone(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        let ws_client = client.start(ctx.bind());
        ctx.terminator()
            .new_stage("ws_client", false)
            .insert(ws_client);
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<WsClientStatus<RillServerProtocol>> for Connector {
    async fn handle(
        &mut self,
        status: WsClientStatus<RillServerProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match status {
            WsClientStatus::Connected { sender } => {
                self.sender = Some(sender);
                // TODO: Send declarations if they already exists
            }
            WsClientStatus::Failed(reason) => {}
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<RillToProvider>> for Connector {
    async fn handle(
        &mut self,
        msg: WsIncoming<RillToProvider>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match msg.0 {
            RillToProvider::CanDrop { stream_id } => {
                todo!();
            }
            RillToProvider::ControlStream { stream_id, active } => {
                todo!();
            }
        }
        Ok(())
    }
}
