use crate::actors::supervisor::RillSupervisor;
use crate::exporters::BroadcastData;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::{
    Actor, Consumer, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy, StopReceiver,
    Task,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use warp::Filter;

pub struct PrometheusExporter {
    rx: Option<broadcast::Receiver<Arc<BroadcastData>>>,
}

impl PrometheusExporter {
    pub fn new(receiver: broadcast::Receiver<Arc<BroadcastData>>) -> Self {
        Self { rx: Some(receiver) }
    }
}

impl Actor for PrometheusExporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillSupervisor> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let rx = self
            .rx
            .take()
            .ok_or(Error::msg(
                "attempt to start the same prometheus exporter twice",
            ))?
            .into_stream()
            .boxed();
        ctx.address().attach(rx);
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

#[async_trait]
impl Consumer<Result<Arc<BroadcastData>, broadcast::RecvError>> for PrometheusExporter {
    async fn handle(
        &mut self,
        msg: Result<Arc<BroadcastData>, broadcast::RecvError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        todo!()
    }
}

struct Endpoint {}

impl Endpoint {
    fn new() -> Self {
        Self {}
    }

    async fn metrics() -> Result<impl warp::Reply, Infallible> {
        Ok("# METRICS")
    }
}

#[async_trait]
impl LiteTask for Endpoint {
    async fn routine(mut self, stop: StopReceiver) -> Result<(), Error> {
        let metrics = warp::path("metrics").and_then(Self::metrics);
        let index = warp::any().map(|| "Rill Prometheus Client");
        let routes = metrics.or(index);
        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([0, 0, 0, 0], 9090), stop.into_future());
        log::info!("Prometheus endpoint binded to: {}", addr);
        server.await;
        Ok(())
    }
}
