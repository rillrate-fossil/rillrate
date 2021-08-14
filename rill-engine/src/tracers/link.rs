//! Link allows to connect to a `Recorder` to listen to incoming actions.

use crate::actors::pool::{RillPoolTask, DISTRIBUTOR};
use crate::tracers::tracer::{self, ControlReceiver, ControlSender};
use anyhow::Error;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use rill_protocol::flow::core;
use rill_protocol::flow::core::ActionEnvelope;
use std::sync::Arc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A link for listening events from a `Recorder`.
pub struct Link<T: core::Flow> {
    tx: ControlSender<T>,
    rx: ControlReceiver<T>,
}

impl<T: core::Flow> Link<T> {
    /// Creates a new link.
    pub fn new() -> Self {
        let (tx, rx) = tracer::channel();
        Self { tx, rx }
    }

    /// Clones a sender.
    pub fn sender(&self) -> ControlSender<T> {
        self.tx.clone()
    }

    /// Takes a receiver.
    pub fn receiver(self) -> ControlReceiver<T> {
        self.rx
    }

    /// Converts receiver into a stream of actions
    pub fn actions(self) -> impl Stream<Item = T::Action> {
        let stream = UnboundedReceiverStream::new(self.rx);
        stream.filter_map(|envelope| async move { envelope.activity.to_action() })
    }

    /// Assigns a callback to the `Link`.
    pub fn sync<F>(self, callback: F) -> Result<(), Error>
    where
        F: SyncCallback<T>,
    {
        let sync_callback = SyncCallbackTask {
            rx: self.rx,
            callback: Arc::new(callback),
        };
        DISTRIBUTOR.spawn_task(sync_callback)?;
        Ok(())
    }
}

/// Synchronous callback specification.
pub trait SyncCallback<T: core::Flow>: Sync + Send + 'static {
    /// Perform a callback.
    fn execute(&self, envelope: ActionEnvelope<T>) -> Result<(), Error>;
}

impl<F, T> SyncCallback<T> for F
where
    T: core::Flow,
    F: Fn(ActionEnvelope<T>) -> Result<(), Error>,
    F: Sync + Send + 'static,
{
    fn execute(&self, envelope: ActionEnvelope<T>) -> Result<(), Error> {
        (self)(envelope)
    }
}

struct SyncCallbackTask<T, F>
where
    T: core::Flow,
{
    rx: ControlReceiver<T>,
    callback: Arc<F>,
}

#[async_trait]
impl<T, F> RillPoolTask for SyncCallbackTask<T, F>
where
    T: core::Flow,
    F: SyncCallback<T>,
{
    async fn routine(mut self) -> Result<(), Error> {
        while let Some(envelope) = self.rx.recv().await {
            let callback = self.callback.clone();
            let res = tokio::task::spawn_blocking(move || callback.execute(envelope))
                .await
                .map_err(Error::from)
                .and_then(std::convert::identity);
            if let Err(err) = res {
                log::error!("Sync callback failed with: {}", err);
            }
        }
        Ok(())
    }
}

/*
impl<T: core::Flow> From<Link<T>> for ControlReceiver<T> {
    fn from(link: Link<T>) -> Self {
        link.rx
    }
}
*/
