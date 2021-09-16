use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Clone, Default)]
pub struct ClickOpts {
    pub label: Option<String>,
}

impl From<ClickOpts> for ClickSpec {
    fn from(opts: ClickOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_else(|| "Button".into()),
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Click {
    tracer: BindedTracer<ClickState>,
}

impl Click {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<ClickSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self) {
        let msg = ClickEvent;
        self.tracer.send(msg, None);
    }
}
