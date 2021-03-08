use rill_client::actors::client::StateOrDelta;
use rill_protocol::data::{counter, gauge};
use rill_protocol::io::provider::Timestamp;

pub struct Converter<T> {
    state: Option<T>,
}

impl<T> Converter<T> {
    pub fn process_state_or_delta(&mut self, msg: StateOrDelta) {}
}

/// Converts a state and deltas into a flow of numeric values
/// suitable for the metrics tracing systems.
pub trait Extractor {
    fn to_value(&self) -> Option<(Timestamp, f64)>;
}

impl Extractor for counter::CounterState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        self.timestamp.map(|ts| (ts, self.value))
    }
}

impl Extractor for gauge::GaugeState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        self.frame
            .iter()
            .last()
            .map(|point| (point.timestamp, point.value))
    }
}
