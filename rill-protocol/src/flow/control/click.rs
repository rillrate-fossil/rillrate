use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClickFlow {
    pub caption: String,
}

impl Flow for ClickFlow {
    type State = ClickState;
    type Event = ClickEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.click.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        state.last_click = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickState {
    pub last_click: Option<Timestamp>,
}

#[allow(clippy::new_without_default)]
impl ClickState {
    pub fn new() -> Self {
        Self { last_click: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;
