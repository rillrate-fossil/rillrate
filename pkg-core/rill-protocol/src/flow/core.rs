use crate::encoding;
use crate::io::provider::{PackedAction, PackedEvent, PackedState, ProviderReqId, StreamType};
use anyhow::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

// TODO: Move to the separate module
/// Requirements for a data fraction in a data flow.
pub trait DataFraction:
    DeserializeOwned + Serialize + Clone + fmt::Debug + Sync + Send + 'static
{
}

impl<T> DataFraction for T where
    T: DeserializeOwned + Serialize + Clone + fmt::Debug + Sync + Send + 'static
{
}

/// Immutable state of a data flow.
pub trait Flow: DataFraction {
    /// `ControlEvent` - that send from a client to a server
    type Action: DataFraction;

    /// `UpdateEvent` - that sent from a server to a client
    type Event: DataFraction;

    fn stream_type() -> StreamType;

    fn apply(&mut self, event: Self::Event);

    fn pack_state(&self) -> Result<PackedState, Error> {
        encoding::pack(self)
    }

    fn unpack_state(data: &PackedState) -> Result<Self, Error> {
        encoding::unpack(data)
    }

    fn pack_event(delta: &Self::Event) -> Result<PackedEvent, Error> {
        encoding::pack(delta)
    }

    fn unpack_event(data: &PackedEvent) -> Result<Self::Event, Error> {
        encoding::unpack(data)
    }

    fn pack_action(action: &Self::Action) -> Result<PackedAction, Error> {
        encoding::pack(action)
    }

    fn unpack_action(data: &PackedAction) -> Result<Self::Action, Error> {
        encoding::unpack(data)
    }
}

/// Envelope for incoming actions that contains routing information.
#[derive(Debug, Clone)]
pub struct ActionEnvelope<T: Flow> {
    /// Direction to a client.
    pub origin: ProviderReqId,
    /// The reason of sending an envelope.
    pub activity: Activity,
    /// An action sent to a clinet.
    /// It's detached from activity to make it suitable for
    /// third-party languages.
    pub action: Option<T::Action>,
}

/// Variant of activity that send to tracers.
///
/// It doesn't include `Action` value to make this type
/// compatible with languages that have no ADTs.
#[derive(Debug, Clone)]
pub enum Activity {
    /// No one connected client
    Suspend = 0, // 0b0000
    /// At least one client connected
    Awake = 1, // 0b0001

    /// Listener disconnected
    Disconnected = 2, // 0b0010
    /// Listener connected
    Connected = 3, // 0b0011

    /// Forwards an action
    Action = 4, // 0b010
}

impl Activity {
    pub fn is_action(&self) -> bool {
        matches!(self, Self::Action)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FlowMode {
    Realtime,
    Throttle { ms: u64 },
    FlushOnly,
}

impl Default for FlowMode {
    fn default() -> Self {
        Self::Realtime
    }
}
