use crate::actors::engine::RillEngine;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy};

pub struct RillPool {}

impl RillPool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for RillPool {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillEngine> for RillPool {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillEngine> for RillPool {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
