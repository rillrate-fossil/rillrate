use crate::config::{Config, ReadConfigFile};
use crate::env;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy, System, TaskEliminated,
    TaskError,
};
use rill_engine::{ProviderConfig, RillEngine};
use rill_export::{ExportConfig, RillExport};
use rill_server::{AddrCell, RillHub};
use std::marker::PhantomData;
use std::net::SocketAddr;

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

    fn spawn_exporter(&mut self, ctx: &mut Context<Self>) {
        let config = self.export_config.take().unwrap_or_default();
        let actor = RillExport::new(config);
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
impl Eliminated<RillHub> for RillRate {
    async fn handle(&mut self, _id: IdOf<RillHub>, ctx: &mut Context<Self>) -> Result<(), Error> {
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
            let actor = RillHub::new(config.server, config.export);
            ctx.spawn_actor(actor, Group::Hub);

            let task = WaitForAddr::<RillEngine>::new(&rill_server::INTERN_ADDR);
            ctx.spawn_task(task, Group::Tuning);

            let task = WaitForAddr::<RillExport>::new(&rill_server::EXTERN_ADDR);
            ctx.spawn_task(task, Group::Tuning);
        }

        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WaitForAddr<RillEngine>> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<WaitForAddr<RillEngine>>,
        res: Result<SocketAddr, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match res {
            Ok(addr) => {
                log::info!("Connecting engine (provider) to {}", addr);
                rill_engine::config::NODE.offer(addr.to_string());
                self.spawn_engine(ctx);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl TaskEliminated<WaitForAddr<RillExport>> for RillRate {
    async fn handle(
        &mut self,
        _id: IdOf<WaitForAddr<RillExport>>,
        res: Result<SocketAddr, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match res {
            Ok(addr) => {
                log::info!("Connecting exporter to {}", addr);
                rill_export::config::NODE.offer(addr.to_string());
                self.spawn_exporter(ctx);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

struct WaitForAddr<T> {
    cell: AddrCell,
    _waiter: PhantomData<T>,
}

impl<T> WaitForAddr<T> {
    fn new(cell: AddrCell) -> Self {
        Self {
            cell,
            _waiter: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Actor> LiteTask for WaitForAddr<T> {
    type Output = SocketAddr;

    async fn interruptable_routine(self) -> Result<Self::Output, Error> {
        let intern_rx = self
            .cell
            .lock()
            .await
            .1
            .take()
            .ok_or_else(|| Error::msg("intern address notifier not found"))?;
        intern_rx.await.map_err(Error::from)
    }
}
