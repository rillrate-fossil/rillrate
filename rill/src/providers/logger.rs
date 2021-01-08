use super::provider::Provider;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Path, RillData, StreamType};
use std::time::SystemTime;

/// This provider sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct LogProvider {
    provider: Provider,
}

impl LogProvider {
    /// Create a new instance of the `Provider`.
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::LogStream);
        Self { provider }
    }

    /// Writes a message.
    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = RillData::LogRecord { message };
        self.provider.send(data, timestamp);
    }
}
