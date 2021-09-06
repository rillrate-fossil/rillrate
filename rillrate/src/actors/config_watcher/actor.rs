use crate::actors::supervisor::NodeSupervisor;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy};

pub struct ConfigWatcher {}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for ConfigWatcher {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<NodeSupervisor> for ConfigWatcher {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<NodeSupervisor> for ConfigWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
