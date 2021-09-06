use super::{Group, NodeSupervisor};
use crate::actors::config_watcher::ConfigWatcher;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Context, Eliminated, IdOf};

impl NodeSupervisor {
    pub(super) fn spawn_config_watcher(&mut self, ctx: &mut Context<Self>) {
        let config_watcher = ConfigWatcher::new();
        ctx.spawn_actor(config_watcher, Group::ConfigWatcher);
    }
}

#[async_trait]
impl Eliminated<ConfigWatcher> for NodeSupervisor {
    async fn handle(
        &mut self,
        _id: IdOf<ConfigWatcher>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
