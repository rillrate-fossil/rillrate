use super::state::*;
use crate::live_flow::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{timed, ControlSender, Tracer};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Click {
    #[deref]
    #[deref_mut]
    tracer: Tracer<ClickState>,
    _binder: Binder,
}

impl Click {
    pub fn new(auto_path: AutoPath, caption: String, sender: ControlSender<ClickState>) -> Self {
        let path = auto_path.into();
        let state = ClickState::new(caption);
        let tracer = Tracer::new(state, path, None, Some(sender));
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn clicked(&self) {
        if let Some(msg) = timed(ClickEvent) {
            self.tracer.send(msg, None);
        }
    }
}
