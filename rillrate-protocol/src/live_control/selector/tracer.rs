use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{ControlSender, Tracer};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Selector {
    #[deref]
    #[deref_mut]
    tracer: Tracer<SelectorState>,
    _binder: Binder,
}

impl Selector {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        label: impl ToString,
        options: Vec<String>,
        sender: ControlSender<SelectorState>,
    ) -> Self {
        let path = auto_path.into();
        let state = SelectorState::new(label.to_string(), options);
        let tracer = Tracer::new(state, path.into(), None, Some(sender));
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn select(&self, value: Option<String>) {
        let msg = SelectorEvent {
            update_selected: value,
        };
        self.tracer.send(msg, None);
    }
}
