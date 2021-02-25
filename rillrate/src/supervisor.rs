use crate::config::{Config, ReadConfigFile};
use crate::env;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy, System, TaskEliminated,
    TaskError,
};
use rill::RillEngine;
use rill_export::EmbeddedNode;
use std::net::SocketAddr;

pub struct RillRate {
    app_name: String,
    // TODO: Keep node addr here as `Option`
    // and if it's not configured than spawn a standalone server
    // and with for it install the port here and spawn a tracer.
}

impl RillRate {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }

    fn spawn_tracer(&mut self, node: String, ctx: &mut Context<Self>) {
        // TODO: Use the same config
        let name = env::name(Some(self.app_name.clone()));
        let actor = RillEngine::new(node, name);
        ctx.spawn_actor(actor, Group::Engine);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Tuning,
    Engine,
    EmbeddedNode,
}

impl Actor for RillRate {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Tuning, Group::Engine, Group::EmbeddedNode]);

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
impl Eliminated<EmbeddedNode> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<EmbeddedNode>,
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
            .unwrap_or_default();

        // TODO: Check config for node as well
        if let Some(node) = env::node() {
            self.spawn_tracer(node, ctx);
        } else {
            let actor = EmbeddedNode::new(config.server, config.export);
            ctx.spawn_actor(actor, Group::EmbeddedNode);
            let task = WaitForAddr::new();
            ctx.spawn_task(task, Group::Tuning);
        }

        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WaitForAddr> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<WaitForAddr>,
        res: Result<SocketAddr, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match res {
            Ok(addr) => {
                log::info!("Connecting tracer to {}", addr);
                self.spawn_tracer(addr.to_string(), ctx);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

struct WaitForAddr {}

impl WaitForAddr {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl LiteTask for WaitForAddr {
    type Output = SocketAddr;

    async fn interruptable_routine(self) -> Result<Self::Output, Error> {
        let intern_rx = rill_export::INTERN_ADDR
            .lock()
            .await
            .1
            .take()
            .ok_or_else(|| Error::msg("intern address notifier not found"))?;
        intern_rx.await.map_err(Error::from)
    }
}
