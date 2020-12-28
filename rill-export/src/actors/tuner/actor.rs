use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InterruptedBy, StartedBy};

pub struct Tuner {}

impl Tuner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for Tuner {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Tuner {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for Tuner {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
