use crate::tracers::tracer::Tracer;
use anyhow::Error;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::auth::{AuthEvent, AuthState};
use rill_protocol::flow::core::TimedEvent;
use rill_protocol::io::provider::Path;

/// Receives auths from a user.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct AuthWatcher {
    tracer: Tracer<AuthState>,
}

impl AuthWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path) -> Self {
        let state = AuthState::new();
        let tracer = Tracer::new_watcher(state, path);
        Self { tracer }
    }

    /*
    /// Wait for the auth event.
    pub async fn watch_auth(&mut self) -> Result<AuthEvent, Error> {
        self.tracer.recv().await.map(TimedEvent::into_inner)
    }
    */
}
