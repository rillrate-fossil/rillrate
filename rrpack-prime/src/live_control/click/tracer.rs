use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{ActionSender, Tracer};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Click {
    #[deref]
    #[deref_mut]
    tracer: Tracer<ClickState>,
    _binder: Binder,
}

impl Click {
    pub fn new(auto_path: impl Into<AutoPath>, label: impl ToString) -> Self {
        let path = auto_path.into();
        let state = ClickState::new(label.to_string());
        let tracer = Tracer::new(state, path.into(), None);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn clicked(&self) {
        let msg = ClickEvent;
        self.tracer.send(msg, None);
    }
}
