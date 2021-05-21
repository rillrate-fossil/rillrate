pub mod parcel;

use crate::actors::engine::RillEngine;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy};

pub struct RillPool {}

impl RillPool {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    ParcelStream,
    Tasks,
}

impl Actor for RillPool {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<RillEngine> for RillPool {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::ParcelStream, Group::Tasks]);
        self.attach_distributor(ctx).await?;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillEngine> for RillPool {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.detach_distributor(ctx);
        ctx.shutdown();
        Ok(())
    }
}
