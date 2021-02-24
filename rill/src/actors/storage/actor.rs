use crate::actors::engine::RillEngine;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InterruptedBy, StartedBy};

pub struct RillStorage {}

impl RillStorage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for RillStorage {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillEngine> for RillStorage {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Opens a log file
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillEngine> for RillStorage {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
