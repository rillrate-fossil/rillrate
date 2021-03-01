mod actor;
pub use actor::Broadcaster;

mod link;
pub use link::{BroadcasterLinkForClient, BroadcasterLinkForProvider};

use meio::Action;
use rill_protocol::provider::{Description, EntryId};

#[derive(Debug, Clone)]
pub enum PathNotification {
    Name { name: EntryId },
    Paths { descriptions: Vec<Description> },
}

impl Action for PathNotification {}

/*
/// An `Actor` that exports metrics to a third-party system.
pub trait Publisher: Actor + StartedBy<Broadcaster> + InterruptedBy<Broadcaster> {
    type Config: Send;
    fn create(
        config: Self::Config,
        exporter: BroadcasterLinkForClient,
        server: &HttpServerLink,
    ) -> Self;
}
*/
