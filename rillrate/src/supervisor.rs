use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};
use rill::RillEngine;
use rill_export::EmbeddedNode;

pub struct RillRate {}

impl RillRate {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Engine,
    EmbeddedNode,
}

impl Actor for RillRate {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Engine, Group::EmbeddedNode]);

        let config_path = Some(crate::env::config());
        let actor = EmbeddedNode::new(config_path);
        ctx.spawn_actor(actor, Group::EmbeddedNode);

        // TODO: Use the same config
        let actor = RillEngine::new("127.0.0.1:1636".into(), "rillrate");
        ctx.spawn_actor(actor, Group::Engine);

        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillEngine> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<RillEngine>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<EmbeddedNode> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<EmbeddedNode>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
