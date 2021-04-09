use crate::encoding;
use crate::io::provider::{
    PackedDelta, PackedEvent, PackedFlow, PackedState, StreamType, Timestamp,
};
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
    type State: DataFraction;
    type Event: DataFraction;

    fn stream_type() -> StreamType;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>);

    fn pack_flow(&self) -> Result<PackedFlow, Error> {
        encoding::pack(self)
    }

    fn unpack_flow(data: &PackedFlow) -> Result<Self, Error> {
        encoding::unpack(data)
    }

    fn pack_state(state: &Self::State) -> Result<PackedState, Error> {
        encoding::pack(state)
    }

    fn unpack_state(data: &PackedState) -> Result<Self::State, Error> {
        encoding::unpack(data)
    }

    fn pack_delta(delta: &[TimedEvent<Self::Event>]) -> Result<PackedDelta, Error> {
        encoding::pack(delta)
    }

    fn unpack_delta(data: &PackedDelta) -> Result<Delta<Self::Event>, Error> {
        encoding::unpack(data)
    }

    fn pack_event(event: &Self::Event) -> Result<PackedEvent, Error> {
        encoding::pack(event)
    }

    fn unpack_event(data: &PackedEvent) -> Result<Self::Event, Error> {
        encoding::unpack(data)
    }
}

pub type Delta<T> = Vec<TimedEvent<T>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}
