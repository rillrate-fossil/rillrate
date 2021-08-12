use rill_protocol::flow::core::{Flow, TimedEvent};
use rill_protocol::io::provider::{StreamType, Timestamp};
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
    type Action = ClickAction;
    type Event = TimedEvent<ClickEvent>;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.click.v0")
    }

    fn apply(&mut self, event: Self::Event) {
        self.last_click = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;
