// TODO: Remove this clippy fix
#![allow(clippy::new_without_default)]

use crate::actors::provider::RillProvider;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InterruptedBy, StartedBy};

pub struct RillStorage {}

impl RillStorage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for RillStorage {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillProvider> for RillStorage {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Opens a log file
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillProvider> for RillStorage {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
