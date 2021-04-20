use crate::tracers::tracer::Tracer;
use anyhow::Error;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::sign_in::SignInState;
use rill_protocol::io::provider::Path;

/// Receives sign_ins from a user.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct SignInWatcher {
    tracer: Tracer<SignInState>,
}

impl SignInWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path) -> Self {
        let state = SignInState::new();
        let tracer = Tracer::new_watcher(state, path);
        Self { tracer }
    }

    /// Wait for the sign_in event.
    pub async fn watch_sign_in(&mut self) -> Result<(), Error> {
        self.tracer.recv().await.map(drop)
    }
}
