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
    pub fn discover(&mut self, path: Path) -> &mut Self {
        let mut record = self;
        let entries: Vec<_> = path.into();
        for element in entries {
            record = record.subs.entry(element).or_default();
        }
        record
    }

    pub fn list(&self) -> Vec<EntryId> {
        self.subs.keys().cloned().collect()
    }

    pub fn set_link(&mut self, link: Option<T>) {
        self.link = link;
    }

    pub fn get_link(&mut self) -> Option<&T> {
        self.link.as_ref()
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Pathfinder<T> {
    root: Record<T>,
}
