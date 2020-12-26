use crate::actors::endpoints::Endpoints;
use crate::actors::exporter::Exporter;
use crate::actors::server::Server;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Bridge, Consumer, Context, Eliminated, IdOf, InterruptedBy, Link, StartedBy, System,
};
use meio_http::HttpServer;

pub struct EmbeddedNode {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Exporter,
    Server,
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
        ctx.termination_sequence(vec![Group::Exporter, Group::Server, Group::Endpoints]);

        let addr = format!("127.0.0.1:{}", rill::PORT.get()).parse().unwrap();
        let server_actor = HttpServer::new(addr);
        let server = ctx.spawn_actor(server_actor, Group::Server);

        let exporter_actor = Exporter::new(server.link(), Default::default());
        let exporter = ctx.spawn_actor(exporter_actor, Group::Exporter);

        let endpoints_actor = Endpoints::new(server.link());
        let endpoints = ctx.spawn_actor(endpoints_actor, Group::Endpoints);

        /*
        let server_actor = Server::new(exporter.link());
        let server = ctx.spawn_actor(server_actor, Group::Server);
        */

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
    async fn handle(&mut self, _id: IdOf<Exporter>, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Exporter finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<HttpServer> for EmbeddedNode {
    async fn handle(
        &mut self,
        _id: IdOf<HttpServer>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("HttpServer finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Endpoints> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Endpoints>, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Endpoints finished");
        Ok(())
    }
}

/*
#[async_trait]
impl Eliminated<Server> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Server>, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Server finished");
        Ok(())
    }
}
*/
