use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Action, ActionHandler, Actor, Context, InterruptedBy, StartedBy};

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
        ctx.address().act(Configure).await?;
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

struct Configure;

impl Action for Configure {}

#[async_trait]
impl ActionHandler<Configure> for Tuner {
    async fn handle(&mut self, _: Configure, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
