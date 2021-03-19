pub mod counter;
pub mod dict;
pub mod gauge;
pub mod histogram;
pub mod logger;
pub mod pulse;
pub mod table;

use crate::encoding;
use crate::io::provider::{PackedDelta, PackedMetric, PackedState, StreamType, Timestamp};
use anyhow::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

pub trait Metric:
    DeserializeOwned + Serialize + Clone + fmt::Debug + Sync + Send + 'static
{
    type State: DeserializeOwned + Serialize + Clone + fmt::Debug + Send + 'static;
    type Event: DeserializeOwned + Serialize + Clone + fmt::Debug + Send + 'static;

    fn stream_type() -> StreamType;

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>);

    fn pack_metric(&self) -> Result<PackedMetric, Error> {
        encoding::to_vec(self)
            .map_err(Error::from)
            .map(PackedMetric::from)
    }

    fn unpack_metric(data: &PackedMetric) -> Result<Self, Error> {
        encoding::from_slice(&data.0).map_err(Error::from)
    }

    fn pack_delta(delta: &Delta<Self::Event>) -> Result<PackedDelta, Error> {
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
