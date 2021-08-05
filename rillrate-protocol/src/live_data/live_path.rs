use rill_protocol::io::provider::{EntryId, Path};

/// `Live` bacause of `Live` product approach.
pub struct LivePath {
    pub package: EntryId,
    pub dashboard: EntryId,
    pub group: EntryId,
    pub name: EntryId,
}

impl Into<Path> for LivePath {
    fn into(self) -> Path {
        vec![self.package, self.dashboard, self.group, self.name].into()
    }
}

impl From<[&str; 4]> for LivePath {
    fn from(array: [&str; 4]) -> Self {
        Self {
            package: array[0].into(),
            dashboard: array[1].into(),
            group: array[2].into(),
            name: array[3].into(),
        }
    }
}
