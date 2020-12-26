use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InteractionHandler, InterruptedBy, StartedBy};
use meio_http::hyper::{Body, Request, Response};
use meio_http::{FromRequest, HttpServerLink, Req};

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

struct Index;

impl FromRequest for Index {
    fn from_request(request: &Request<Body>) -> Option<Self> {
        let path = request.uri().path();
        if path == "/" || path == "/index.html" {
            Some(Self)
        } else {
            None
        }
    }
}

#[async_trait]
impl InteractionHandler<Req<Index>> for Endpoints {
    async fn handle(
        &mut self,
        _: Req<Index>,
        ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        Ok(Response::new("<Rill Embedded Server>".into()))
    }
}
