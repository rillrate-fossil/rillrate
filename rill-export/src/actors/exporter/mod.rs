mod actor;
pub use actor::Exporter;

mod link;
pub use link::{ExporterLinkForClient, ExporterLinkForProvider};

mod graphite;
use graphite::GraphiteExporter;

mod prometheus;
use prometheus::PrometheusExporter;

use meio::prelude::Action;
use rill_protocol::provider::{Path, RillData, StreamType};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum ExportEvent {
    SetInfo {
        path: Path,
        info: String,
    },
    BroadcastData {
        path: Path,
        timestamp: Duration,
        data: RillData,
    },
}

impl Action for ExportEvent {}

#[derive(Debug, Clone)]
pub struct PathNotification {
    pub path: Path,
    pub stream_type: StreamType,
}

impl Action for PathNotification {}
