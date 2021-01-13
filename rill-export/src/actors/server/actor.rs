use super::Assets;
use crate::actors::client_session::ClientSession;
use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::exporter::Exporter;
use crate::actors::provider_session::ProviderSession;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Address, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy,
    LiteTask, StartedBy, TaskEliminated, TaskError,
};
use meio_connect::headers::{ContentType, HeaderMapExt, HeaderValue};
use meio_connect::hyper::{header, Body, Request, Response, StatusCode};
use meio_connect::server::{DirectPath, FromRequest, HttpServerLink, Req, WsReq};
use rill_protocol::provider::RillProtocol;
use rill_protocol::view::ViewProtocol;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn read_file(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path).await?;
    let mut content = Vec::new();
    file.read_to_end(&mut content).await?;
    Ok(content)
}

enum AssetsMode {
    Loading,
    Local(PathBuf),
    Packed(Assets),
    //Proxy(Uri),
}

pub struct Server {
    inner_server: HttpServerLink,
    extern_server: HttpServerLink,
    exporter: Address<Exporter>,
    connected: bool,
    assets: AssetsMode,
}

impl Server {
    pub fn new(
        inner_server: HttpServerLink,
        extern_server: HttpServerLink,
        exporter: Address<Exporter>,
    ) -> Self {
        Self {
            inner_server,
            extern_server,
            exporter,
            connected: false,
            assets: AssetsMode::Loading,
        }
    }

    async fn read_assets(&mut self, path: &str) -> Result<AssetsMode, Error> {
        let ui_path = Path::new(path).to_path_buf();
        if ui_path.exists() {
            let metadata = tokio::fs::metadata(&ui_path).await?;
            if metadata.is_dir() {
                Ok(AssetsMode::Local(ui_path))
            } else {
                let data = read_file(&ui_path).await?;
                let assets = Assets::parse(&data)?;
                Ok(AssetsMode::Packed(assets))
            }
        } else {
            Err(Error::msg(format!("Can't load assets from {}", path)))
        }
    }

    async fn init_assets(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        if let Ok(path) = std::env::var("_ASSETS_") {
            self.assets = self.read_assets(&path).await?;
            log::warn!("Assets overriden to: {}", path);
        } else {
            ctx.spawn_task(FetchUiPack, ());
        }
        Ok(())
    }
}

impl Actor for Server {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Server {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.init_assets(ctx).await?;
        self.inner_server
            .add_route(Index, ctx.address().clone())
            .await?;
        self.inner_server
            .add_ws_route(ProviderLive, ctx.address().clone())
            .await?;

        self.extern_server
            .add_route(ForwardToUi, ctx.address().clone())
            .await?;
        self.extern_server
            .add_ws_route(ClientLive, ctx.address().clone())
            .await?;
        self.extern_server
            .add_route(Ui, ctx.address().clone())
            .await?;
        self.extern_server
            .add_route(Info, ctx.address().clone())
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
    type Parameter = ();
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
        let response = Response::builder().body(Body::from("Rill Export Inner Server"))?;
        Ok(response)
    }
}

#[derive(Default)]
struct ForwardToUi;

impl DirectPath for ForwardToUi {
    type Parameter = ();
    fn paths() -> &'static [&'static str] {
        &["/", "/index.html"]
    }
}

#[async_trait]
impl InteractionHandler<Req<ForwardToUi>> for Server {
    async fn handle(
        &mut self,
        _: Req<ForwardToUi>,
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
    type Parameter = ();
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
    type Parameter = RillProtocol;

    fn paths() -> &'static [&'static str] {
        &["/live/provider"]
    }
}

#[async_trait]
impl ActionHandler<WsReq<ProviderLive>> for Server {
    async fn handle(
        &mut self,
        req: WsReq<ProviderLive>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            if !self.connected {
                self.connected = true;
                let session_actor = ProviderSession::new(req.stream, self.exporter.link());
                ctx.spawn_actor(session_actor, ());
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
        // It allows to connect again
        self.connected = false;
        Ok(())
    }
}

#[derive(Default)]
struct ClientLive;

impl DirectPath for ClientLive {
    type Parameter = ViewProtocol;

    fn paths() -> &'static [&'static str] {
        &["/live/client"]
    }
}

#[async_trait]
impl ActionHandler<WsReq<ClientLive>> for Server {
    async fn handle(
        &mut self,
        req: WsReq<ClientLive>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            let session_actor = ClientSession::new(req.stream, self.exporter.link());
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
struct Ui;

struct UiReq {
    tail: PathBuf,
}

impl FromRequest for Ui {
    type Output = UiReq;

    fn from_request(request: &Request<Body>) -> Option<Self::Output> {
        let path = request.uri().path();
        if path.starts_with("/ui/") {
            let tail = Path::new(&path[4..]).to_path_buf();
            Some(UiReq { tail })
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
        use thiserror::Error;
        #[derive(Debug, Error)]
        enum Fail {
            #[error("wrong path")]
            WrongPath,
            #[error("not found")]
            NotFound,
        }

        match &self.assets {
            AssetsMode::Packed(assets) => {
                let path = path.to_str().ok_or(Fail::WrongPath)?;
                let content = assets.get(path).ok_or(Fail::NotFound)?.to_vec();
                Ok(content)
            }
            AssetsMode::Local(ui_path) => {
                let mut full_path = ui_path.clone();
                full_path.push(path);
                log::warn!(
                    "Read overriden file asset from the path: {}",
                    full_path.display()
                );
                let content = read_file(&full_path).await?;
                Ok(content)
            }
            AssetsMode::Loading => {
                todo!();
            }
        }
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

#[async_trait]
impl TaskEliminated<FetchUiPack> for Server {
    async fn handle(
        &mut self,
        _id: IdOf<FetchUiPack>,
        result: Result<Assets, TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match result {
            Ok(assets) => {
                self.assets = AssetsMode::Packed(assets);
                log::info!("Assets pack attached.");
                Ok(())
            }
            Err(err) => {
                // TODO: Schedule refetching...
                log::error!("Can't load UI pack: {}", err);
                Err(err.into())
            }
        }
    }
}

pub struct FetchUiPack;

#[async_trait]
impl LiteTask for FetchUiPack {
    type Output = Assets;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        log::info!("Fetching UI assets...");
        let bytes = reqwest::get("http://ui.rillrate.com/rate-ui.tar.gz")
            .await?
            .bytes()
            .await?;
        Assets::parse(&bytes)
    }
}
