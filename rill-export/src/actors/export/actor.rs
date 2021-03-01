use crate::config::ExportConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Address, Context, Eliminated, IdOf, InterruptedBy, StartedBy};
use rill_client::actors::broadcaster::Broadcaster;
use rill_client::actors::client::RillClient;

pub struct RillExport {
    config: ExportConfig,
    client: Option<Address<RillClient>>,
    broadcaster: Option<Address<Broadcaster>>,
}

impl RillExport {
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            client: None,
            broadcaster: None,
        }
    }
}

impl Actor for RillExport {
    type GroupBy = ();
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillExport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let url = self.config.node_url();
        let actor = Broadcaster::new();
        let broadcaster = ctx.spawn_actor(actor, ());
        let link = broadcaster.link();
        self.broadcaster = Some(broadcaster);

        let actor = RillClient::new(url, link);
        let client = ctx.spawn_actor(actor, ());
        self.client = Some(client);
        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for RillExport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Broadcaster> for RillExport {
    async fn handle(
        &mut self,
        _id: IdOf<Broadcaster>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillClient> for RillExport {
    async fn handle(
        &mut self,
        _id: IdOf<RillClient>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
