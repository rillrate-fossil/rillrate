use crate::actors::supervisor::NodeSupervisor;
use crate::config::cases::CaseConfig;
use crate::config::server::RillRateConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{Action, ActionHandler, Actor, Context, InterruptedBy, StartedBy};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rate_config::ReadableConfig;
use rill_protocol::diff::diff_full;
use rill_protocol::io::provider::EntryId;
use rrpack_prime::manifest::layouts::global::LAYOUTS;
use rrpack_prime::manifest::layouts::layout::LayoutConfig;
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs;

pub struct ConfigWatcher {
    watcher: Option<RecommendedWatcher>,
    layouts: HashMap<EntryId, LayoutConfig>,
}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            layouts: HashMap::new(),
        }
    }
}

impl Actor for ConfigWatcher {
    type GroupBy = ();
}

struct Reload;

impl Action for Reload {}

#[async_trait]
impl StartedBy<NodeSupervisor> for ConfigWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let mut addr = ctx.address().clone();
        let mut watcher = RecommendedWatcher::new(move |res: Result<notify::Event, _>| {
            if let Ok(event) = res {
                let changed = event
                    .paths
                    .iter()
                    .any(|p| p.as_path().to_string_lossy().contains(".toml"));
                if changed {
                    if let Err(err) = addr.blocking_act(Reload) {
                        log::error!("Can't notify config watcher about config changes: {}", err);
                    } else {
                        log::info!("Config updated. Loading...");
                    }
                }
            }
        })?;
        watcher.watch(".rillrate".as_ref(), RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);

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
        if let Err(err) = self.read_config_impl().await {
            log::error!("Can't read config: {}", err);
        }
    }

    async fn read_config_impl(&mut self) -> Result<(), Error> {
        let mut dir = fs::read_dir(".rillrate/cases").await?;
        while let Some(entry) = dir.next_entry().await? {
            let meta = entry.metadata().await?;
            if meta.is_file() {
                //let case = CaseConfig::read(entry.path()).await?;
            }
        }
        Ok(())
    }

    /*
    async fn read_config(&mut self) {
        let config = RillRateConfig::read(".rillrate/config.toml".into()).await;
        match config {
            Ok(config) => {
                let (to_add, to_remove, to_check) =
                    diff_full(self.layouts.keys(), config.layout.keys());
                for name in to_add {
                    let layout_config = config.layout.get(&name).unwrap();
                    log::debug!("Add Layout: {}", name);
                    LAYOUTS.add_layout(name, layout_config.clone().into());
                }
                for name in to_remove {
                    log::debug!("Remove Layout: {}", name);
                    LAYOUTS.remove_layout(name);
                }
                for name in to_check {
                    log::debug!("Update Layout: {}", name);
                    let prev = self.layouts.get(&name).unwrap();
                    let new = config.layout.get(&name).unwrap();
                    if prev != new {
                        LAYOUTS.add_layout(name, new.clone().into());
                    }
                }
            }
            Err(err) => {
                log::error!("Can't read config: {}", err);
            }
        }
    }
    */
}

#[async_trait]
impl ActionHandler<Reload> for ConfigWatcher {
    async fn handle(&mut self, _event: Reload, _ctx: &mut Context<Self>) -> Result<(), Error> {
        self.read_config().await;
        Ok(())
    }
}

// TODO: How about to use plain actions for `HeartBeat`?
#[async_trait]
impl OnTick for ConfigWatcher {
    async fn tick(&mut self, _: Tick, _ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Check config exists
        self.read_config().await;
        Ok(())
    }

    async fn done(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
