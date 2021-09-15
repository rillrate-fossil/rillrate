use super::Router;
use crate::actors::supervisor::Supervisor;
use crate::connection_limiter::Limit;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Action, ActionHandler, Context};

pub struct ChangeLimits {
    pub clients_limit: Limit,
    pub providers_limit: Limit,
}

impl Action for ChangeLimits {}

#[async_trait]
impl<T: Supervisor> ActionHandler<ChangeLimits> for Router<T> {
    async fn handle(&mut self, limits: ChangeLimits, ctx: &mut Context<Self>) -> Result<(), Error> {
        let denied_providers = self.active_providers.set_limit(limits.providers_limit);
        for mut provider in denied_providers {
            ctx.interrupt(&mut provider).ok();
        }
        let denied_clients = self.active_clients.set_limit(limits.clients_limit);
        for mut client in denied_clients {
            ctx.interrupt(&mut client).ok();
        }
        Ok(())
    }
}
