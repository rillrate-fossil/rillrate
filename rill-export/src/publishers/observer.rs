use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::LiteTask;
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::data::{counter, dict, gauge, histogram, logger, pulse, table, Metric};
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

/// `Observer` subscribes to a path and receives all the new values from the stream
/// and writes values to a shared state (`SharedRecord`).
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
                    let new_state = T::unpack_state(new_state)?;
                    state = Some(new_state);
                }
                StateOrDelta::Delta(delta) => {
                    if let Some(state) = state.as_mut() {
                        let events = T::unpack_delta(delta)?;
                        for event in events {
                            T::apply(state, event);
                        }
                    }
                }
            }
            let pair = state
                .as_ref()
                .map(T::to_value)
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
            StreamType::CounterStream => self.state_routine::<counter::CounterMetric>().await,
            StreamType::GaugeStream => self.state_routine::<gauge::GaugeMetric>().await,
            StreamType::PulseStream => self.state_routine::<pulse::PulseMetric>().await,
            StreamType::LogStream => self.state_routine::<logger::LogMetric>().await,
            StreamType::DictStream => self.state_routine::<dict::DictMetric>().await,
            StreamType::TableStream => self.state_routine::<table::TableMetric>().await,
            StreamType::HistogramStream => self.state_routine::<histogram::HistogramMetric>().await,
        }
    }
}

/// Converts a state and deltas into a flow of numeric values
/// suitable for the metrics tracing systems.
trait Extractor: Metric {
    fn to_value(state: &Self::State) -> Option<(Timestamp, f64)>;
}

impl Extractor for counter::CounterMetric {
    fn to_value(state: &Self::State) -> Option<(Timestamp, f64)> {
        state.timestamp.map(|ts| (ts, state.value))
    }
}

impl Extractor for gauge::GaugeMetric {
    fn to_value(state: &Self::State) -> Option<(Timestamp, f64)> {
        state.timestamp.map(|ts| (ts, state.value))
    }
}

impl Extractor for pulse::PulseMetric {
    fn to_value(state: &Self::State) -> Option<(Timestamp, f64)> {
        state
            .frame
            .iter()
            .last()
            .map(|event| (event.timestamp, event.event.value))
    }
}

impl Extractor for logger::LogMetric {
    fn to_value(_state: &Self::State) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for table::TableMetric {
    fn to_value(_state: &Self::State) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for dict::DictMetric {
    fn to_value(_state: &Self::State) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for histogram::HistogramMetric {
    fn to_value(_state: &Self::State) -> Option<(Timestamp, f64)> {
        // TODO: Histogram result have to be here (not a plain timestamp)
        todo!()
    }
}
