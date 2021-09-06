use crate::actors::supervisor::NodeSupervisor;
use crate::config::RillRateConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{Actor, Context, InterruptedBy, StartedBy};
use rate_config::ReadableConfig;
use rrpack_prime::manifest::layouts::global::LAYOUTS;
use tokio::time::Duration;

pub struct ConfigWatcher {}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for ConfigWatcher {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<NodeSupervisor> for ConfigWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        /* TODO: Implement live updates
        let interval = Duration::from_secs(5);
        let _heartbeat = HeartBeat::new(interval, ctx.address().clone());
        */
        self.read_config().await;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<NodeSupervisor> for ConfigWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

impl ConfigWatcher {
    async fn read_config(&mut self) {
        let config = RillRateConfig::read("rillrate.toml".into()).await;
        match config {
            Ok(config) => {
                //if let Some(layouts) = config.layout {
                for (name, layout) in config.layout {
                    log::debug!("Add Layout: {}", layout.name);
                    LAYOUTS.add_layout(name.into(), layout);
                }
                //}
            }
            Err(err) => {
                log::error!("Can't read config: {}", err);
            }
        }
    }
}

#[async_trait]
impl OnTick for ConfigWatcher {
    async fn tick(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.read_config().await;
        Ok(())
    }

    async fn done(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
