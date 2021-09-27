use super::FixedPath;
use rill_protocol::io::provider::EntryId;

pub type AutoPath = FixedPath<4>;

impl AutoPath {
    pub fn package(&self) -> &EntryId {
        &self.entries[0]
    }

    pub fn dashboard(&self) -> &EntryId {
        &self.entries[1]
    }

    pub fn group(&self) -> &EntryId {
        &self.entries[2]
    }

    pub fn name(&self) -> &EntryId {
        &self.entries[3]
    }
}
