//! Contains generic implementation of a watcher.

use crate::state::RILL_LINK;
use anyhow::Error;
use async_trait::async_trait;
use rill_protocol::inflow::action;
use rill_protocol::io::provider::Path;
use std::sync::Arc;

#[async_trait]
pub trait Callback<T: action::Inflow>: Send + 'static {
    async fn perform(&mut self, event: T::Event) -> Result<(), Error>;
}

pub(crate) enum WatcherMode<T: action::Inflow> {
    Push { callback: Box<dyn Callback<T>> },
}

#[derive(Debug)]
pub(crate) struct WatcherDescription<T> {
    pub path: Path,
    pub info: String,
    pub inflow: T,
}

pub struct Watcher<T: action::Inflow> {
    description: Arc<WatcherDescription<T>>,
}

impl<T: action::Inflow> Watcher<T> {
    pub fn new(inflow: T, path: Path, callback: Box<dyn Callback<T>>) -> Self {
        let stream_type = T::stream_type();
        let info = format!("{} - {}", path, stream_type);
        let description = WatcherDescription { path, info, inflow };
        let mode = WatcherMode::Push { callback };
        let description = Arc::new(description);
        let this = Watcher {
            description: description.clone(),
        };
        if let Err(err) = RILL_LINK.register_watcher(description, mode) {
            log::error!(
                "Can't register a Watcher. The worker can be terminated already: {}",
                err
            );
        }
        this
    }
}
