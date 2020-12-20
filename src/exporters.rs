use crate::protocol::{Path, RillData};
use std::time::Duration;

#[derive(Debug)]
pub struct BroadcastData {
    pub path: Path,
    pub timestamp: Duration,
    pub data: RillData,
}
