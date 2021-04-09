use crate::calc::Ema;
use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

const PERIOD: u32 = 10;

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

impl Flow for ConnectionState {
    type Event = ConnectionEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.connection.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            ConnectionEvent::AddRoundTrip { ms } => {
                let value = ms as f64;
                if let Some(round_trip) = self.round_trip.as_mut() {
                    round_trip.update(value);
                } else {
                    let ema = Ema::new(value, PERIOD);
                    self.round_trip = Some(ema);
                }
            }
        }
    }
}

pub type PathDelta = Vec<TimedEvent<ConnectionEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionEvent {
    AddRoundTrip { ms: u32 },
}
