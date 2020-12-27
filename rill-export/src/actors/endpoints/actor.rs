use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::exporter::ExporterLink;
use crate::actors::session::Session;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy, Link,
    StartedBy,
};
use meio_connect::hyper::{Body, Response};
use meio_connect::server_2::{DirectPath, HttpServerLink, Req, WsReq};
use rill::protocol::RillProtocol;

// TODO: Rename to server?
pub struct Endpoints {
    server: HttpServerLink,
    exporter: ExporterLink,
    connected: bool,
}

impl Endpoints {
    pub fn new(server: HttpServerLink, exporter: ExporterLink) -> Self {
        Self {
            server,
            exporter,
            connected: false,
        }
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
            .add_ws_route::<Live, RillProtocol, _>(ctx.address().clone())
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
impl ActionHandler<WsReq<Live, RillProtocol>> for Endpoints {
    async fn handle(
        &mut self,
        req: WsReq<Live, RillProtocol>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            if !self.connected {
                self.connected = true;
                let session_actor = Session::new(req.stream, self.exporter.clone());
                let session = ctx.spawn_actor(session_actor, ());
                self.exporter.session_attached(session.link()).await?;
            } else {
                // TODO: Add address
                log::error!("Reject the second incoming connection from: {}", "msg.addr");
            }
        } else {
            log::warn!("Incoming ws connection rejected, because the server is terminating.");
        }
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Session> for Endpoints {
    async fn handle(&mut self, _id: IdOf<Session>, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.exporter.session_detached().await?;
        // It allows to connect again
        self.connected = false;
        Ok(())
    }
}
