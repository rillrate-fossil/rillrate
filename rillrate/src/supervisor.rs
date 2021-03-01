use crate::config::{Config, ReadConfigFile};
use crate::env;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System, TaskEliminated, TaskError,
};
use meio_connect::server::HttpServerLink;
use rill_engine::{ProviderConfig, RillEngine};
use rill_export::{ExportConfig, RillExport};
use rill_server::{RillServer, ServerLink};

pub struct RillRate {
    // TODO: Keep node addr here as `Option`
    // and if it's not configured than spawn a standalone server
    // and with for it install the port here and spawn a tracer.
    provider_config: Option<ProviderConfig>,
    export_config: Option<ExportConfig>,
}

impl RillRate {
    pub fn new(app_name: String) -> Self {
        // TODO: Use MyOnceCell here that will inform that it was set
        rill_engine::config::NAME.offer(app_name.into());
        Self {
            provider_config: None,
            export_config: None,
        }
    }

    fn spawn_engine(&mut self, ctx: &mut Context<Self>) {
        let config = self.provider_config.take().unwrap_or_default();
        let actor = RillEngine::new(config);
        ctx.spawn_actor(actor, Group::Provider);
    }

    fn spawn_exporter(&mut self, server: HttpServerLink, ctx: &mut Context<Self>) {
        let config = self.export_config.take().unwrap_or_default();
        let actor = RillExport::new(config, server);
        ctx.spawn_actor(actor, Group::Exporter);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Tuning,
    Exporter,
    Provider,
    Hub,
}

impl Actor for RillRate {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::Tuning,
            Group::Exporter,
            Group::Provider,
            Group::Hub,
        ]);

        let config_path = env::config();
        let config_task = ReadConfigFile(config_path);
        ctx.spawn_task(config_task, Group::Tuning);

        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillEngine> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<RillEngine>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillExport> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<RillExport>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillServer> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<RillServer>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<ReadConfigFile> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<ReadConfigFile>,
        result: Result<Option<Config>, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let config = result
            .map_err(|err| {
                log::warn!(
                    "Can't read config file. No special configuration parameters applied: {}",
                    err
                );
            })
            .ok()
            .and_then(std::convert::identity)
            .unwrap_or_else(|| {
                log::warn!("Default config will be used.");
                Config::default()
            });

        self.provider_config = config.rillrate;
        let node_specified = self
            .provider_config
            .as_ref()
            .map(ProviderConfig::is_node_specified)
            .unwrap_or(false);
        if node_specified {
            self.spawn_engine(ctx);
        } else {
            // If node wasn't specified than spawn an embedded node and
            // wait for the address to spawn a provider connected to that.
            let actor = RillServer::new(config.server);
            let server: ServerLink = ctx.spawn_actor(actor, Group::Hub).link();

            // TODO: Add timeout here
            let public_http = server.wait_public_endpoint().recv().await?;
            //ctx.track_interaction(task, Group::Tuning);

            // TODO: Add timeout here
            let private_http = server.wait_private_endpoint().recv().await?;
            //ctx.track_interaction(task, Group::Tuning);

            let addr = private_http.wait_for_address().recv().await?;
            log::info!("Connecting engine (provider) to {}", addr);
            rill_engine::config::NODE.offer(addr.to_string());
            self.spawn_engine(ctx);
            self.spawn_exporter(public_http, ctx);
        }
        Ok(())
    }
}
