pub mod counter;
pub mod dict;
pub mod gauge;
pub mod histogram;
pub mod logger;
pub mod pulse;
pub mod table;

use crate::encoding;
use crate::io::provider::{PackedDelta, PackedMetric, PackedState, StreamType, Timestamp};
use crate::range::Range;
use anyhow::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

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

// TODO: Move to the same mode with Range and Frame
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Pct(f64);

impl Pct {
    pub fn from_value(mut value: f64) -> Self {
        // TODO: Use `clamp` here.
        if value < 0.0 {
            value = 0.0;
        } else if value > 1.0 {
            value = 1.0;
        }
        Self(value)
    }

    pub fn from_div(value: f64, total: f64) -> Self {
        let pct = {
            if total == 0.0 {
                0.0
            } else {
                value / total
            }
        };
        Pct::from_value(pct)
    }

    pub fn from_range(value: f64, range: &Range) -> Self {
        let value = value - range.min();
        let diff = range.diff();
        Pct::from_div(value, diff)
    }

    pub fn to_cent(&self) -> f64 {
        (self.0 * 100.0).round()
    }
}

impl Deref for Pct {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
