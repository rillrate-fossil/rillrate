use rill_protocol::provider::EntryId;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RillConfig {
    inner: Arc<RillConfigInner>,
}

impl RillConfig {
    pub fn new(host: String, entry_id: EntryId, with_meta: bool) -> Self {
        let url = format!("ws://{}/live/provider", host);
        let inner = Arc::new(RillConfigInner {
            entry_id,
            url,
            with_meta,
        });
        Self { inner }
    }

    pub fn entry_id(&self) -> &EntryId {
        &self.inner.entry_id
    }

    pub fn url(&self) -> &str {
        &self.inner.url
    }

    /*
    pub fn with_meta(&self) -> bool {
        self.inner.with_meta
    }
    */
}

#[derive(Debug)]
struct RillConfigInner {
    entry_id: EntryId,
    url: String,
    with_meta: bool,
}
