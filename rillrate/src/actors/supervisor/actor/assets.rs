use super::{Group, NodeSupervisor};
use crate::actors::error::NotSet;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Context, Eliminated, IdOf, InteractionDone};
use meio_connect::server::HttpServerLink;
use rate_core::actors::app_bind::{AppBind, AssetsOptions};
use rate_core::actors::node::WaitHttpServer;

impl NodeSupervisor {
    // TODO: Avoid returning error here
    pub(super) fn spawn_assets(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let node = self.node.as_mut().ok_or(NotSet)?;
        // TODO: Change this bool.
        let task = node.wait_for_server(true);
        ctx.spawn_task(task, (), Group::Assets);
        Ok(())
    }
}

#[async_trait]
impl InteractionDone<WaitHttpServer, ()> for NodeSupervisor {
    async fn handle(
        &mut self,
        _: (),
        link: HttpServerLink,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let url = format!(
            "https://cdn.rillrate.com/live/v0/v{}.tar.gz",
            crate::meta::VERSION
        );
        let embedded = None;
        let options = AssetsOptions {
            prefix: "/ui/",
            env_var: Some("RILLRATE_UI"),
            url: Some(url.parse()?),
            embedded,
        };
        let app_bind = AppBind::new(link, options);
        ctx.spawn_actor(app_bind, Group::Assets);
        Ok(())
    }
}

#[async_trait]
impl Eliminated<AppBind> for NodeSupervisor {
    async fn handle(&mut self, _id: IdOf<AppBind>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
