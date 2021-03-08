mod graphite;
pub use graphite::GraphitePublisher;

mod prometheus;
pub use prometheus::PrometheusPublisher;

mod converter;
use converter::Converter;

use crate::actors::export::RillExport;
use anyhow::Error;
use meio::{Actor, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::BroadcasterLinkForClient;
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::io::provider::{StreamDelta, StreamState, Timestamp};
use thiserror::Error;

/// An `Actor` that exports metrics to a third-party system.
pub trait Publisher: Actor + StartedBy<RillExport> + InterruptedBy<RillExport> {
    type Config: Send;
    fn create(
        config: Self::Config,
        broadcaster: BroadcasterLinkForClient,
        client: ClientLink,
        // by reference, because it's optinal to use, but required to be present
        server: &HttpServerLink,
    ) -> Self;
}

#[derive(Debug, Error)]
enum ExtractError {
    #[error("Empty state yet.")]
    EmptyState,
    #[error("Convertion is not applicable.")]
    NotApplicable,
}

fn extract_value(msg: StateOrDelta) -> Result<(Timestamp, f64), ExtractError> {
    match msg {
        StateOrDelta::State(state) => match state {
            StreamState::Counter(state) => {
                let value = state.value;
                state
                    .timestamp
                    .map(|ts| (ts, value))
                    .ok_or(ExtractError::EmptyState)
            }
            StreamState::Gauge(state) => state
                .frame
                .iter()
                .last()
                .map(|point| (point.timestamp, point.value))
                .ok_or(ExtractError::EmptyState),
            StreamState::Table(state) => Err(ExtractError::NotApplicable),
            StreamState::Dict(state) => Err(ExtractError::NotApplicable),
            StreamState::Log(state) => Err(ExtractError::NotApplicable),
        },
        StateOrDelta::Delta(delta) => match delta {
            StreamDelta::Counter(delta) => {
                todo!()
            }
            StreamDelta::Gauge(delta) => {
                todo!()
            }
            StreamDelta::Table(delta) => Err(ExtractError::NotApplicable),
            StreamDelta::Dict(delta) => Err(ExtractError::NotApplicable),
            StreamDelta::Log(delta) => Err(ExtractError::NotApplicable),
        },
    }
}
