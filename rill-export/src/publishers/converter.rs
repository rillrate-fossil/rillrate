use anyhow::Error;
use rill_client::actors::client::StateOrDelta;
use rill_protocol::data::{counter, dict, gauge, logger, table, State};
use rill_protocol::io::provider::{Description, StreamType, Timestamp};
use std::convert::TryFrom;
use thiserror::Error;

/*
impl Extractor {
    pub fn make_extractor(desc: &Description) -> Box<dyn Extractor> {
        match desc.stream_type {
            StreamType::LogStream => Box::new(Converter::<logger::LogState>::new()),
            StreamType::CounterStream => Box::new(Converter::<counter::CounterState>::new()),
            StreamType::GaugeStream => Box::new(Converter::<gauge::GaugeState>::new()),
            StreamType::DictStream => Box::new(Converter::<dict::DictState>::new()),
            StreamType::TableStream => Box::new(Converter::<table::TableState>::new()),
        }
    }
}

pub struct Converter<T: Extractor> {
    state: Option<T>,
}

impl<T: Extractor> Converter<T> {
    fn new() -> Self {
        Self { state: None }
    }
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
        self.state
            .as_ref()
            .map(Extractor::to_value)
            .and_then(std::convert::identity)
    }
}
*/

/// Converts a state and deltas into a flow of numeric values
/// suitable for the metrics tracing systems.
pub trait Extractor: State {
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

impl Extractor for logger::LogState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for table::TableState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        None
    }
}

impl Extractor for dict::DictState {
    fn to_value(&self) -> Option<(Timestamp, f64)> {
        None
    }
}
