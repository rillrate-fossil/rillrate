use crate::manifest::description::{Layer, PackFlow};
use rill_protocol::flow::core::Flow;
use rill_protocol::frame::Frame;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub module: String,
    pub level: String,
    pub timestamp: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveLogsSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveLogsState {
    pub spec: LiveLogsSpec,
    pub frame: Frame<LogRecord>,
}

impl From<LiveLogsSpec> for LiveLogsState {
    fn from(spec: LiveLogsSpec) -> Self {
        let frame = Frame::new(50); // TODO: Get from spec
        Self { spec, frame }
    }
}

impl PackFlow for LiveLogsState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for LiveLogsState {
    type Action = ();
    type Event = LiveLogsEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            LiveLogsEvent::Add(record) => {
                self.frame.insert_pop(record);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveLogsEvent {
    Add(LogRecord),
}
