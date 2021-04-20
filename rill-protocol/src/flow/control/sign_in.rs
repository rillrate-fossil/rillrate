use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthState {
    Unauthorized,
    LoggingIn,
    Authorized,
    LoggingOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInState {
    pub auth_state: AuthState,
    pub last_change: Option<Timestamp>,
}

impl SignInState {
    pub fn new() -> Self {
        Self {
            auth_state: AuthState::Unauthorized,
            last_change: None,
        }
    }
}

impl Flow for SignInState {
    type Event = SignInEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.signin.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            SignInEvent::TrySignIn { .. } => {
                self.auth_state = AuthState::LoggingIn;
            }
            SignInEvent::TrySignOut => {
                self.auth_state = AuthState::LoggingOut;
            }
            SignInEvent::Authorized => {
                self.auth_state = AuthState::Authorized;
            }
            SignInEvent::Unauthorized => {
                self.auth_state = AuthState::Unauthorized;
            }
        }
        self.last_change = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignInEvent {
    // TODO: Split to `ControlEvent`
    TrySignIn { username: String, password: String },
    TrySignOut,

    // TODO: Split to `UpdateEvent`
    Authorized,
    Unauthorized,
}
