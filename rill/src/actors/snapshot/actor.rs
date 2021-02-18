use crate::actors::supervisor::RillSupervisor;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InterruptedBy, StartedBy};

pub struct SnapshotTracker {}

impl SnapshotTracker {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for SnapshotTracker {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillSupervisor> for SnapshotTracker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for SnapshotTracker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
