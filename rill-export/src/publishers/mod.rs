mod graphite;
pub use graphite::GraphitePublisher;

//mod prometheus;
//pub use prometheus::PrometheusPublisher;

use crate::actors::export::RillExport;
use meio::{Actor, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::BroadcasterLinkForClient;

/// An `Actor` that exports metrics to a third-party system.
pub trait Publisher: Actor + StartedBy<RillExport> + InterruptedBy<RillExport> {
    type Config: Send;
    fn create(
        config: Self::Config,
        exporter: BroadcasterLinkForClient,
        server: &HttpServerLink,
    ) -> Self;
}
