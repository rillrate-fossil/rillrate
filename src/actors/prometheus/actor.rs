use crate::actors::supervisor::RillSupervisor;
use crate::exporters::BroadcastData;
use crate::protocol::{Path, RillData};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::{
    Actor, Address, Context, Eliminated, IdOf, Interaction, InteractionHandler, InterruptedBy,
    LiteTask, StartedBy, StopReceiver, Task, TryConsumer,
};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use warp::Filter;

pub struct PrometheusExporter {
    rx: Option<broadcast::Receiver<Arc<BroadcastData>>>,
    metrics: BTreeMap<Path, RillData>,
}

impl PrometheusExporter {
    pub fn new(receiver: broadcast::Receiver<Arc<BroadcastData>>) -> Self {
        Self {
            rx: Some(receiver),
            metrics: BTreeMap::new(),
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
        let endpoint = Endpoint::new(ctx.address().to_owned());
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
        log::debug!("Add metrics");
        self.metrics.insert(item.path.clone(), item.data.clone());
        Ok(())
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

struct RenderMetrics;

impl Interaction for RenderMetrics {
    type Output = String;
}

#[async_trait]
impl InteractionHandler<RenderMetrics> for PrometheusExporter {
    async fn handle(
        &mut self,
        _: RenderMetrics,
        _ctx: &mut Context<Self>,
    ) -> Result<String, Error> {
        let mut buffer = String::new();
        for (path, data) in &self.metrics {
            let line = format!("{} - {:?}\n", path, data);
            buffer.push_str(&line);
        }
        Ok(buffer)
    }
}

struct Endpoint {
    exporter: Address<PrometheusExporter>,
}

impl Endpoint {
    fn new(exporter: Address<PrometheusExporter>) -> Self {
        Self { exporter }
    }

    async fn metrics(
        mut exporter: Address<PrometheusExporter>,
    ) -> Result<impl warp::Reply, Infallible> {
        let data = exporter.interact(RenderMetrics).await.unwrap();
        Ok(data)
    }
}

#[async_trait]
impl LiteTask for Endpoint {
    async fn routine(mut self, stop: StopReceiver) -> Result<(), Error> {
        let exporter = self.exporter.clone();
        let metrics = warp::path("metrics").and_then(move || Self::metrics(exporter.clone()));
        let index = warp::any().map(|| "Rill Prometheus Client");
        let routes = metrics.or(index);
        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([0, 0, 0, 0], 9090), stop.into_future());
        log::info!("Prometheus endpoint binded to: {}", addr);
        server.await;
        Ok(())
    }
}
