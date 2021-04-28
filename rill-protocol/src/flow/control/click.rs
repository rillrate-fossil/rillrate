use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickState {
    pub caption: String,
    pub last_click: Option<Timestamp>,
}

impl ClickState {
    pub fn new(caption: String) -> Self {
        Self {
            caption,
            last_click: None,
        }
    }
}

impl Flow for ClickState {
    type Action = ();
    type Event = ClickEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.click.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        self.last_click = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;
