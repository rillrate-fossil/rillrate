use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{ActionHandler, Actor, Context, InteractionHandler, InterruptedBy, StartedBy};
use meio_http::hyper::{Body, Request, Response};
use meio_http::{DirectPath, FromRequest, HttpServerLink, Req, WsReq};

pub struct Endpoints {
    server: HttpServerLink,
}

impl Endpoints {
    pub fn new(server: HttpServerLink) -> Self {
        Self { server }
    }
}

impl Actor for Endpoints {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Endpoints {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.server
            .add_route::<Index, _>(ctx.address().clone())
            .await?;
        self.server
            .add_ws_route::<Live, _>(ctx.address().clone())
            .await?;
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

#[derive(Default)]
struct Index;

impl DirectPath for Index {
    fn paths() -> &'static [&'static str] {
        &["/", "/index.html"]
    }
}

#[async_trait]
impl InteractionHandler<Req<Index>> for Endpoints {
    async fn handle(
        &mut self,
        _: Req<Index>,
        ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        Ok(Response::new("Rill Embedded Server".into()))
    }
}

#[derive(Default)]
struct Live;

impl DirectPath for Live {
    fn paths() -> &'static [&'static str] {
        &["/live/provider"]
    }
}

#[async_trait]
impl ActionHandler<WsReq<Live>> for Endpoints {
    async fn handle(&mut self, req: WsReq<Live>, ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
