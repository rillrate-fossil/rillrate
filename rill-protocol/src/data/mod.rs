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
    //type Delta: Delta<Event = Self::Event> + Into<Vec<TimedEvent<Self::Event>>>;

    fn apply(&mut self, event: TimedEvent<Self::Event>);

    // TODO: Use `struct Delta` here.
    fn wrap(events: Vec<TimedEvent<Self::Event>>) -> StreamDelta;
    fn try_extract(delta: StreamDelta) -> Result<Vec<TimedEvent<Self::Event>>, ConvertError>;
}

pub type Delta<T> = Vec<TimedEvent<T>>;

/*
pub trait Delta: Convertable<StreamDelta> + Default + Clone {
    type Event;

    fn push(&mut self, event: TimedEvent<Self::Event>);
}
*/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}

#[derive(Debug, Error)]
#[error("Can't convert into the specific state of delta.")]
pub struct ConvertError;
