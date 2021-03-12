pub mod counter;
pub mod dict;
pub mod gauge;
pub mod logger;
pub mod table;

use crate::io::provider::{StreamDelta, StreamState, Timestamp};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

pub trait Convertable<T>: Into<T> + TryFrom<T, Error = ConvertError> {}

impl<B, T> Convertable<T> for B where Self: Into<T> + TryFrom<T, Error = ConvertError> {}

pub trait Metric: fmt::Debug + Send + 'static {
    type State: Convertable<StreamState> + Clone + Default + fmt::Debug + Send + 'static;
    type Event: DeserializeOwned + Serialize + Clone + fmt::Debug + Send + 'static;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>);

    // TODO: Replace with `pack_*` methods
    fn wrap(events: Delta<Self::Event>) -> StreamDelta;
    fn try_extract(delta: StreamDelta) -> Result<Delta<Self::Event>, ConvertError>;

    fn pack_delta(delta: Delta<Self::Event>) -> Result<Vec<u8>, ConvertError> {
        serde_json::to_vec(&delta).map_err(|_| ConvertError)
    }

    fn unpack_delta(data: Vec<u8>) -> Result<Delta<Self::Event>, ConvertError> {
        serde_json::from_slice(&data).map_err(|_| ConvertError)
    }

    // TODO: Add methods:
    // - `pack_state(self) -> Result<Vec<u8>, ConvertError>`
    // - `unpack_state(data: Vec<u8>) -> Result<Self, ConvertError>`
}

pub type Delta<T> = Vec<TimedEvent<T>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}

#[derive(Debug, Error)]
#[error("Can't convert into the specific state of delta.")]
pub struct ConvertError;
