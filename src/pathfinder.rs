use crate::protocol::{EntryId, Path};
use derive_more::{Deref, DerefMut};
use std::collections::HashMap;

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
    pub fn discover(&self, path: &Path) -> (&Self, Path) {
        let mut record = self;
        let mut iter = path.as_ref().iter();
        let mut rem_path = Vec::new();
        while let Some(element) = iter.next() {
            if let Some(next_record) = record.subs.get(element) {
                record = next_record;
            } else {
                rem_path.push(element.clone());
                break;
            }
        }
        rem_path.extend(iter.cloned());
        (record, rem_path.into())
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

    pub fn list(&self) -> Vec<EntryId> {
        self.subs.keys().cloned().collect()
    }

    pub fn set_link(&mut self, link: T) {
        self.link = Some(link);
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
