use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Counter {
    #[deref]
    #[deref_mut]
    tracer: Tracer<CounterState>,
    _binder: Binder,
}

impl Counter {
    // TODO: Use `ms` here and move `realtime` paramter to the rillrate constructor
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: CounterSpec) -> Self {
        let path = auto_path.into();
        let state = spec.into();
        let tracer = Tracer::new(state, path.into(), mode);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn inc(&self, delta: impl Into<i64>) {
        let msg = CounterEvent::Inc {
            delta: delta.into(),
        };
        self.tracer.send(msg, None);
    }
}
