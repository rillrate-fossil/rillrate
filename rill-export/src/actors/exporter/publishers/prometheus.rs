use crate::actors::exporter::{
    ExportEvent, Exporter, ExporterLinkForClient, PathNotification, Publisher,
};
use crate::config::PrometheusConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{ActionHandler, Actor, Context, InteractionHandler, InterruptedBy, StartedBy};
use meio_connect::hyper::{Body, Response};
use meio_connect::server::{DirectPath, HttpServerLink, Req};
use rill_protocol::provider::{Description, Path, RillData};
use std::collections::BTreeMap;

#[derive(Debug)]
struct Record {
    data: Option<RillData>,
    description: Description,
}

pub struct PrometheusPublisher {
    config: PrometheusConfig,
    exporter: ExporterLinkForClient,
    server: HttpServerLink,
    metrics: BTreeMap<Path, Record>,
}

impl Publisher for PrometheusPublisher {
    type Config = PrometheusConfig;

    fn create(
        config: Self::Config,
        exporter: ExporterLinkForClient,
        server: &HttpServerLink,
    ) -> Self {
        Self {
            config,
            exporter,
            server: server.clone(),
            metrics: BTreeMap::new(),
        }
    }
}

impl PrometheusPublisher {
    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        self.exporter.unsubscribe_all(ctx.address()).await.ok();
        ctx.shutdown();
    }
}

impl Actor for PrometheusPublisher {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<Exporter> for PrometheusPublisher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.server
            .add_route::<RenderMetrics, _>(ctx.address().clone())
            .await?;
        self.exporter
            .subscribe_to_paths(ctx.address().clone())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Exporter> for PrometheusPublisher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<PathNotification> for PrometheusPublisher {
    async fn handle(
        &mut self,
        msg: PathNotification,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        for description in msg.descriptions {
            let path = description.path.clone();
            // TODO: Improve that... Maybe use `PatternMatcher` that wraps `HashSet` of `Patterns`
            let pattern = crate::config::PathPattern { path: path.clone() };
            if self.config.paths.contains(&pattern) {
                self.exporter
                    .subscribe_to_data(path.clone(), ctx.address().clone())
                    .await?;
                if !self.metrics.contains_key(&path) {
                    let record = Record {
                        data: None,
                        description,
                    };
                    self.metrics.insert(path, record);
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<ExportEvent> for PrometheusPublisher {
    async fn handle(&mut self, msg: ExportEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match msg {
            ExportEvent::BroadcastData { path, data, .. } => {
                if let Some(record) = self.metrics.get_mut(&path) {
                    record.data = Some(data);
                }
            }
        }
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
impl InteractionHandler<Req<RenderMetrics>> for PrometheusPublisher {
    async fn handle(
        &mut self,
        _: Req<RenderMetrics>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        let mut buffer = String::new();
        for (path, record) in &self.metrics {
            if let Some(data) = record.data.as_ref() {
                let line = format!("# {}\n", path);
                buffer.push_str(&line);
                let line = format!("# {}\n", "<record.info>");
                buffer.push_str(&line);
                let line = format!("{:?}\n", data);
                buffer.push_str(&line);
            }
        }
        Ok(Response::new(buffer.into()))
    }
}
