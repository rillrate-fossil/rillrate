use crate::env;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};
use rill::RillEngine;
use rill_export::EmbeddedNode;

pub struct RillRate {
    app_name: String,
    // TODO: Keep node addr here as `Option`
    // and if it's not configured than spawn a standalone server
    // and with for it install the port here and spawn a tracer.
}

impl RillRate {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    BindWatchers,
    Engine,
    EmbeddedNode,
}

impl Actor for RillRate {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::BindWatchers,
            Group::Engine,
            Group::EmbeddedNode,
        ]);

        let config_path = Some(env::config());

        let node = {
            if let Some(node) = env::node() {
                node
            } else {
                let actor = EmbeddedNode::new(config_path);
                ctx.spawn_actor(actor, Group::EmbeddedNode);
                "127.0.0.1:1636".into()
            }
        };

        /* TODO: Create a LiteTask to detect it was binded
        let extern_rx = rill_export::EXTERN_ADDR.1.clone();
        let intern_rx = rill_export::INTERN_ADDR.1.clone();
        */

        // TODO: Use the same config
        let name = env::name(Some(self.app_name.clone()));
        let actor = RillEngine::new(node, name);
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
