use crate::actors::supervisor::RillSupervisor;
use crate::exporters::BroadcastData;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy, StopReceiver, Task,
    TryConsumer,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use warp::Filter;

pub struct PrometheusExporter {
    rx: Option<broadcast::Receiver<Arc<BroadcastData>>>,
    metrics: Arc<RwLock<String>>,
}

impl PrometheusExporter {
    pub fn new(receiver: broadcast::Receiver<Arc<BroadcastData>>) -> Self {
        Self {
            rx: Some(receiver),
            metrics: Arc::new(RwLock::new(String::new())),
        }
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
        let endpoint = Endpoint::new(self.metrics.clone());
        ctx.spawn_task(endpoint, ());
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
impl TryConsumer<Arc<BroadcastData>> for PrometheusExporter {
    type Error = broadcast::RecvError;

    async fn handle(
        &mut self,
        item: Arc<BroadcastData>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn error(&mut self, err: Self::Error, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::error!(
            "Broadcasting stream failed. Not possible to continue: {}",
            err
        );
        ctx.shutdown();
        Ok(())
    }
}

struct Endpoint {
    metrics: Arc<RwLock<String>>,
}

impl Endpoint {
    fn new(metrics: Arc<RwLock<String>>) -> Self {
        Self { metrics }
    }

    async fn metrics(metrics: Arc<RwLock<String>>) -> Result<impl warp::Reply, Infallible> {
        let data = metrics.read().await;
        Ok(data.clone())
    }
}

#[async_trait]
impl LiteTask for Endpoint {
    async fn routine(mut self, stop: StopReceiver) -> Result<(), Error> {
        let state = self.metrics.clone();
        let metrics = warp::path("metrics").and_then(move || Self::metrics(state.clone()));
        let index = warp::any().map(|| "Rill Prometheus Client");
        let routes = metrics.or(index);
        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([0, 0, 0, 0], 9090), stop.into_future());
        log::info!("Prometheus endpoint binded to: {}", addr);
        server.await;
        Ok(())
    }
}
