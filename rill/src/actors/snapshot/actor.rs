use super::link;
use crate::actors::supervisor::RillSupervisor;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{ActionHandler, Actor, Context, InterruptedBy, StartedBy};

pub struct SnapshotWorker {}

impl SnapshotWorker {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for SnapshotWorker {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillSupervisor> for SnapshotWorker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for SnapshotWorker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::AttachTracer> for SnapshotWorker {
    async fn handle(
        &mut self,
        msg: link::AttachTracer,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
