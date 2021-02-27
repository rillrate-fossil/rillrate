use crate::actors::client::RillClient;
use crate::config::ExportConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy};

pub struct RillExport {
    config: ExportConfig,
}

impl RillExport {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }
}

impl Actor for RillExport {
    type GroupBy = ();
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillExport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let url = self.config.node_url();
        let actor = RillClient::new(url);
        ctx.spawn_actor(actor, ());
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
impl Eliminated<RillClient> for RillExport {
    async fn handle(
        &mut self,
        _id: IdOf<RillClient>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
