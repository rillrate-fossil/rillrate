use crate::encoding;
use crate::io::provider::{PackedDelta, PackedFlow, PackedState, StreamType, Timestamp};
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

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>);

    fn pack_flow(&self) -> Result<PackedFlow, Error> {
        encoding::to_vec(self)
            .map_err(Error::from)
            .map(PackedFlow::from)
    }

    fn unpack_flow(data: &PackedFlow) -> Result<Self, Error> {
        encoding::from_slice(&data.0).map_err(Error::from)
    }

    fn pack_delta(delta: &[TimedEvent<Self::Event>]) -> Result<PackedDelta, Error> {
        encoding::to_vec(delta)
            .map_err(Error::from)
            .map(PackedDelta::from)
    }

    fn unpack_delta(data: &PackedDelta) -> Result<Delta<Self::Event>, Error> {
        encoding::from_slice(&data.0).map_err(Error::from)
    }

    fn pack_state(state: &Self::State) -> Result<PackedState, Error> {
        encoding::to_vec(state)
            .map_err(Error::from)
            .map(PackedState::from)
    }

    fn unpack_state(data: &PackedState) -> Result<Self::State, Error> {
        encoding::from_slice(&data.0).map_err(Error::from)
    }
}

pub type Delta<T> = Vec<TimedEvent<T>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}

/// `MetaFlow`s are flows that don't require getting a `Flow` object
/// to bootstrap a `State`.
pub trait MetaFlow: Default {}
