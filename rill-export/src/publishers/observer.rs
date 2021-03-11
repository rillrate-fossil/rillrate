use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::LiteTask;
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::data::{counter, dict, gauge, logger, table, State};
use rill_protocol::io::provider::{Description, StreamType, Timestamp};
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
    description: Description,
    client: ClientLink,
    record: SharedRecord,
}

impl Observer {
    pub fn new(description: Description, client: ClientLink, record: SharedRecord) -> Self {
        Self {
            description,
            client,
            record,
        }
    }

    async fn state_routine<T>(mut self) -> Result<(), Error>
    where
        T: Extractor,
    {
        let path = self.description.path.clone();
        let mut subscription = self.client.subscribe_to_path(path).recv().await?;
        let mut state = None;
        while let Some(msg) = subscription.next().await {
            match msg {
                StateOrDelta::State(new_state) => {
                    let new_state = T::try_from(new_state)?;
                    state = Some(new_state);
                }
                StateOrDelta::Delta(delta) => {
                    if let Some(state) = state.as_mut() {
                        let events = T::try_extract(delta)?;
                        for event in events {
                            state.apply(event);
                        }
                    }
                }
            }
            let pair = state
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

#[async_trait]
impl LiteTask for Observer {
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        match self.description.stream_type {
            StreamType::CounterStream => self.state_routine::<counter::CounterState>().await,
            StreamType::GaugeStream => self.state_routine::<gauge::GaugeState>().await,
            StreamType::LogStream => self.state_routine::<logger::LogState>().await,
            StreamType::DictStream => self.state_routine::<dict::DictState>().await,
            StreamType::TableStream => self.state_routine::<table::TableState>().await,
        }
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
            .map(|event| (event.timestamp, event.event.value))
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
