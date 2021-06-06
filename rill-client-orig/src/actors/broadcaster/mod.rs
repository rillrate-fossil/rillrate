mod actor;
pub use actor::Broadcaster;

mod link;
pub use link::{BroadcasterLinkForClient, BroadcasterLinkForProvider};

use meio::Action;
use rill_protocol::io::provider::{Description, EntryId};

#[derive(Debug, Clone)]
pub enum PathNotification {
    Name { name: EntryId },
    Paths { descriptions: Vec<Description> },
}

impl Action for PathNotification {}
