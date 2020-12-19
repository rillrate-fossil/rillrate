use crate::actors::RillSupervisor;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, InterruptedBy, StartedBy};

pub struct PrometheusExporter {}

impl PrometheusExporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for PrometheusExporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillSupervisor> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        todo!();
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        todo!();
    }
}
