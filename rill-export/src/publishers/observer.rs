use anyhow::{anyhow, Error};
use async_trait::async_trait;
use futures::StreamExt;
use meio::LiteTask;
use once_cell::sync::Lazy;
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::flow::data::{pulse::PulseFlow, Flow};
use rill_protocol::io::provider::{Description, StreamType, Timestamp};
use std::collections::HashMap;
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

// TODO: Remove all this things...
static ROUTINES: Lazy<RoutineMap> = Lazy::new(|| {
    let mut map = RoutineMap::new();
    map.insert(PulseFlow { range: None });
    map
});

struct RoutineMap {
    map: HashMap<StreamType, Box<dyn AbstractObserver>>,
}

impl RoutineMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn insert<T: Extractor>(&mut self, extractor: T) {
        let stream_type = T::stream_type();
        let routine = Box::new(extractor);
        self.map.insert(stream_type, routine);
    }

    fn get(&self, stream_type: &StreamType) -> Option<&dyn AbstractObserver> {
        self.map.get(stream_type).map(Box::as_ref)
    }
}

#[async_trait]
trait AbstractObserver: Sync + Send {
    async fn execute(&self, observer: Observer) -> Result<(), Error>;
}

#[async_trait]
impl<T> AbstractObserver for T
where
    T: Extractor,
{
    async fn execute(&self, observer: Observer) -> Result<(), Error> {
        observer.state_routine::<T>().await
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
        let metric: T = self.description.try_extract_metric()?;
        let path = self.description.path.clone();
        let mut subscription = self.client.subscribe_to_path(path).recv().await?;
        let mut state = None;
        while let Some(msg) = subscription.next().await {
            match msg {
                StateOrDelta::State(new_state) => {
                    let new_state = T::unpack_state(&new_state)?;
                    state = Some(new_state);
                }
                StateOrDelta::Delta(delta) => {
                    if let Some(state) = state.as_mut() {
                        let events = T::unpack_delta(&delta)?;
                        for event in events {
                            metric.apply(state, event);
                        }
                    }
                }
            }
            let pair = state
                .as_ref()
                .map(T::get_value)
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
        let stream_type = &self.description.stream_type;
        if let Some(routine) = ROUTINES.get(&self.description.stream_type) {
            routine.execute(self).await
        } else {
            Err(anyhow!(
                "Streams with type {} are not supported.",
                stream_type
            ))
        }
    }
}

/// Converts a state and deltas into a flow of numeric values
/// suitable for the metrics tracing systems.
trait Extractor: Flow {
    fn get_value(state: &Self::State) -> Option<(Timestamp, f64)>;
}

impl Extractor for PulseFlow {
    fn get_value(state: &Self::State) -> Option<(Timestamp, f64)> {
        state
            .frame
            .iter()
            .last()
            .map(|event| (event.timestamp, event.event.value))
    }
}
