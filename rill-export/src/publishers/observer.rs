use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::{Actor, Context, LiteTask};
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::data::{counter, dict, gauge, logger, table, State};
use rill_protocol::io::provider::{Description, Path, Timestamp, StreamType};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Record {
    pub timestamp: Timestamp,
    pub value: f64,
}

#[derive(Debug, Clone, Default)]
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

impl<T> ObserveTask<T> {
    fn new(path: Path, client: ClientLink, record: SharedRecord) -> Self {
        Self {
            path,
            client,
            state: None,
            record,
        }
    }
}

#[async_trait]
impl<T> LiteTask for ObserveTask<T>
where
    T: Extractor,
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

/// Converts a state and deltas into a flow of numeric values
/// suitable for the metrics tracing systems.
trait Extractor: State {
    fn to_value(&self) -> Option<(Timestamp, f64)>;
}

impl Extractor for counter::CounterState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        self.timestamp.map(|ts| (ts, self.value))
    }
}

impl Extractor for gauge::GaugeState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        self.frame
            .iter()
            .last()
            .map(|point| (point.timestamp, point.value))
    }
}

impl Extractor for logger::LogState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for table::TableState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for dict::DictState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        None
    }
}
