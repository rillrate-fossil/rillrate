use crate::actors::embedded_node::EmbeddedNode;
use crate::exporters::ExportEvent;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Address, Context, IdOf, Interaction, InteractionHandler, InterruptedBy, LiteTask,
    StartedBy, StopReceiver, TaskEliminated, TryConsumer,
};
use rill::protocol::{Path, RillData};
use std::collections::BTreeMap;
use std::convert::Infallible;
use tokio::sync::broadcast;
use warp::Filter;

#[derive(Debug, Default)]
struct Record {
    data: Option<RillData>,
    info: Option<String>,
}

pub struct PrometheusExporter {
    metrics: BTreeMap<Path, Record>,
}

impl PrometheusExporter {
    pub fn new() -> Self {
        Self {
            metrics: BTreeMap::new(),
        }
    }
}

impl Actor for PrometheusExporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let endpoint = Endpoint::new(ctx.address().to_owned());
        ctx.spawn_task(endpoint, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<Endpoint> for PrometheusExporter {
    async fn handle(&mut self, _id: IdOf<Endpoint>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TryConsumer<ExportEvent> for PrometheusExporter {
    type Error = broadcast::RecvError;

    async fn handle(&mut self, event: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ExportEvent::SetInfo { path, info } => {
                let record = self.metrics.entry(path).or_default();
                record.info = Some(info);
            }
            ExportEvent::BroadcastData { path, data, .. } => {
                let record = self.metrics.entry(path).or_default();
                record.data = Some(data);
            }
        }
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
        for (path, record) in &self.metrics {
            if let (Some(info), Some(data)) = (record.info.as_ref(), record.data.as_ref()) {
                let line = format!("# {}\n", path);
                buffer.push_str(&line);
                let line = format!("# {}\n", info);
                buffer.push_str(&line);
                let line = format!("{:?}\n", data);
                buffer.push_str(&line);
            }
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
