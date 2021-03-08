use crate::publishers::converter::Extractor;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::LiteTask;
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::data::State;
use rill_protocol::io::provider::{Path, Timestamp};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Record {
    pub timestamp: Timestamp,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct SharedRecord {
    protected_record: Arc<Mutex<Option<Record>>>,
}

impl SharedRecord {
    pub fn new() -> Self {
        Self {
            protected_record: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn update(&self, record: Record) {
        let mut cell = self.protected_record.lock().await;
        *cell = Some(record);
    }

    pub async fn get(&self) -> Option<Record> {
        self.protected_record.lock().await.clone()
    }
}

pub struct Observer {
    map: HashMap<Path, SharedRecord>,
}

pub struct ObserveTask<T> {
    path: Path,
    client: ClientLink,
    state: Option<T>,
    record: SharedRecord,
}

#[async_trait]
impl<T> LiteTask for ObserveTask<T>
where
    T: State + Extractor, // TODO: Require `State` in the `Extractor`
{
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let path = self.path.clone();
        let mut subscription = self.client.subscribe_to_path(path).recv().await?;
        while let Some(msg) = subscription.next().await {
            match msg {
                StateOrDelta::State(state) => {
                    let state = T::try_from(state)?;
                    self.state = Some(state);
                }
                StateOrDelta::Delta(delta) => {
                    let delta = T::Delta::try_from(delta)?;
                    if let Some(state) = self.state.as_mut() {
                        state.apply(delta);
                    }
                }
            }
            let pair = self
                .state
                .as_ref()
                .map(Extractor::to_value)
                .and_then(std::convert::identity);
            if let Some((timestamp, value)) = pair {
                let record = Record { timestamp, value };
                self.record.update(record).await;
            }
        }
        Ok(())
    }
}
