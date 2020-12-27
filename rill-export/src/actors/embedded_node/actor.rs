use crate::actors::exporter::Exporter;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};
use meio_connect::server::HttpServer;

pub struct EmbeddedNode {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Exporter,
    HttpServer,
    Endpoints,
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
        ctx.termination_sequence(vec![Group::Exporter, Group::HttpServer, Group::Endpoints]);

        let addr = format!("127.0.0.1:{}", rill::PORT.get()).parse().unwrap();
        let http_server_actor = HttpServer::new(addr);
        let http_server = ctx.spawn_actor(http_server_actor, Group::HttpServer);

        let exporter_actor = Exporter::new(http_server.link(), Default::default());
        let exporter = ctx.spawn_actor(exporter_actor, Group::Exporter);

        let server_actor = Server::new(http_server.link(), exporter.link());
        let server = ctx.spawn_actor(server_actor, Group::Endpoints);

        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Exporter> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Exporter>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Exporter finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<HttpServer> for EmbeddedNode {
    async fn handle(
        &mut self,
        _id: IdOf<HttpServer>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("HttpServer finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Server> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Server>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Server finished");
        Ok(())
    }
}
