use super::state::*;
use crate::range::{Bound, Label, Range};
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_engine::tracers::tracer::timed;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct PulseOpts {
    pub retain: Option<u32>,

    pub suffix: Option<String>,
    pub divisor: Option<f64>,

    pub min: Option<f64>,
    pub lower: Option<bool>,
    pub max: Option<f64>,
    pub higher: Option<bool>,
}

impl From<PulseOpts> for PulseSpec {
    fn from(opts: PulseOpts) -> Self {
        Self {
            retain: opts.retain.unwrap_or(30),
            label: Label::from_options(opts.suffix, opts.divisor),
            range: Range {
                min: Bound::from_options(opts.min, opts.lower),
                max: Bound::from_options(opts.max, opts.higher),
            },
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Pulse {
    tracer: BindedTracer<PulseState>,
}

impl Pulse {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: impl Into<PulseSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    pub fn push(&self, value: impl Into<f64>) {
        if let Some(value) = timed(value.into()) {
            let msg = PulseEvent::Push { value };
            self.tracer.send(msg, None);
        }
    }
}
