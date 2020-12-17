use super::provider::Provider;
use crate::protocol::{Path, RillData, StreamType};
use derive_more::{Deref, DerefMut};

#[derive(Debug, Deref, DerefMut)]
pub struct LogProvider {
    provider: Provider,
}

impl LogProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::LogStream);
        Self { provider }
    }

    pub fn log(&self, timestamp: String, message: String) {
        let data = RillData::LogRecord { timestamp, message };
        self.provider.send(data);
    }
}
