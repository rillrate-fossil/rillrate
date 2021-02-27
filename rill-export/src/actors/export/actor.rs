use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy};

pub struct RillExport {}

impl RillExport {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for RillExport {
    type GroupBy = ();
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillExport {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
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
