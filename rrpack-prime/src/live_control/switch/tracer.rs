use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{ActionSender, Tracer};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Switch {
    #[deref]
    #[deref_mut]
    tracer: Tracer<SwitchState>,
    _binder: Binder,
}

impl Switch {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        caption: impl ToString,
        sender: ActionSender<SwitchState>,
    ) -> Self {
        let path = auto_path.into();
        let state = SwitchState::new(caption.to_string());
        let tracer = Tracer::new(state, path.into(), None, Some(sender));
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn turn(&self, turn_on: bool) {
        let msg = SwitchEvent { turn_on };
        self.tracer.send(msg, None);
    }
}
