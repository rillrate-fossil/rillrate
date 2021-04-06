use crate::flow::data::{Flow, TimedEvent};
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClickFlow;

impl Flow for ClickFlow {
    type State = ClickState;
    type Event = ClickEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.click.v0")
    }

    fn apply(&self, _state: &mut Self::State, _event: TimedEvent<Self::Event>) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickState;

#[allow(clippy::new_without_default)]
impl ClickState {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;
