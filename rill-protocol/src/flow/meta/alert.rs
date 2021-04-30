use crate::flow::core::{Flow, TimedEvent};
use crate::flow::location::Location;
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

pub const ALERTS: Location = Location::new("meta:alerts");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertState {}

#[allow(clippy::new_without_default)]
impl AlertState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Flow for AlertState {
    type Action = ();
    type Event = AlertEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.data.alert.v0")
    }

    fn apply(&mut self, _event: TimedEvent<Self::Event>) {}
}

pub type AlertDelta = Vec<TimedEvent<AlertEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub msg: String,
}
