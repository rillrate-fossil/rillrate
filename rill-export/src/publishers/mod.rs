mod graphite;
pub use graphite::GraphitePublisher;

mod prometheus;
pub use prometheus::PrometheusPublisher;

mod observer;
use observer::{SharedRecord, Observer};

use crate::actors::export::RillExport;
use anyhow::Error;
use meio::{Actor, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::BroadcasterLinkForClient;
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::io::provider::{StreamDelta, StreamState, Timestamp};

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
