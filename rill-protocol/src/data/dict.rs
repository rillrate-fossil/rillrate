use super::{ConvertError, Delta, State, TimedEvent};
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

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
    type Event = DictEvent;

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            DictEvent::SetValue { key, value } => {
                self.map.insert(key, value);
            }
        }
    }

    fn wrap(events: Vec<TimedEvent<Self::Event>>) -> StreamDelta {
        StreamDelta::from(events)
    }

    fn try_extract(delta: StreamDelta) -> Result<Vec<TimedEvent<Self::Event>>, ConvertError> {
        delta.try_into()
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

/*
impl Delta for DictDelta {
    type Event = DictEvent;

    fn push(&mut self, event: TimedEvent<Self::Event>) {
        self.push(event);
    }
}
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DictEvent {
    SetValue { key: String, value: String },
}
