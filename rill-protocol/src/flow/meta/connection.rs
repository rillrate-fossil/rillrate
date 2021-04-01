use super::MetaFlow;
use crate::flow::data::{Flow, TimedEvent};
use crate::indicators::ema::Ema;
use crate::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};

const PERIOD: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ConnectionFlow;

impl MetaFlow for ConnectionFlow {
    fn location() -> Path {
        Path::single("meta:connection")
    }
}

impl Flow for ConnectionFlow {
    type State = ConnectionState;
    type Event = ConnectionEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.connection.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            ConnectionEvent::AddRoundTrip { ms } => {
                let value = ms as f64;
                if let Some(round_trip) = state.round_trip.as_mut() {
                    round_trip.update(value);
                } else {
                    let ema = Ema::new(value, PERIOD);
                    state.round_trip = Some(ema);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    round_trip: Option<Ema>,
}

#[allow(clippy::new_without_default)]
impl ConnectionState {
    pub fn new() -> Self {
        Self { round_trip: None }
    }

    pub fn latency(&self) -> Option<f64> {
        self.round_trip.as_ref().map(Ema::value).map(|ms| ms / 2.0)
    }
}

pub type PathDelta = Vec<TimedEvent<ConnectionEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionEvent {
    AddRoundTrip { ms: u32 },
}
