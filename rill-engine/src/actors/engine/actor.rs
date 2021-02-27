//use crate::actors::storage::RillStorage;
use crate::actors::worker::RillWorker;
use crate::config::ProviderConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy};

/// The supervisor that spawns a worker.
pub struct RillEngine {
    config: Option<ProviderConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Worker,
    Storage,
}

impl Actor for RillEngine {
    type GroupBy = Group;

    /*
    fn name(&self) -> String {
        format!("RillEngine({})", self.config.entry_id())
    }
    */
}

impl RillEngine {
    /// Creates a new supervisor instance.
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillEngine {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Worker, Group::Storage]);

        /*
        let storage = RillStorage::new();
        ctx.spawn_actor(storage, Group::Storage);
        */

        let worker = RillWorker::new(self.config.take());
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

/*
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
*/
