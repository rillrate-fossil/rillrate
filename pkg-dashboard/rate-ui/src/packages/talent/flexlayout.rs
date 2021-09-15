use std::collections::btree_map::{BTreeMap, Entry};
use std::fmt::Debug;

pub struct FlexLayout<K, V> {
    vacant: Vec<usize>,
    items: Vec<Option<Item<V>>>,
    pointers: BTreeMap<K, usize>,
}

impl<K, V> Default for FlexLayout<K, V>
where
    K: Debug + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> FlexLayout<K, V>
where
    K: Debug + Ord,
{
    pub fn new() -> Self {
        Self {
            vacant: Vec::new(),
            items: Vec::new(),
            pointers: BTreeMap::new(),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &'_ K> {
        self.pointers.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = Option<&Item<V>>> {
        self.items.iter().map(Option::as_ref)
    }

    pub fn acquire(&mut self, key: K, value: V) {
        let item = Item {
            order: 0,
            record: value,
        };
        let entry = self.pointers.entry(key);
        let value = Some(item);
        match entry {
            Entry::Vacant(entry) => {
                if let Some(idx) = self.vacant.pop() {
                    let cell = self.items.get_mut(idx).unwrap();
                    *cell = value;
                    entry.insert(idx);
                } else {
                    entry.insert(self.items.len());
                    self.items.push(value);
                }
            }
            Entry::Occupied(entry) => {
                log::warn!("Attempt to add by the key {:?} twice.", entry.key());
            }
        }
        self.relayout();
    }

    pub fn release(&mut self, key: K) {
        if let Some(idx) = self.pointers.remove(&key) {
            let cell = self.items.get_mut(idx).unwrap();
            *cell = None;
            self.vacant.push(idx);
        } else {
            log::error!("No cell in frame with the key: {:?}", key);
        }
        self.relayout();
    }

    fn relayout(&mut self) {
        for (i, (_, idx)) in self.pointers.iter().enumerate() {
            let item = self
                .items
                .get_mut(*idx)
                .map(Option::as_mut)
                .and_then(std::convert::identity);
            if let Some(item) = item {
                item.order = i;
            }
        }
    }
}

pub struct Item<T> {
    pub order: usize,
    pub record: T,
}
