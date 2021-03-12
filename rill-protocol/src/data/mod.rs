pub mod counter;
pub mod dict;
pub mod gauge;
pub mod histogram;
pub mod logger;
pub mod pulse;
pub mod table;

use crate::encoding;
use crate::io::provider::Timestamp;
use anyhow::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

pub trait Metric: fmt::Debug + Send + 'static {
    type State: DeserializeOwned + Serialize + Clone + fmt::Debug + Send + 'static;
    type Event: DeserializeOwned + Serialize + Clone + fmt::Debug + Send + 'static;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>);

    fn pack_delta(delta: Delta<Self::Event>) -> Result<Vec<u8>, Error> {
        encoding::to_vec(&delta).map_err(Error::from)
    }

    fn unpack_delta(data: Vec<u8>) -> Result<Delta<Self::Event>, Error> {
        encoding::from_slice(&data).map_err(Error::from)
    }

    fn pack_state(state: Self::State) -> Result<Vec<u8>, Error> {
        encoding::to_vec(&state).map_err(Error::from)
    }

    fn unpack_state(data: Vec<u8>) -> Result<Self::State, Error> {
        encoding::from_slice(&data).map_err(Error::from)
    }
}

pub type Delta<T> = Vec<TimedEvent<T>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}
