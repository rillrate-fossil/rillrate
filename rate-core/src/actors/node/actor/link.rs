use super::{Node, NodeLink};
use crate::actors::router::actor::limits::ChangeLimits;
use crate::actors::supervisor::Supervisor;
use crate::connection_limiter::Limit;
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Context, Interaction, InteractionHandler, InteractionTask};
use meio_connect::server::HttpServerLink;

pub struct WaitHttpServer {
    external: bool,
}

impl Interaction for WaitHttpServer {
    type Output = HttpServerLink;
}

impl<T: Supervisor> NodeLink<T> {
    pub fn wait_for_server(&self, external: bool) -> InteractionTask<WaitHttpServer> {
        self.address.interact(WaitHttpServer { external })
    }
}

#[async_trait]
impl<T: Supervisor> InteractionHandler<WaitHttpServer> for Node<T> {
    async fn handle(
        &mut self,
        msg: WaitHttpServer,
        _ctx: &mut Context<Self>,
    ) -> Result<HttpServerLink, Error> {
        let link;
        if msg.external {
            link = self.external_server.clone();
        } else {
            link = self.internal_server.clone();
        }
        link.ok_or_else(|| Error::msg("no http endpoint attached"))
    }
}

impl<T: Supervisor> NodeLink<T> {
    pub async fn change_limits(
        &mut self,
        clients_limit: Limit,
        providers_limit: Limit,
    ) -> Result<(), Error> {
        let msg = ChangeLimits {
            clients_limit,
            providers_limit,
        };
        self.address.act(msg).await
    }
}

#[async_trait]
impl<T: Supervisor> ActionHandler<ChangeLimits> for Node<T> {
    async fn handle(
        &mut self,
        limits: ChangeLimits,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // Just forwards limits
        self.router()?.act(limits).await?;
        Ok(())
    }
}
