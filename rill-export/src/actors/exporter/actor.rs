use super::link;
use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::session::SessionLink;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{ActionHandler, Actor, Context, InterruptedBy, StartedBy};
use rill::protocol::Path;
use std::collections::HashSet;

/// The `Actor` that subscribes to data according to available `Path`s.
pub struct Exporter {
    session: Option<SessionLink>,
    paths_to_export: HashSet<Path>,
}

impl Exporter {
    pub fn new(paths_to_export: HashSet<Path>) -> Self {
        Self {
            session: None,
            paths_to_export,
        }
    }
}

impl Actor for Exporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Exporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for Exporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::SessionLifetime> for Exporter {
    async fn handle(
        &mut self,
        msg: link::SessionLifetime,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        use link::SessionLifetime::*;
        match msg {
            Attached { session } => {
                self.session = Some(session);
            }
            Detached => {}
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::PathDeclared> for Exporter {
    async fn handle(
        &mut self,
        msg: link::PathDeclared,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        todo!("subscribe");
    }
}
