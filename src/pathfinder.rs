use crate::protocol::{EntryId, Path};
use derive_more::{Deref, DerefMut};
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Debug)]
pub struct Record<T> {
    //meta_data: Option<MetaData>,
    subs: HashMap<EntryId, Record<T>>,
    //sources: HashSet<T>,
    link: Option<T>,
}

impl<T> Default for Record<T> {
    fn default() -> Self {
        Self {
            subs: HashMap::new(),
            link: None,
        }
    }
}

pub enum Discovered<'a, T> {
    Pointer {
        remained_path: Path,
        link: Option<&'a T>,
    },
}

impl<T> Record<T> {
    /// Creates nodes for the provided `Path`.
    pub fn dig(&mut self, path: Path) -> &mut Self {
        let mut record = self;
        let entries: Vec<_> = path.into();
        for element in entries {
            record = record.subs.entry(element).or_default();
        }
        record
    }

    /// Tries to find a `Record` for the `Path`, but it it's not
    /// exists than it returned the last record in a chain and the
    /// remained (unprocessed) `Path`.
    pub fn discover(&self, path: &Path) -> Discovered<'_, T> {
        let mut record = self;
        let mut iter = path.as_ref().iter();
        while let Some(element) = iter.next() {
            if let Some(next_record) = record.subs.get(element) {
                record = next_record;
            } else {
                break;
            }
        }
        Discovered::Pointer {
            remained_path: Path::from_iter(iter),
            link: record.get_link(),
        }
    }

    /// Returns the `Record` for the `Path` or `None` if the `Record` not
    /// exists for the path.
    pub fn find(&self, path: &Path) -> Option<&Self> {
        let mut record = self;
        for element in path.as_ref() {
            if let Some(next_record) = record.subs.get(element) {
                record = next_record;
            } else {
                return None;
            }
        }
        Some(record)
    }

    pub fn find_mut(&mut self, path: &Path) -> Option<&mut Self> {
        let mut record = self;
        for element in path.as_ref() {
            if let Some(next_record) = record.subs.get_mut(element) {
                record = next_record;
            } else {
                return None;
            }
        }
        Some(record)
    }

    pub fn list(&self) -> Vec<EntryId> {
        self.subs.keys().cloned().collect()
    }

    pub fn set_link(&mut self, link: T) -> Option<T> {
        let mut cell = Some(link);
        std::mem::swap(&mut self.link, &mut cell);
        cell
    }

    pub fn reset_link(&mut self) {
        self.link = None;
    }

    pub fn get_link(&self) -> Option<&T> {
        self.link.as_ref()
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Pathfinder<T> {
    root: Record<T>,
}

impl<T> Pathfinder<T> {
    pub fn new() -> Self {
        Self {
            root: Record::default(),
        }
    }
}
