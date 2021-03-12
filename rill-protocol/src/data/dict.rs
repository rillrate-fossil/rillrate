use super::{ConvertError, Delta, Metric, TimedEvent};
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct DictMetric;

impl Metric for DictMetric {
    type State = DictState;
    type Event = DictEvent;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            DictEvent::SetValue { key, value } => {
                state.map.insert(key, value);
            }
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DictEvent {
    SetValue { key: String, value: String },
}
