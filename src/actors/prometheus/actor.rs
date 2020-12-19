use crate::actors::supervisor::RillSupervisor;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy, StopReceiver, Task,
};
use warp::Filter;

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
        ctx.spawn_task(Endpoint::new(), ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Task<Endpoint>> for PrometheusExporter {
    async fn handle(
        &mut self,
        _id: IdOf<Task<Endpoint>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

struct Endpoint {}

impl Endpoint {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl LiteTask for Endpoint {
    async fn routine(mut self, stop: StopReceiver) -> Result<(), Error> {
        let metrics = warp::path("metrics").map(|| "#metrics");
        let index = warp::any().map(|| "Rill Prometheus Client");
        let routes = metrics.or(index);
        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([0, 0, 0, 0], 9090), stop.into_future());
        server.await;
        Ok(())
    }
}
