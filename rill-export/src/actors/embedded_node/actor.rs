use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Bridge, Consumer, Context, Eliminated, IdOf, StartedBy, System};

pub struct EmbeddedNode {}

impl Actor for EmbeddedNode {
    type GroupBy = ();
}

impl EmbeddedNode {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StartedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let server_actor = Server::new();
        let server = ctx.spawn_actor(server_actor, ());
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Server> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Server>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
