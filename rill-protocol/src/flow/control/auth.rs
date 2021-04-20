use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InnerState {
    Unauthorized,
    LoggingIn,
    Authorized,
    LoggingOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub auth_state: InnerState,
    pub last_change: Option<Timestamp>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            auth_state: InnerState::Unauthorized,
            last_change: None,
        }
    }
}

impl Flow for AuthState {
    type Event = AuthEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.signin.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            AuthEvent::TrySignIn { .. } => {
                self.auth_state = InnerState::LoggingIn;
            }
            AuthEvent::TrySignOut => {
                self.auth_state = InnerState::LoggingOut;
            }
            AuthEvent::Authorized(true) => {
                self.auth_state = InnerState::Authorized;
            }
            AuthEvent::Authorized(false) => {
                self.auth_state = InnerState::Unauthorized;
            }
        }
        self.last_change = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEvent {
    // TODO: Split to `ControlEvent`
    TrySignIn { username: String, password: String },
    TrySignOut,

    // TODO: Split to `UpdateEvent`
    Authorized(bool),
}
