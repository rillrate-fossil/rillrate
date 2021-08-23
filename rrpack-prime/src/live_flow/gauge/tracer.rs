use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use crate::range::{Bound, Range};
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Default)]
pub struct GaugeOpts {
    pub min: Option<f64>,
    pub lower: Option<bool>,
    pub max: Option<f64>,
    pub higher: Option<bool>,
}

impl From<GaugeOpts> for GaugeSpec {
    fn from(opts: GaugeOpts) -> Self {
        Self {
            range: Range {
                min: Bound::from_options(opts.min, opts.lower),
                max: Bound::from_options(opts.max, opts.higher),
            },
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Gauge {
    tracer: BindedTracer<GaugeState>,
}

impl Gauge {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: impl Into<GaugeSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    pub fn set(&self, value: impl Into<f64>) {
        let msg = GaugeEvent::Set {
            value: value.into(),
        };
        self.tracer.send(msg, None);
    }
}
