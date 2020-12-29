use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::exporter::ExporterLinkForData;
use crate::actors::session::Session;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy, StartedBy,
};
use meio_connect::hyper::{Body, Request, Response};
use meio_connect::server::{DirectPath, FromRequest, HttpServerLink, Req, WsReq};
use rill::protocol::RillProtocol;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Server {
    server: HttpServerLink,
    // TODO: Or maybe use `Address` here if different types of links required:
    // - for data
    // - and for controls
    exporter: ExporterLinkForData,
    connected: bool,
    ui_path: PathBuf,
}

impl Server {
    pub fn new(server: HttpServerLink, exporter: ExporterLinkForData) -> Self {
        Self {
            server,
            exporter,
            connected: false,
            ui_path: Path::new(".").to_path_buf(),
        }
    }
}

impl Actor for Server {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Server {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.server
            .add_route::<Index, _>(ctx.address().clone())
            .await?;
        self.server
            .add_ws_route::<Live, RillProtocol, _>(ctx.address().clone())
            .await?;
        self.server
            .add_route::<Ui, _>(ctx.address().clone())
            .await?;
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

#[derive(Default)]
struct Index;

impl DirectPath for Index {
    fn paths() -> &'static [&'static str] {
        &["/", "/index.html"]
    }
}

#[async_trait]
impl InteractionHandler<Req<Index>> for Server {
    async fn handle(
        &mut self,
        _: Req<Index>,
        _ctx: &mut Context<Self>,
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
impl ActionHandler<WsReq<Live, RillProtocol>> for Server {
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
impl Eliminated<Session> for Server {
    async fn handle(&mut self, _id: IdOf<Session>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        self.exporter.session_detached().await?;
        // It allows to connect again
        self.connected = false;
        Ok(())
    }
}

#[derive(Default)]
struct Ui {
    tail: PathBuf,
}

impl FromRequest for Ui {
    fn from_request(request: &Request<Body>) -> Option<Self> {
        let path = request.uri().path();
        if path.starts_with("/ui/") {
            let tail = Path::new(&path[4..]).to_path_buf();
            Some(Self { tail })
        } else {
            None
        }
    }
}

/// WARNING! This implementation serves any static files by any paths.
/// It's unsafe to use in prod, because you can load any file using `ui` endpoint.
/// It used for UI-debugging purposes only.
#[cfg(debug_assertions)]
#[async_trait]
impl InteractionHandler<Req<Ui>> for Server {
    async fn handle(
        &mut self,
        msg: Req<Ui>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        let mut full_path = self.ui_path.clone();
        full_path.push(msg.request.tail);
        log::warn!(
            "Read overriden file asset from the path: {}",
            full_path.display()
        );
        let mut file = File::open(full_path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let data = contents;
        /*
        let mime = mime_guess::from_path(tail.as_str()).first_or_octet_stream();
        let mut resp = data
            .map(Reply::into_response)
            .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response());
        resp.headers_mut().typed_insert(ContentType::from(mime));
        */
        let response = Response::new(Body::from(data));
        Ok(response)
    }
}
