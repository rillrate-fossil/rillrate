use crate::actors::client_session::ClientSession;
use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::exporter::ExporterLinkForProvider;
use crate::actors::provider_session::ProviderSession;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy, StartedBy,
};
use meio_connect::headers::{ContentType, HeaderMapExt, HeaderValue};
use meio_connect::hyper::{header, Body, Request, Response, StatusCode};
use meio_connect::server::{DirectPath, FromRequest, HttpServerLink, Req, WsReq};
use rill_protocol::provider::RillProtocol;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Server {
    server: HttpServerLink,
    // TODO: Or maybe use `Address` here if different types of links required:
    // - for data
    // - and for controls
    exporter: ExporterLinkForProvider,
    connected: bool,
    ui_path: PathBuf,
}

impl Server {
    pub fn new(server: HttpServerLink, exporter: ExporterLinkForProvider) -> Self {
        Self {
            server,
            exporter,
            connected: false,
            ui_path: Path::new(&crate::env::ui()).to_path_buf(),
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
            .add_ws_route::<ProviderLive, RillProtocol, _>(ctx.address().clone())
            .await?;
        self.server
            .add_ws_route::<ClientLive, RillProtocol, _>(ctx.address().clone())
            .await?;
        self.server
            .add_route::<Ui, _>(ctx.address().clone())
            .await?;
        self.server
            .add_route::<Info, _>(ctx.address().clone())
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
        let mut response = Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .body(Body::empty())?;
        // My eyes cry
        response
            .headers_mut()
            .insert(header::LOCATION, HeaderValue::from_static("/ui/"));
        Ok(response)
    }
}

#[derive(Default)]
struct Info;

impl DirectPath for Info {
    fn paths() -> &'static [&'static str] {
        &["/info"]
    }
}

#[async_trait]
impl InteractionHandler<Req<Info>> for Server {
    async fn handle(
        &mut self,
        _: Req<Info>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        let content = format!(
            "Rill ver. {}\nRill Export ver. {}\n",
            rill_protocol::meta::VERSION,
            crate::meta::VERSION
        );
        Ok(Response::new(content.into()))
    }
}

#[derive(Default)]
struct ProviderLive;

impl DirectPath for ProviderLive {
    fn paths() -> &'static [&'static str] {
        &["/live/provider"]
    }
}

#[async_trait]
impl ActionHandler<WsReq<ProviderLive, RillProtocol>> for Server {
    async fn handle(
        &mut self,
        req: WsReq<ProviderLive, RillProtocol>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            if !self.connected {
                self.connected = true;
                let session_actor = ProviderSession::new(req.stream, self.exporter.clone());
                let session = ctx.spawn_actor(session_actor, ());
                self.exporter.session_attached(session.link()).await?;
            } else {
                // TODO: Add address
                log::error!("Reject the second incoming connection from: {}", "msg.addr");
            }
        } else {
            log::warn!("Incoming provider connection rejected, because the server is terminating.");
        }
        Ok(())
    }
}

#[async_trait]
impl Eliminated<ProviderSession> for Server {
    async fn handle(
        &mut self,
        _id: IdOf<ProviderSession>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.exporter.session_detached().await?;
        // It allows to connect again
        self.connected = false;
        Ok(())
    }
}

#[derive(Default)]
struct ClientLive;

impl DirectPath for ClientLive {
    fn paths() -> &'static [&'static str] {
        &["/live/client"]
    }
}

#[async_trait]
impl ActionHandler<WsReq<ClientLive, RillProtocol>> for Server {
    async fn handle(
        &mut self,
        req: WsReq<ClientLive, RillProtocol>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            let session_actor = ClientSession::new(req.stream);
            let _session = ctx.spawn_actor(session_actor, ());
        } else {
            log::warn!("Incoming client connection rejected, because the server is terminating.");
        }
        Ok(())
    }
}

#[async_trait]
impl Eliminated<ClientSession> for Server {
    async fn handle(
        &mut self,
        _id: IdOf<ClientSession>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
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

impl Server {
    async fn serve_file(&self, path: &Path) -> Result<Response<Body>, Error> {
        let content = self.load_content(path).await?;
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        // No one person on the planet knows how I hate
        // that hyper/http/headers/warp are not
        // convenient crates. I'm crying....
        let mut response = Response::builder().body(content.into())?;
        // Why?!?!?!?!?!?!!?!111
        response.headers_mut().typed_insert(ContentType::from(mime));
        Ok(response)
    }

    /// WARNING! This implementation serves any static files by any paths.
    /// It's unsafe to use in prod, because you can load any file using `ui` endpoint.
    /// It used for UI-debugging purposes only.
    #[cfg(debug_assertions)]
    async fn load_content(&self, path: &Path) -> Result<Vec<u8>, Error> {
        let mut full_path = self.ui_path.clone();
        full_path.push(path);
        log::warn!(
            "Read overriden file asset from the path: {}",
            full_path.display()
        );
        let mut file = File::open(full_path).await?;
        let mut content = Vec::new();
        file.read_to_end(&mut content).await?;
        Ok(content)
    }
}

#[async_trait]
impl InteractionHandler<Req<Ui>> for Server {
    async fn handle(
        &mut self,
        msg: Req<Ui>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        let mut path: &Path = msg.request.tail.as_ref();
        if path == Path::new("") {
            path = Path::new("index.html");
        }
        log::trace!("Reading asset: {}", path.display());
        let res = self.serve_file(path).await;
        match res {
            Ok(response) => Ok(response),
            Err(err) => {
                log::error!("Can't serve '{}' file: {}", path.display(), err);
                let response = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())?;
                Ok(response)
            }
        }
    }
}
