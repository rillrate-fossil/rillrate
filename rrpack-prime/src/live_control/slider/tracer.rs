use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Slider {
    #[deref]
    #[deref_mut]
    tracer: Tracer<SliderState>,
    _binder: Binder,
}

impl Slider {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        mode: FlowMode,
        label: impl ToString,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        let path = auto_path.into();
        let state = SliderState::new(label.to_string(), min, max, step);
        let tracer = Tracer::new(state, path.into(), mode);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn apply(&self, set_value: f64) {
        let msg = SliderEvent { set_value };
        self.tracer.send(msg, None);
    }
}
