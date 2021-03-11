use super::{ConvertError, Delta, Event, State, TimedEvent};
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictState {
    pub map: BTreeMap<String, String>,
}

impl Default for DictState {
    fn default() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

impl TryFrom<StreamState> for DictState {
    type Error = ConvertError;

    fn try_from(state: StreamState) -> Result<Self, ConvertError> {
        match state {
            StreamState::Dict(state) => Ok(state),
            _ => Err(ConvertError),
        }
    }
}

impl State for DictState {
    type Delta = DictDelta;
    type Event = DictEvent;

    fn apply(&mut self, delta: Self::Delta) {
        for event in delta {
            match event.event {
                DictEvent::SetValue { key, value } => {
                    self.map.insert(key, value);
                }
            }
        }
    }
}

pub type DictDelta = Vec<TimedEvent<DictEvent>>;

impl TryFrom<StreamDelta> for DictDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Dict(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

impl Delta for DictDelta {
    type Event = DictEvent;

    fn produce(event: TimedEvent<Self::Event>) -> Self {
        vec![event]
    }

    fn combine(&mut self, event: TimedEvent<Self::Event>) {
        self.push(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DictEvent {
    SetValue { key: String, value: String },
}

impl Event for DictEvent {
    type State = DictState;
    type Delta = DictDelta;
}
