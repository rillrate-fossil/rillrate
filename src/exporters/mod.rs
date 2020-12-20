mod graphite;
pub use graphite::GraphiteExporter;

mod prometheus;
pub use prometheus::PrometheusExporter;

use crate::protocol::{Path, RillData};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum ExportEvent {
    SetInfo { info: String },
    BroadcastData(Arc<BroadcastData>),
}

impl From<BroadcastData> for ExportEvent {
    fn from(data: BroadcastData) -> Self {
        Self::BroadcastData(Arc::new(data))
    }
}

#[derive(Debug)]
pub struct BroadcastData {
    pub path: Path,
    pub timestamp: Duration,
    pub data: RillData,
}
