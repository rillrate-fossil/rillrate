use super::provider::Provider;
use derive_more::{Deref, DerefMut};
use rill_protocol::{Path, RillData, StreamType};
use std::time::SystemTime;

#[derive(Debug, Deref, DerefMut)]
pub struct LogProvider {
    provider: Provider,
}

impl LogProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::LogStream);
        Self { provider }
    }

    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = RillData::LogRecord { message };
        self.provider.send(data, timestamp);
    }
}
