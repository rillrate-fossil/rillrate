use crate::io::provider::{EntryId, Path};

pub struct Location {
    element: &'static str,
}

impl Location {
    pub const fn new(element: &'static str) -> Self {
        Self { element }
    }

    pub fn of(&self, mut path: Path) -> Path {
        let entry_id: EntryId = EntryId::from(self.element);
        path.push(entry_id);
        path
    }

    pub fn root(&self) -> Path {
        Path::single(self.element)
    }
}

pub fn server() -> Path {
    Path::single("@server")
}

pub fn client() -> Path {
    Path::single("@self")
}

pub const ALERTS: Location = Location::new("meta:alerts");
pub const PATHS: Location = Location::new("meta:paths");
pub const READY_BOARDS: Location = Location::new("meta:readyboards");
