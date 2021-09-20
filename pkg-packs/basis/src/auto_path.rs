use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};

const SIZE: usize = 4;

/// `Live` bacause of `Live` product approach.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub struct AutoPath {
    pub package: EntryId,
    pub dashboard: EntryId,
    pub group: EntryId,
    pub name: EntryId,
}

impl AutoPath {
    fn unassigned(name: EntryId) -> Self {
        let entry = EntryId::from("unassigned");
        Self {
            package: entry.clone(),
            dashboard: entry.clone(),
            group: entry,
            name,
        }
    }
}

impl From<AutoPath> for Path {
    fn from(this: AutoPath) -> Self {
        vec![this.package, this.dashboard, this.group, this.name].into()
    }
}

impl From<[&str; SIZE]> for AutoPath {
    fn from(array: [&str; SIZE]) -> Self {
        Self {
            package: array[0].into(),
            dashboard: array[1].into(),
            group: array[2].into(),
            name: array[3].into(),
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
                    group: items.next().unwrap(),
                    name: items.next().unwrap(),
                }
            }
            _ => Self::unassigned(EntryId::from(s)),
        }
    }
}

impl From<AutoPath> for String {
    fn from(path: AutoPath) -> Self {
        format!(
            "{}.{}.{}.{}",
            path.package, path.dashboard, path.group, path.name
        )
    }
}
