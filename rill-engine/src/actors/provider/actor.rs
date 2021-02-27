use crate::actors::storage::RillStorage;
use crate::actors::worker::RillWorker;
use crate::config::ProviderConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy};

/// The supervisor that spawns a worker.
pub struct RillProvider {
    config: Option<ProviderConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Worker,
    Storage,
}

impl Actor for RillProvider {
    type GroupBy = Group;

    /*
    fn name(&self) -> String {
        format!("RillProvider({})", self.config.entry_id())
    }
    */
}

impl RillProvider {
    /// Creates a new supervisor instance.
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillProvider {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Worker, Group::Storage]);
        let storage = RillStorage::new();
        ctx.spawn_actor(storage, Group::Storage);

        let worker = RillWorker::new(self.config.take());
        ctx.spawn_actor(worker, Group::Worker);

        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for RillProvider {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillWorker> for RillProvider {
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
impl Eliminated<RillStorage> for RillProvider {
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
