mod actor;
pub use actor::Exporter;

mod link;
pub use link::{ExporterLinkForClient, ExporterLinkForProvider};

pub mod publishers;

use meio::prelude::{Action, Actor, Address, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
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

/// An `Actor` that exports metrics to a third-party system.
pub trait Publisher: Actor + StartedBy<Exporter> + InterruptedBy<Exporter> {
    type Config: Send;
    fn create(
        config: Self::Config,
        exporter: ExporterLinkForClient,
        server: &HttpServerLink,
    ) -> Self;
}
