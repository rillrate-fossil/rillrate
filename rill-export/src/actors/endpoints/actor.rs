use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InterruptedBy, StartedBy};

pub struct Endpoints {}

impl Endpoints {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for Endpoints {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Endpoints {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for Endpoints {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
