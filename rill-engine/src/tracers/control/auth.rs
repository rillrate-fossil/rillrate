use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::auth::{AuthEvent, AuthState};
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
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Set state to authorized.
    pub fn login(&self) {
        let data = AuthEvent::Authorized(true);
        self.tracer.send(data, None);
    }

    /// Set state to failed.
    pub fn failed(&self) {
        let data = AuthEvent::Failed;
        self.tracer.send(data, None);
    }

    /// Set state to unauthorized.
    pub fn logout(&self) {
        let data = AuthEvent::Authorized(false);
        self.tracer.send(data, None);
    }

    /*
    /// Wait for the auth event.
    pub async fn watch_auth(&mut self) -> Result<AuthEvent, Error> {
        self.tracer.recv().await.map(TimedEvent::into_inner)
    }
    */
}
