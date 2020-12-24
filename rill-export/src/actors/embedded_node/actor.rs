use crate::actors::exporter::Exporter;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Bridge, Consumer, Context, Eliminated, IdOf, StartedBy, System};

pub struct EmbeddedNode {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Exporter,
    Server,
}

impl Actor for EmbeddedNode {
    type GroupBy = Group;
}

impl EmbeddedNode {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StartedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Exporter, Group::Server]);

        let exporter_actor = Exporter::new();
        let exporter = ctx.spawn_actor(exporter_actor, Group::Exporter);

        let server_actor = Server::new();
        let server = ctx.spawn_actor(server_actor, Group::Server);

        Ok(())
    }
}

#[async_trait]
impl Eliminated<Exporter> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Exporter>, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Exporter finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Server> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Server>, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Server finished");
        Ok(())
    }
}
