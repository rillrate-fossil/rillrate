use crate::actors::storage::RillStorage;
use crate::actors::worker::RillWorker;
use crate::config::RillConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};
use rill_protocol::provider::EntryId;

/// The supervisor that spawns a worker.
pub struct RillEngine {
    config: RillConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Worker,
    Storage,
}

impl Actor for RillEngine {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillEngine({})", self.config.entry_id())
    }
}

impl RillEngine {
    /// Creates a new supervisor instance.
    pub fn new(host: String, name: impl Into<EntryId>) -> Self {
        let config = RillConfig::new(host, name.into());
        Self { config }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillEngine {
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
impl<T: Actor> InterruptedBy<T> for RillEngine {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillWorker> for RillEngine {
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
impl Eliminated<RillStorage> for RillEngine {
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
