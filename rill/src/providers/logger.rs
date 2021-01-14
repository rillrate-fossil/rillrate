use super::provider::Provider;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

/// This provider sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct LogProvider {
    provider: Provider,
}

impl LogProvider {
    /// Create a new instance of the `Provider`.
    pub fn new(path: Path) -> Self {
        let info = format!("{} logger", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::LogStream,
        };
        let provider = Provider::new(description, false);
        Self { provider }
    }

    /// Writes a message.
    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = RillData::LogRecord { message };
        self.provider.send(data, timestamp);
    }
}
