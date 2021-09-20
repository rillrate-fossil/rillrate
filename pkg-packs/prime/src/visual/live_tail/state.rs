use rill_protocol::flow::core::Flow;
use rill_protocol::frame::Frame;
use rill_protocol::io::provider::StreamType;
use rrpack_basis::manifest::description::{Layer, PackFlow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub module: String,
    pub level: String,
    pub timestamp: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTailSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTailState {
    pub spec: LiveTailSpec,
    pub frame: Frame<LogRecord>,
}

impl From<LiveTailSpec> for LiveTailState {
    fn from(spec: LiveTailSpec) -> Self {
        let frame = Frame::new(50); // TODO: Get from spec
        Self { spec, frame }
    }
}

impl PackFlow for LiveTailState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for LiveTailState {
    type Action = ();
    type Event = LiveTailEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            LiveTailEvent::Add(record) => {
                self.frame.insert_pop(record);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveTailEvent {
    Add(LogRecord),
}
