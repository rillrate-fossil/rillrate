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

    pub fn of_server(&self) -> Path {
        self.of(server())
    }

    pub fn of_client(&self) -> Path {
        self.of(client())
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
