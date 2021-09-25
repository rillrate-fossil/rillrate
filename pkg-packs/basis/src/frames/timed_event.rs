use rill_protocol::io::provider::Timestamp;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::{SystemTime, SystemTimeError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}

impl<T> TimedEvent<T> {
    pub fn into_inner(self) -> T {
        self.event
    }
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

/// Wraps with timed event
pub fn timed<T>(event: T) -> Option<TimedEvent<T>> {
    time_to_ts(None)
        .map(move |timestamp| TimedEvent { timestamp, event })
        .ok()
}

/// Generates a `Timestamp` of converts `SystemTime` to it.
// TODO: How to avoid errors here?
pub fn time_to_ts(opt_system_time: Option<SystemTime>) -> Result<Timestamp, SystemTimeError> {
    opt_system_time
        .unwrap_or_else(SystemTime::now)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(Timestamp::from)
}
