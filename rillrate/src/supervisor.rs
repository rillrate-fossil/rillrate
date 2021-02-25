use crate::env;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy, System, TaskEliminated,
    TaskError,
};
use rill::RillEngine;
use rill_export::{AddrReceiver, EmbeddedNode};
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
    AddrWatchers,
    Engine,
    EmbeddedNode,
}

impl Actor for RillRate {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<System> for RillRate {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::AddrWatchers,
            Group::Engine,
            Group::EmbeddedNode,
        ]);

        let config_path = Some(env::config());

        if let Some(node) = env::node() {
            self.spawn_tracer(node, ctx);
        } else {
            let actor = EmbeddedNode::new(config_path);
            ctx.spawn_actor(actor, Group::EmbeddedNode);

            // TODO: Dry. Add special class for that to `meio`
            let _extern_rx = rill_export::EXTERN_ADDR
                .lock()
                .await
                .1
                .take()
                .ok_or_else(|| Error::msg("extern address notifier not found"))?;
            let intern_rx = rill_export::INTERN_ADDR
                .lock()
                .await
                .1
                .take()
                .ok_or_else(|| Error::msg("intern address notifier not found"))?;

            let task = WaitForAddr::new(intern_rx);
            ctx.spawn_task(task, Group::AddrWatchers);
        }

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

struct WaitForAddr {
    receiver: AddrReceiver,
}

impl WaitForAddr {
    fn new(receiver: AddrReceiver) -> Self {
        Self { receiver }
    }
}

#[async_trait]
impl LiteTask for WaitForAddr {
    type Output = SocketAddr;

    async fn interruptable_routine(self) -> Result<Self::Output, Error> {
        self.receiver.await.map_err(Error::from)
    }
}
