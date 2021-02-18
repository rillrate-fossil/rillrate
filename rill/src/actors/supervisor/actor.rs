use crate::actors::router::RillRouter;
use crate::actors::storage::RillStorage;
//use crate::actors::worker::RillWorker;
use crate::config::RillConfig;
use crate::state::ControlReceiver;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};

pub(crate) struct RillSupervisor {
    config: RillConfig,
    rx: Option<ControlReceiver>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Exporters,
    Router,
    //Worker,
    Storage,
}

impl Actor for RillSupervisor {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillSupervisor({})", self.config.entry_id())
    }
}

impl RillSupervisor {
    pub fn new(config: RillConfig, rx: ControlReceiver) -> Self {
        Self {
            config,
            rx: Some(rx),
        }
    }
}

#[async_trait]
impl StartedBy<System> for RillSupervisor {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::Exporters,
            Group::Router,
            //Group::Worker,
            Group::Storage,
        ]);
        let storage = RillStorage::new();
        ctx.spawn_actor(storage, Group::Storage);

        /*
        let worker = RillWorker::new(self.config.clone());
        ctx.spawn_actor(worker, Group::Worker);
        */

        let router = RillRouter::new(self.config.clone());
        let mut router_addr = ctx.spawn_actor(router, Group::Router);
        let rx = self
            .rx
            .take()
            .ok_or_else(|| Error::msg("attempt to start supervisor twice"))?;
        router_addr.attach(rx)?;

        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for RillSupervisor {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

/*
#[async_trait]
impl Eliminated<RillWorker> for RillSupervisor {
    async fn handle(
        &mut self,
        _id: IdOf<RillWorker>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Do we really need it here?
        ctx.shutdown();
        Ok(())
    }
}
*/

#[async_trait]
impl Eliminated<RillRouter> for RillSupervisor {
    async fn handle(
        &mut self,
        _id: IdOf<RillRouter>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Do we really need it here?
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillStorage> for RillSupervisor {
    async fn handle(
        &mut self,
        _id: IdOf<RillStorage>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Do we really need it here?
        ctx.shutdown();
        Ok(())
    }
}
