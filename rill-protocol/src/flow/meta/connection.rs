use super::MetaFlow;
use crate::flow::data::{Flow, TimedEvent};
use crate::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};

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
                state.round_trip = Some(ms);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    round_trip: Option<u32>,
}

#[allow(clippy::new_without_default)]
impl ConnectionState {
    pub fn new() -> Self {
        Self { round_trip: None }
    }
}

pub type PathDelta = Vec<TimedEvent<ConnectionEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionEvent {
    AddRoundTrip { ms: u32 },
}
