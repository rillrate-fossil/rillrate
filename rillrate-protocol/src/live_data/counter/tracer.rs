use super::state::*;
use crate::manifest::descriptions_flow::DescriptionBinder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{Tracer, Watcher};
use rill_protocol::io::provider::Path;
use std::time::Duration;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CounterTracer {
    #[deref]
    #[deref_mut]
    tracer: Tracer<CounterState>,
    binder: DescriptionBinder,
}

impl CounterTracer {
    pub fn new(path: Path, pull: Option<u64>) -> Self {
        let state = CounterState::new();
        let tracer = {
            if let Some(ms) = pull {
                let duration = Duration::from_millis(ms);
                Tracer::new_pull(state, path, duration)
            } else {
                Tracer::new_push(state, path).0
            }
        };
        let binder = DescriptionBinder::new(&tracer);
        Self { tracer, binder }
    }

    pub fn inc(&self, delta: i64) {
        let msg = CounterEvent::Inc { delta };
        self.tracer.send(msg, None);
    }
}
