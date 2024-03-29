use crate::config::cases::Layout;
use anyhow::Error;
use async_trait::async_trait;
use meio::task::{HeartBeat, OnTick, Tick};
use meio::{Action, ActionHandler, Actor, Context, InterruptedBy, StartedBy, TaskAddress};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rate_core::assets::Assets;
use rill_protocol::diff::diff_full;
use rill_protocol::io::provider::Path;
use rrpack_basis::manifest::layouts::global::LAYOUTS;
use std::collections::HashMap;
use std::path::Path as FilePath;
use std::time::Duration;
use strum::{EnumIter, IntoEnumIterator};
use tokio::fs;

const CASE_EXT: &str = "xml";
const PATH: &str = ".rillrate";

pub struct ConfigWatcher {
    watcher: Option<RecommendedWatcher>,
    layouts: HashMap<Path, Layout>,
    heartbeat: Option<TaskAddress<HeartBeat>>,
}

impl ConfigWatcher {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            watcher: None,
            layouts: HashMap::new(),
            heartbeat: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Group {
    HeartBeat,
}

impl Actor for ConfigWatcher {
    type GroupBy = Group;
}

struct Reload;

impl Action for Reload {}

#[async_trait]
impl<T: Actor> StartedBy<T> for ConfigWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(Group::iter().collect());
        self.read_and_watch(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for ConfigWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

impl ConfigWatcher {
    async fn read_and_watch(&mut self, ctx: &mut Context<Self>) {
        let mut success = true;
        if let Err(err) = self.read_from_tar() {
            log::error!("Can't read embedded config tar: {}", err);
        }
        if FilePath::new(PATH).exists() {
            success &= self.read_from_dir().await.is_ok();
            success &= self.assign_watcher(ctx).is_ok();
        }
        if !success {
            self.unassign_watcher();
            self.start_heartbeat(ctx);
        } else {
            self.stop_heartbeat();
        }
    }

    fn start_heartbeat(&mut self, ctx: &mut Context<Self>) {
        if self.heartbeat.is_none() {
            let interval = Duration::from_secs(5);
            let heartbeat = HeartBeat::new(interval, ctx.address().clone());
            let addr = ctx.spawn_task(heartbeat, (), Group::HeartBeat);
            self.heartbeat = Some(addr);
        }
    }

    fn stop_heartbeat(&mut self) {
        if let Some(heartbeat) = self.heartbeat.take() {
            if let Err(err) = heartbeat.stop() {
                log::error!(
                    "Can't stop the HeartBeat task of the ConfigWatcher: {}",
                    err
                );
            }
        }
    }
}

impl ConfigWatcher {
    fn assign_watcher(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        if self.watcher.is_none() {
            let mut addr = ctx.address().clone();
            let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
                log::warn!("Res {:?}", res);
                if let Ok(event) = res {
                    let changed = event
                        .paths
                        .iter()
                        .any(|p| p.as_path().extension() == Some(CASE_EXT.as_ref()));
                    if changed {
                        if let Err(err) = addr.blocking_act(Reload) {
                            log::error!(
                                "Can't notify config watcher about config changes: {}",
                                err
                            );
                        } else {
                            log::info!("Config updated. Loading...");
                        }
                    }
                }
            })?;
            watcher.watch(PATH.as_ref(), RecursiveMode::Recursive)?;
            self.watcher = Some(watcher);
            log::info!("Config watcher activated");
        }
        Ok(())
    }

    fn unassign_watcher(&mut self) {
        self.watcher.take();
    }

    fn read_from_tar(&mut self) -> Result<(), Error> {
        // TODO: Skip that step using config
        if let Some(data) = crate::preserved::PRESERVED.get() {
            let assets = Assets::parse(data)?;
            for (path, data) in assets.iter() {
                if path.contains("cases") {
                    let layout: Result<Layout, _> = serde_xml_rs::from_reader(data);
                    match layout {
                        Ok(layout) => {
                            let path = layout.name.clone();
                            log::debug!("Add Embedded Layout: {}", path);
                            // Embedded layouts aren't tracked by the `self.layouts` map
                            // and they exists always.
                            LAYOUTS.add_tab(path, layout.into());
                        }
                        Err(err) => {
                            log::error!("Can't parse embedded layout file {}: {}", path, err);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn read_from_dir(&mut self) -> Result<(), Error> {
        log::info!("Reading confing files");
        let mut dir = fs::read_dir(".rillrate/cases").await?;
        let mut layouts = HashMap::new();
        while let Some(entry) = dir.next_entry().await? {
            let meta = entry.metadata().await?;
            if meta.is_file() && entry.path().extension() == Some(CASE_EXT.as_ref()) {
                let path = entry.path();
                let data = fs::read_to_string(path.as_path()).await?;
                let layout: Result<Layout, _> = serde_xml_rs::from_str(&data);
                match layout {
                    Ok(layout) => {
                        layouts.insert(layout.name.clone(), layout);
                    }
                    Err(err) => {
                        log::error!("Can't parse layout file {}: {}", path.display(), err);
                    }
                }
            }
        }
        let (to_add, to_remove, to_check) = diff_full(self.layouts.keys(), layouts.keys());
        for name in to_add {
            let layout = layouts.get(&name).unwrap();
            log::debug!("Add Layout: {}", name);
            LAYOUTS.add_tab(name.clone(), layout.clone().into());
            self.layouts.insert(name, layout.clone());
        }
        for name in to_remove {
            log::debug!("Remove Layout: {}", name);
            LAYOUTS.remove_tab(name.clone());
            self.layouts.remove(&name);
        }
        for name in to_check {
            log::debug!("Update Layout: {}", name);
            let prev = self.layouts.get(&name).unwrap();
            let layout = layouts.get(&name).unwrap();
            if prev != layout {
                LAYOUTS.add_tab(name.clone(), layout.clone().into());
                self.layouts.insert(name, layout.clone());
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<Reload> for ConfigWatcher {
    async fn handle(&mut self, _event: Reload, _ctx: &mut Context<Self>) -> Result<(), Error> {
        self.read_from_dir().await
    }
}

// TODO: How about to use plain actions for `HeartBeat`?
#[async_trait]
impl OnTick for ConfigWatcher {
    async fn tick(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.read_and_watch(ctx).await;
        Ok(())
    }

    async fn done(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}
