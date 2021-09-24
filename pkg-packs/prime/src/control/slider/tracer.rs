use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct SliderOpts {
    pub label: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub instant: Option<bool>,
}

impl From<SliderOpts> for SliderSpec {
    fn from(opts: SliderOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_else(|| "Slider".into()),
            min: opts.min.unwrap_or(1.0),
            max: opts.max.unwrap_or(100.0),
            step: opts.step.unwrap_or(1.0),
            instant: opts.instant.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Slider {
    tracer: BindedTracer<SliderState>,
}

impl Slider {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<SliderSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self, set_value: impl Into<f64>) {
        let msg = SliderEvent {
            set_value: set_value.into(),
        };
        self.tracer.send(msg, None);
    }
}
