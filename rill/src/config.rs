use rill_protocol::provider::EntryId;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RillConfig {
    inner: Arc<RillConfigInner>,
}

impl RillConfig {
    pub fn new(host: String, entry_id: EntryId) -> Self {
        let url = format!("ws://{}/live/provider", host);
        let inner = Arc::new(RillConfigInner { entry_id, url });
        Self { inner }
    }

    pub fn entry_id(&self) -> &EntryId {
        &self.inner.entry_id
    }

    pub fn url(&self) -> &str {
        &self.inner.url
    }
}

#[derive(Debug)]
struct RillConfigInner {
    entry_id: EntryId,
    url: String,
}
