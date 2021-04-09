use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SelectorFlow;

impl Flow for SelectorFlow {
    type State = SelectorState;
    type Event = SelectorEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.selector.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        let new_value = event.event.select;
        if state.options.contains(&new_value) {
            state.selected = new_value;
        } else {
            log::error!("No option {} in the selector: {}.", new_value, state.label);
        }
        state.updated = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorState {
    // IMMUTABLE
    pub label: String,
    /// It's `Vec` to keep the order.
    pub options: Vec<String>,

    // MUTABLE
    pub selected: String,
    pub updated: Option<Timestamp>,
}

#[allow(clippy::new_without_default)]
impl SelectorState {
    pub fn new(label: String, options: Vec<String>, selected: String) -> Self {
        Self {
            label,
            options,
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
