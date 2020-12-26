use crate::actors::exporter::Exporter;
use crate::exporters::ExportEvent;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Address, Context, IdOf, Interaction, InteractionHandler, InterruptedBy, LiteTask,
    StartedBy, StopReceiver, TaskEliminated, TryConsumer,
};
use meio_connect::hyper::{Body, Request, Response};
use meio_connect::server_2::{DirectPath, FromRequest, HttpServerLink, Req};
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
    server: HttpServerLink,
    metrics: BTreeMap<Path, Record>,
}

impl PrometheusExporter {
    pub fn new(server: HttpServerLink) -> Self {
        Self {
            server,
            metrics: BTreeMap::new(),
        }
    }
}

impl Actor for PrometheusExporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Exporter> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.server
            .add_route::<RenderMetrics, _>(ctx.address().clone())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Exporter> for PrometheusExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
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

#[derive(Default)]
struct RenderMetrics;

impl DirectPath for RenderMetrics {
    fn paths() -> &'static [&'static str] {
        &["/metrics"]
    }
}

#[async_trait]
impl InteractionHandler<Req<RenderMetrics>> for PrometheusExporter {
    async fn handle(
        &mut self,
        _: Req<RenderMetrics>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
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
        Ok(Response::new(buffer.into()))
    }
}
