use crate::actors::worker::RillWorker;
use crate::state::ControlReceiver;
use crate::EntryId;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System};

pub(crate) struct RillSupervisor {
    entry_id: EntryId,
    rx: Option<ControlReceiver>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Worker,
    Exporters,
}

impl Actor for RillSupervisor {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillSupervisor({})", self.entry_id)
    }
}

impl RillSupervisor {
    pub fn new(entry_id: EntryId, rx: ControlReceiver) -> Self {
        Self {
            entry_id,
            rx: Some(rx),
        }
    }
}

#[async_trait]
impl StartedBy<System> for RillSupervisor {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Exporters, Group::Worker]);
        let worker = RillWorker::new(self.entry_id.clone());
        let rx = self
            .rx
            .take()
            .ok_or(Error::msg("attempt to start supervisor twice"))?;
        let mut worker = ctx.spawn_actor(worker, Group::Worker);
        worker.attach(rx);
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
        ctx.shutdown();
        Ok(())
    }
}
