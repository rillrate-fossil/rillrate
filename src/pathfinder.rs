use crate::protocol::{EntryId, Path};
use derive_more::{Deref, DerefMut};
use std::collections::HashMap;

/// Universal storage with `EntryId` hierarchy.
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

#[derive(Debug)]
pub struct Record<T> {
    subs: HashMap<EntryId, Record<T>>,
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

pub struct Discovered<'a, T> {
    pub remained_path: Path,
    pub record: &'a Record<T>,
}

impl<T> Record<T> {
    /// Creates nodes for the provided `Path`.
    ///
    /// It returns empty record if value is not exists and you
    /// have to use `set_link` method to assign a value to it.
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
        let mut remained = Vec::new();
        while let Some(element) = iter.next() {
            if let Some(next_record) = record.subs.get(element) {
                record = next_record;
            } else {
                remained.push(element.clone());
                break;
            }
        }
        remained.extend(iter.cloned());
        Discovered {
            remained_path: Path::from(remained),
            record,
        }
    }

    pub fn remove(&mut self, path: &Path) -> Option<Self> {
        let mut record = self;
        let mut iter = path.as_ref().iter();
        while let Some(element) = iter.next() {
            if iter.len() == 0 {
                return record.subs.remove(element);
            } else {
                if let Some(next_record) = record.subs.get_mut(element) {
                    record = next_record;
                } else {
                    break;
                }
            }
        }
        None
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
