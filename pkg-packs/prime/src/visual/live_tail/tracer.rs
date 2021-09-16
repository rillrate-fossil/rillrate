use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Default)]
pub struct LiveTailOpts {
    // TODO: Add levels here (maybe)
}

impl From<LiveTailOpts> for LiveTailSpec {
    fn from(_opts: LiveTailOpts) -> Self {
        Self {}
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LiveTail {
    tracer: BindedTracer<LiveTailState>,
}

impl LiveTail {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        mode: FlowMode,
        spec: impl Into<LiveTailSpec>,
    ) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    pub fn log(
        &self,
        module: impl Into<String>,
        level: impl Into<String>,
        timestamp: impl Into<String>,
        content: impl Into<String>,
    ) {
        let record = LogRecord {
            module: module.into(),
            level: level.into(),
            timestamp: timestamp.into(),
            content: content.into(),
        };
        let msg = LiveTailEvent::Add(record);
        self.tracer.send(msg, None);
    }

    pub fn log_now(
        &self,
        module: impl Into<String>,
        level: impl Into<String>,
        content: impl Into<String>,
    ) {
        let timestamp = chrono::Local::now().format("%F%T%.3f").to_string();
        let record = LogRecord {
            module: module.into(),
            level: level.into(),
            timestamp: timestamp.into(),
            content: content.into(),
        };
        let msg = LiveTailEvent::Add(record);
        self.tracer.send(msg, None);
    }
}
