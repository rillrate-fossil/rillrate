use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct AlertOpts {}

impl From<AlertOpts> for AlertSpec {
    fn from(_opts: AlertOpts) -> Self {
        Self {}
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Alert {
    tracer: BindedTracer<AlertState>,
}

impl Alert {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<AlertSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn notify(&self, reason: impl Into<String>) {
        let msg = AlertEvent::Notify {
            text: reason.into(),
        };
        self.tracer.send(msg, None);
    }
}
