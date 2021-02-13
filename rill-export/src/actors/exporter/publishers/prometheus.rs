use crate::actors::exporter::{
    ExportEvent, Exporter, ExporterLinkForClient, PathNotification, Publisher,
};
use crate::config::PrometheusConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{ActionHandler, Actor, Context, InteractionHandler, InterruptedBy, StartedBy};
use meio_connect::hyper::{Body, Response};
use meio_connect::server::{DirectPath, HttpServerLink, Req};
use rill_protocol::provider::{Description, Path, RillEvent, StreamType};
use std::collections::btree_map::{BTreeMap, Entry};
use std::convert::TryInto;

struct Record {
    event: Option<RillEvent>,
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
            .add_route(RenderMetrics, ctx.address().clone())
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
        match msg {
            PathNotification::Paths { descriptions } => {
                for description in descriptions {
                    let path = description.path.clone();
                    // TODO: Improve that... Maybe use `PatternMatcher` that wraps `HashSet` of `Patterns`
                    let pattern = crate::config::PathPattern { path: path.clone() };
                    if self.config.paths.contains(&pattern) {
                        self.exporter
                            .subscribe_to_data(path.clone(), ctx.address().clone())
                            .await?;
                        if let Entry::Vacant(entry) = self.metrics.entry(path) {
                            let record = Record {
                                event: None,
                                description,
                            };
                            entry.insert(record);
                        }
                    }
                }
            }
            PathNotification::Name { .. } => {}
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<ExportEvent> for PrometheusPublisher {
    async fn handle(&mut self, msg: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match msg {
            ExportEvent::BroadcastData { path, event } => {
                if let Some(record) = self.metrics.get_mut(&path) {
                    record.event = Some(event);
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct RenderMetrics;

impl DirectPath for RenderMetrics {
    type Parameter = ();
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
            let info = &record.description.info;
            if let Some(event) = record.event.clone() {
                let name = path.as_ref().join("_");
                let typ = match record.description.stream_type {
                    StreamType::CounterStream => "counter",
                    StreamType::GaugeStream => "gauge",
                    _ => {
                        log::error!(
                            "Prometheus publisher is not supported type of stream: {}",
                            record.description.stream_type
                        );
                        continue;
                    }
                };
                let value: f64 = match event.data.clone().try_into() {
                    Ok(n) => n, // TODO: Round?
                    Err(err) => {
                        log::error!("Can't convert data {:?} into a number: {}", event.data, err);
                        continue;
                    }
                };
                let ts = event.timestamp;
                let line = format!("# HELP {}\n", info);
                buffer.push_str(&line);
                let line = format!("# TYPE {} {}\n", name, typ);
                buffer.push_str(&line);
                let line = format!("{} {} {}\n\n", name, value, ts.as_millis());
                buffer.push_str(&line);
            }
        }
        Ok(Response::new(buffer.into()))
    }
}
