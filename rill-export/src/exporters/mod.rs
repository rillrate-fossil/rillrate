/*
mod graphite;
pub use graphite::GraphiteExporter;

mod prometheus;
pub use prometheus::PrometheusExporter;
*/

use rill::protocol::{Path, RillData};
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
