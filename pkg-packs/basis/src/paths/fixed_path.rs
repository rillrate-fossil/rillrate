use derive_more::From;
use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::iter::{repeat, FromIterator};

/// `Live` bacause of `Live` product approach.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, From)]
#[serde(from = "String", into = "String")]
pub struct FixedPath<const T: usize> {
    pub entries: [EntryId; T],
}

impl<const T: usize> FixedPath<T> {
    fn unassigned(name: EntryId) -> Self {
        let entry = EntryId::from("unassigned");
        let entries: [EntryId; T] = repeat(entry)
            .take(T - 1)
            .chain([name])
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self { entries }
    }
}

impl<const T: usize> From<FixedPath<T>> for Path {
    fn from(this: FixedPath<T>) -> Self {
        Path::from_iter(this.entries)
    }
}

impl<const T: usize> From<[&str; T]> for FixedPath<T> {
    fn from(array: [&str; T]) -> Self {
        let entries: [EntryId; T] = array
            .iter()
            .map(|item| EntryId::from(*item))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self::from(entries)
    }
}

impl<const T: usize> From<String> for FixedPath<T> {
    fn from(s: String) -> Self {
        let s: &str = s.as_ref();
        Self::from(s)
    }
}

impl<const T: usize> From<&str> for FixedPath<T> {
    fn from(s: &str) -> Self {
        let entries: Result<[EntryId; T], ()> = s
            .parse::<Path>()
            .map_err(drop)
            .map(Vec::from)
            .and_then(|vec| vec.try_into().map_err(drop));
        match entries {
            Ok(entries) => Self { entries },
            Err(_) => Self::unassigned(EntryId::from(s)),
        }
    }
}

impl<const T: usize> From<FixedPath<T>> for String {
    fn from(path: FixedPath<T>) -> Self {
        path.entries.join(".")
    }
}
