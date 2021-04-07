use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectorFlow {
    pub label: String,
    pub options: BTreeSet<String>,
}

impl Flow for SelectorFlow {
    type State = SelectorState;
    type Event = SelectorEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.selector.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        let new_value = event.event.select;
        if self.options.contains(&new_value) {
            state.selected = new_value;
        } else {
            log::error!("No option {} in the selector: {}.", new_value, self.label);
        }
        state.updated = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorState {
    pub selected: String,
    pub updated: Option<Timestamp>,
}

#[allow(clippy::new_without_default)]
impl SelectorState {
    pub fn new(selected: String) -> Self {
        Self {
            selected,
            updated: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorEvent {
    pub select: String,
}

impl SelectorEvent {
    pub fn select(value: String) -> Self {
        Self { select: value }
    }
}
