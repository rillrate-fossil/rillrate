mod graphite;
pub use graphite::GraphiteExporter;

mod prometheus;
pub use prometheus::PrometheusExporter;

use crate::protocol::{Path, RillData};
use std::time::Duration;

#[derive(Debug)]
pub struct BroadcastData {
    pub path: Path,
    pub timestamp: Duration,
    pub data: RillData,
}
