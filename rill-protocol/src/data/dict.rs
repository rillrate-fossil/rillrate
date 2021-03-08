use super::{Delta, Event, State, TimedEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictState {
    map: HashMap<String, String>,
}

impl Default for DictState {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
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
