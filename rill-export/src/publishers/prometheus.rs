use super::{Observer, Publisher, SharedRecord};
use crate::actors::export::RillExport;
use crate::config::PrometheusConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Context, IdOf, InteractionHandler, InterruptedBy, StartedBy,
    TaskEliminated, TaskError,
};
use meio_connect::hyper::{Body, Response};
use meio_connect::server::{DirectPath, HttpServerLink, Req, WebRoute};
use rill_client::actors::broadcaster::{BroadcasterLinkForClient, PathNotification};
use rill_client::actors::client::ClientLink;
use rill_protocol::flow::data::{pulse::PulseMetric, Metric};
use rill_protocol::io::provider::{Description, Path, PathPattern, StreamType};
use serde::Deserialize;
use std::collections::{
    btree_map::{BTreeMap, Entry},
    HashMap,
};
use strum::Display;

struct Record {
    event: SharedRecord,
    description: Description,
}

#[derive(Debug, Display)]
#[strum(serialize_all = "UPPERCASE")]
enum PrometheusType {
    Gauge,
}

pub struct PrometheusPublisher {
    config: PrometheusConfig,
    broadcaster: BroadcasterLinkForClient,
    client: ClientLink,
    server: HttpServerLink,
    metrics: BTreeMap<Path, Record>,
    supported_types: HashMap<StreamType, PrometheusType>,
}

impl Publisher for PrometheusPublisher {
    type Config = PrometheusConfig;

    fn create(
        config: Self::Config,
        broadcaster: BroadcasterLinkForClient,
        client: ClientLink,
        server: &HttpServerLink,
    ) -> Self {
        let mut supported_types = HashMap::new();
        supported_types.insert(PulseMetric::stream_type(), PrometheusType::Gauge);
        Self {
            config,
            broadcaster,
            client,
            server: server.clone(),
            metrics: BTreeMap::new(),
            supported_types,
        }
    }
}

impl PrometheusPublisher {
    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        //self.broadcaster.unsubscribe_all(ctx.address()).await.ok();
        ctx.shutdown();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Streams,
}

impl Actor for PrometheusPublisher {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<RillExport> for PrometheusPublisher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Streams]);
        let metrics_route = WebRoute::<RenderMetrics, _>::new(ctx.address().clone());
        self.server.add_route(metrics_route).await?;
        self.broadcaster
            .subscribe_to_struct_changes(ctx.address().clone())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillExport> for PrometheusPublisher {
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
                    let pattern = PathPattern { path: path.clone() };
                    if self.config.paths.contains(&pattern) {
                        if let Entry::Vacant(entry) = self.metrics.entry(path.clone()) {
                            let event = SharedRecord::new();
                            let record = Record {
                                event: event.clone(),
                                description: description.clone(),
                            };
                            entry.insert(record);
                            let observer = Observer::new(description, self.client.clone(), event);
                            ctx.spawn_task(observer, (), Group::Streams);
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
impl TaskEliminated<Observer, ()> for PrometheusPublisher {
    async fn handle(
        &mut self,
        _id: IdOf<Observer>,
        _tag: (),
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default, Deserialize)]
struct RenderMetrics {}

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
            if let Some(event) = record.event.get().await {
                let name = path.as_ref().join("_");
                let stream_type = &record.description.stream_type;
                if let Some(typ) = self.supported_types.get(stream_type) {
                    let value = event.value;
                    let ts = event.timestamp;
                    let line = format!("# HELP {}\n", info);
                    buffer.push_str(&line);
                    let line = format!("# TYPE {} {}\n", name, typ);
                    buffer.push_str(&line);
                    let line = format!("{} {} {}\n\n", name, value, ts.as_millis());
                    buffer.push_str(&line);
                } else {
                    log::error!(
                        "Prometheus publisher is not supported type of stream: {}",
                        stream_type
                    );
                    continue;
                }
            }
        }
        Ok(Response::new(buffer.into()))
    }
}
