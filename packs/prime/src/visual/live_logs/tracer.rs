use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Default)]
pub struct LiveLogsOpts {
    // TODO: Add levels here (maybe)
}

impl From<LiveLogsOpts> for LiveLogsSpec {
    fn from(_opts: LiveLogsOpts) -> Self {
        Self {}
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LiveLogs {
    tracer: BindedTracer<LiveLogsState>,
}

impl LiveLogs {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        mode: FlowMode,
        spec: impl Into<LiveLogsSpec>,
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
        let msg = LiveLogsEvent::Add(record);
        self.tracer.send(msg, None);
    }
}
