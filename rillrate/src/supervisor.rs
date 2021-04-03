use crate::config::{Config, ReadConfigFile};
use crate::env;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System, TaskEliminated, TaskError,
};
use rill_engine::{EngineConfig, RillEngine};

pub struct RillRate {}

impl RillRate {
    pub fn new(app_name: String) -> Self {
        // TODO: Use MyOnceCell here that will inform that it was set
        rill_engine::config::NAME.offer(app_name.into());
        Self {}
    }

    fn spawn_engine(&mut self, config: EngineConfig, ctx: &mut Context<Self>) {
        let actor = RillEngine::new(config);
        ctx.spawn_actor(actor, Group::Provider);
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

    fn name(&self) -> String {
        "RillRate".into()
    }
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
        ctx.spawn_task(config_task, (), Group::Tuning);

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
impl TaskEliminated<ReadConfigFile, ()> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<ReadConfigFile>,
        _tag: (),
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

        let engine_config = config.rillrate.unwrap_or_default();
        log::info!("Remote node set.");
        self.spawn_engine(engine_config, ctx);
        /*
        let export_config = config.export.unwrap_or_default();
        if engine_config.is_node_specified() {
            log::info!("Remote node set.");
            self.spawn_engine(engine_config, ctx);
        } else {
            log::info!("Local node set.");
            // If node wasn't specified than spawn an embedded node and
            // wait for the address to spawn a provider connected to that.
            let actor = RillServer::new(config.server);
            let server: ServerLink = ctx.spawn_actor(actor, Group::Hub).link();

            // TODO: Add timeout here
            let public_http = server.wait_public_endpoint().recv().await?;

            // TODO: Add timeout here
            let private_http = server.wait_private_endpoint().recv().await?;

            let addr = private_http.wait_for_address().recv().await?;
            log::info!("Connecting engine (provider) to {}", addr);
            rill_engine::config::NODE.offer(addr.to_string());
            self.spawn_engine(engine_config, ctx);
            self.spawn_exporter(export_config, public_http, ctx);
        }
        */
        Ok(())
    }
}
