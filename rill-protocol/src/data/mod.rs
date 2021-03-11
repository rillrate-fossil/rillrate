pub mod counter;
pub mod dict;
pub mod gauge;
pub mod logger;
pub mod table;

use crate::io::provider::{StreamDelta, StreamState, Timestamp};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use thiserror::Error;

pub trait Convertable<T>: Into<T> + TryFrom<T, Error = ConvertError> {}

impl<B, T> Convertable<T> for B where Self: Into<T> + TryFrom<T, Error = ConvertError> {}

pub trait State: Convertable<StreamState> + Clone + Default + Send + 'static {
    type Event: Event;
    type Delta: Delta;

    fn apply(&mut self, update: Self::Delta);
}

pub trait Delta: Convertable<StreamDelta> + Default + Clone {
    type Event: Event;

    fn push(&mut self, event: TimedEvent<Self::Event>);
}

pub trait Event: Send + 'static {
    type State: State<Delta = Self::Delta, Event = Self>;
    type Delta: Delta<Event = Self>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}

#[derive(Debug, Error)]
#[error("Can't convert into the specific state of delta.")]
pub struct ConvertError;
