use crate::actors::supervisor::NodeSupervisor;
use crate::config::RillRateConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{Actor, Context, InterruptedBy, StartedBy};
use rate_config::ReadableConfig;
use rrpack_prime::manifest::layouts::global::LAYOUTS;
use rrpack_prime::manifest::layouts::layout::Layout;
use std::time::Duration;

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
        let interval = Duration::from_secs(5);
        let heartbeat = HeartBeat::new(interval, ctx.address().clone());
        ctx.spawn_task(heartbeat, (), ());
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
                for (name, layout_config) in config.layout {
                    let layout: Layout = layout_config.into();
                    log::debug!("Add Layout: {}", layout.name);
                    LAYOUTS.add_layout(name, layout);
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
    async fn tick(&mut self, _: Tick, _ctx: &mut Context<Self>) -> Result<(), Error> {
        self.read_config().await;
        Ok(())
    }

    async fn done(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
