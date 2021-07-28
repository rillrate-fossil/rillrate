use super::{
    assets::{self, AssetsMode},
    AppBind,
};
use anyhow::Error;
use async_trait::async_trait;
use meio::{Context, InteractionHandler};
use meio_connect::headers::{ContentType, HeaderMapExt};
use meio_connect::hyper::{Body, Request, Response, StatusCode};
use meio_connect::server::{FromRequest, Req, WebRoute};
use std::path::{Path, PathBuf};

impl AppBind {
    pub(super) async fn app_bind_route(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let app_route = AppRoute {
            prefix: self.options.prefix,
        };
        let route = WebRoute::new(app_route, ctx.address().clone());
        self.server.add_route(route).await?;
        Ok(())
    }
}

pub struct AppRoute {
    prefix: &'static str,
}

pub struct AppRouteReq {
    tail: PathBuf,
}

impl FromRequest for AppRoute {
    type Output = AppRouteReq;

    fn from_request(&self, request: &Request<Body>) -> Result<Option<Self::Output>, Error> {
        let path = request.uri().path();
        if let Some(stripped) = path.strip_prefix(self.prefix) {
            let tail = Path::new(stripped).to_path_buf();
            Ok(Some(AppRouteReq { tail }))
        } else {
            Ok(None)
        }
    }
}

impl AppBind {
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
                let content = assets::read_file(&full_path).await?;
                Ok(content)
            }
            AssetsMode::Loading => Err(Error::msg("UI assets not loaded yet...")),
            AssetsMode::Failed(reason) => Err(Error::msg(format!("Can't load UI: {}", reason))),
        }
    }
}

#[async_trait]
impl InteractionHandler<Req<AppRoute>> for AppBind {
    async fn handle(
        &mut self,
        msg: Req<AppRoute>,
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
                let reason = err.to_string();
                let response = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(reason))?;
                Ok(response)
            }
        }
    }
}
