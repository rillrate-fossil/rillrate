use super::{ConvertError, Delta, Event, State, TimedEvent};
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictState {
    pub map: HashMap<String, String>,
}

impl Default for DictState {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
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

    fn apply(&mut self, update: Self::Delta) {
        self.map.extend(update.map);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictDelta {
    map: HashMap<String, String>,
}

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
        let mut this = Self {
            map: HashMap::new(),
        };
        this.combine(event);
        this
    }

    fn combine(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            DictEvent::SetValue { key, value } => {
                self.map.insert(key, value);
            }
        }
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
