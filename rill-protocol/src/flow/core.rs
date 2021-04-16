use crate::encoding;
use crate::io::provider::{PackedDelta, PackedEvent, PackedState, StreamType, Timestamp};
use anyhow::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cmp::Ordering;
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
    type Event: DataFraction;

    fn stream_type() -> StreamType;

    fn apply(&mut self, event: TimedEvent<Self::Event>);

    fn pack_state(&self) -> Result<PackedState, Error> {
        encoding::pack(self)
    }

    fn unpack_state(data: &PackedState) -> Result<Self, Error> {
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

impl<T> Ord for TimedEvent<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl<T> PartialOrd for TimedEvent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for TimedEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl<T> Eq for TimedEvent<T> {}
