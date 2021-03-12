pub mod counter;
pub mod dict;
pub mod gauge;
pub mod logger;
pub mod table;

use crate::io::provider::{StreamDelta, StreamState, Timestamp};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

pub trait Convertable<T>: Into<T> + TryFrom<T, Error = ConvertError> {}

impl<B, T> Convertable<T> for B where Self: Into<T> + TryFrom<T, Error = ConvertError> {}

pub trait State: Convertable<StreamState> + Clone + Default + fmt::Debug + Send + 'static {
    type Event: Clone + fmt::Debug + Send + 'static;

    fn apply(&mut self, event: TimedEvent<Self::Event>);

    // TODO: Replace with `pack_*` methods
    fn wrap(events: Delta<Self::Event>) -> StreamDelta;
    fn try_extract(delta: StreamDelta) -> Result<Delta<Self::Event>, ConvertError>;

    // TODO: Add methods:
    // - `pack_state(self) -> Vec<u8>`
    // - `unpack_state(data: Vec<u8>) -> Self`
    // - `pack_delta(delta: Delta<Self::Event>) -> Vec<u8>`
    // - `unpack_delta(data: Vec<u8>) -> Delta<Self::Event>`
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
