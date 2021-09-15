mod app_route;
mod assets;

use anyhow::Error;
use assets::AssetsMode;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use reqwest::Url;

pub struct AssetsOptions {
    pub prefix: &'static str,
    pub env_var: Option<&'static str>,
    pub url: Option<Url>,
    pub embedded: Option<Vec<u8>>,
}

pub struct AppBind {
    server: HttpServerLink,
    assets: AssetsMode,
    options: AssetsOptions,
}

impl AppBind {
    pub fn new(server: HttpServerLink, options: AssetsOptions) -> Self {
        Self {
            server,
            assets: AssetsMode::Loading,
            options,
        }
    }
}

impl Actor for AppBind {
    type GroupBy = ();
}

#[async_trait]
impl<T: Actor> StartedBy<T> for AppBind {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.app_bind_route(ctx).await?;
        self.init_assets(ctx).await?;
        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for AppBind {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Terminating the app...");
        ctx.shutdown();
        Ok(())
    }
}
