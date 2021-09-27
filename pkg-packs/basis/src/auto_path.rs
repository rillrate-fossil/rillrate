use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};

const SIZE: usize = 4;

/// `Live` bacause of `Live` product approach.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub struct AutoPath {
    pub entries: [EntryId; 4],
    pub package: EntryId,
    pub dashboard: EntryId,
    pub group: EntryId,
    pub name: EntryId,
}

impl AutoPath {
    fn unassigned(name: EntryId) -> Self {
        let entry = EntryId::from("unassigned");
        [entry.clone(), entry.clone(), entry, name].into()
    }
}

impl From<AutoPath> for Path {
    fn from(this: AutoPath) -> Self {
        vec![this.package, this.dashboard, this.group, this.name].into()
    }
}

impl From<[EntryId; SIZE]> for AutoPath {
    fn from(entries: [EntryId; SIZE]) -> Self {
        Self {
            package: entries[0].clone(),
            dashboard: entries[1].clone(),
            group: entries[2].clone(),
            name: entries[3].clone(),
            entries,
        }
    }
}

impl From<[&str; SIZE]> for AutoPath {
    fn from(array: [&str; SIZE]) -> Self {
        let entries: [EntryId; SIZE] = [
            array[0].into(),
            array[1].into(),
            array[2].into(),
            array[3].into(),
        ];
        Self::from(entries)
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
                [
                    items.next().unwrap(),
                    items.next().unwrap(),
                    items.next().unwrap(),
                    items.next().unwrap(),
                ]
                .into()
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
