use anyhow::Error;
use rill_client::actors::client::StateOrDelta;
use rill_protocol::data::{counter, gauge, State};
use rill_protocol::io::provider::Timestamp;
use std::convert::TryFrom;
use thiserror::Error;

pub struct Converter<T: Extractor> {
    state: Option<T>,
}

impl<T> Extractor for Converter<T>
where
    T: Extractor + State,
{
    fn process_state_or_delta(&mut self, msg: StateOrDelta) -> Result<(), Error> {
        match msg {
            StateOrDelta::State(state) => {
                let state = T::try_from(state)?;
                self.state = Some(state);
            }
            StateOrDelta::Delta(delta) => {
                let delta = T::Delta::try_from(delta)?;
                if let Some(state) = self.state.as_mut() {
                    state.apply(delta);
                }
            }
        }
        Ok(())
    }

    fn to_value(&self) -> Option<(Timestamp, f64)> {
        self.state.as_ref().map(Extractor::to_value).and_then(std::convert::identity)
    }
}

/// Converts a state and deltas into a flow of numeric values
/// suitable for the metrics tracing systems.
pub trait Extractor {
    fn process_state_or_delta(&mut self, msg: StateOrDelta) -> Result<(), Error> {
        Err(Error::msg("not implemented for the state directly"))
    }

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
