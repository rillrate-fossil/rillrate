use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy, StartedBy, TaskEliminated,
};
use meio_connect::{
    filters::{IndexRequest, WsRequest},
    server::WebServer,
    warp::{Filter, Reply},
};
use rill::protocol::PORT;
use std::net::SocketAddr;
use strum_macros::EnumString;

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new() -> Self {
        let addr = format!("127.0.0.1:{}", PORT).parse().unwrap();
        Self { addr }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Provider,
    WebServer,
}

#[async_trait]
impl Actor for Server {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("Server({})", self.addr)
    }
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Server {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Provider, Group::WebServer]);
        let index = IndexRequest::filter(ctx.address().clone());
        let live = WsRequest::filter("live", ctx.address().clone());
        let routes = index.or(live);
        let web_server = WebServer::new(self.addr, routes);
        ctx.spawn_task(web_server, Group::WebServer);
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for Server {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<F> TaskEliminated<WebServer<F>> for Server
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: Reply,
{
    async fn handle(
        &mut self,
        _id: IdOf<WebServer<F>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<IndexRequest> for Server {
    async fn handle(
        &mut self,
        _msg: IndexRequest,
        _ctx: &mut Context<Self>,
    ) -> Result<Box<dyn Reply>, Error> {
        Ok(Box::new("Rate Export Service"))
    }
}

#[derive(EnumString)]
#[strum(serialize_all = "lowercase")]
enum Endpoint {
    Provider,
}

#[async_trait]
impl InteractionHandler<WsRequest<Endpoint>> for Server {
    async fn handle(
        &mut self,
        msg: WsRequest<Endpoint>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::debug!("Incoming ws connection: {}", msg.addr);
        if !ctx.is_terminating() {
            match msg.section {
                Endpoint::Provider => {
                    /*
                    let session = ProviderSession::new(msg.into(), self.router.clone());
                    ctx.spawn_actor(session, Group::Providers);
                    */
                }
            }
        } else {
            log::warn!("Incoming sesson rejected, because the server is terminating...");
        }
        Ok(())
    }
}
