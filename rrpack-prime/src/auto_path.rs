use rill_protocol::io::provider::{EntryId, Path};

const SIZE: usize = 3;

/// `Live` bacause of `Live` product approach.
pub struct AutoPath {
    pub package: EntryId,
    pub dashboard: EntryId,
    pub name: EntryId,
}

impl AutoPath {
    fn unassigned(name: EntryId) -> Self {
        let entry = EntryId::from("unassigned");
        Self {
            package: entry.clone(),
            dashboard: entry,
            name,
        }
    }
}

impl From<AutoPath> for Path {
    fn from(this: AutoPath) -> Self {
        vec![this.package, this.dashboard, this.name].into()
    }
}

impl From<[&str; SIZE]> for AutoPath {
    fn from(array: [&str; SIZE]) -> Self {
        Self {
            package: array[0].into(),
            dashboard: array[1].into(),
            name: array[2].into(),
        }
    }
}

impl From<String> for AutoPath {
    fn from(s: String) -> Self {
        let s: &str = s.as_ref();
        Self::from(s)
    }
}

impl From<&str> for AutoPath {
    fn from(s: &str) -> Self {
        let path = s.parse::<Path>().map(Vec::from);
        match path {
            Ok(path) if path.len() == SIZE => {
                let mut items = path.into_iter();
                Self {
                    package: items.next().unwrap(),
                    dashboard: items.next().unwrap(),
                    name: items.next().unwrap(),
                }
            }
            _ => Self::unassigned(EntryId::from(s)),
        }
    }
}
