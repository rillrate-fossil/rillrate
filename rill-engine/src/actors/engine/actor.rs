use crate::actors::connector::RillConnector;
use crate::config::EngineConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy};
use rill_protocol::io::provider::EntryId;

/// The supervisor that spawns a connector.
pub struct RillEngine {
    name: EntryId,
    /// It wrapped with `Option` to take it for a `Worker` instance later.
    config: Option<EngineConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Worker,
    Storage,
}

impl Actor for RillEngine {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillEngine({})", &self.name)
    }
}

impl RillEngine {
    /// Creates a new supervisor instance.
    pub fn new(config: EngineConfig) -> Self {
        let name = config.provider_name();
        Self {
            name,
            config: Some(config),
        }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillEngine {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Worker, Group::Storage]);

        let config = self.config.take().unwrap();
        let connector = RillConnector::new(config);
        ctx.spawn_actor(connector, Group::Worker);

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
impl Eliminated<RillConnector> for RillEngine {
    async fn handle(
        &mut self,
        _id: IdOf<RillConnector>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Do we really need it here?
        ctx.shutdown();
        Ok(())
    }
}
