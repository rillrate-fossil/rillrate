use crate::actors::storage::RillStorage;
use crate::actors::worker::RillWorker;
use crate::config::RillConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};

pub(crate) struct RillSupervisor {
    config: RillConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Worker,
    Storage,
}

impl Actor for RillSupervisor {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillSupervisor({})", self.config.entry_id())
    }
}

impl RillSupervisor {
    pub fn new(config: RillConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl StartedBy<System> for RillSupervisor {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Worker, Group::Storage]);
        let storage = RillStorage::new();
        ctx.spawn_actor(storage, Group::Storage);

        let worker = RillWorker::new(self.config.clone());
        ctx.spawn_actor(worker, Group::Worker);

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
