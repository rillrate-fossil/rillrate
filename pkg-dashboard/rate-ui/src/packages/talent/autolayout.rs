use itertools::Itertools;
use std::cmp::{Ord, Ordering};
use std::collections::hash_map::{Entry, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRect {
    pub width: usize,
    pub height: usize,
}

pub struct AutoLayout<K, V> {
    layout_cell: LayoutRect,
    vacant: Vec<usize>,
    items: Vec<Option<Item<V>>>,
    pointers: HashMap<K, usize>,
}

impl<K, V> AutoLayout<K, V> {
    pub fn new(layout_cell: LayoutRect) -> Self {
        Self {
            layout_cell,
            vacant: Vec::new(),
            items: Vec::new(),
            pointers: HashMap::new(),
        }
    }

    pub fn change_cell(&mut self, layout_cell: LayoutRect) {
        self.layout_cell = layout_cell;
    }
}

impl<K, V> AutoLayout<K, V> {
    pub fn keys(&self) -> impl Iterator<Item = &'_ K> {
        self.pointers.keys()
    }

    /// `Option` provided to let components `reserve` the space
    /// or aviod relayout of the all elements by diffs.
    pub fn values(&self) -> impl Iterator<Item = Option<&Item<V>>> {
        self.items.iter().map(Option::as_ref)
    }
}

impl<K, V> AutoLayout<K, V>
where
    V: Ord,
{
    pub fn relayout(&mut self, real_width: f64, _real_height: f64) -> usize {
        // BASIC PARAMETERS
        let view_width = real_width as usize;
        let rounded_cols = (view_width / self.layout_cell.width).max(1);
        let col_width = view_width / rounded_cols;
        let row_height = self.layout_cell.height;

        // ITEMS (FILTERED, SORTED)
        let mut items = self
            .items
            .iter_mut()
            .map(Option::as_mut)
            .flatten()
            .sorted_by(Item::by_priority)
            .peekable();

        let mut row_num = 0;
        let mut col_shift = 0;
        while let Some(item) = items.next() {
            let item_height = item.preffered_size.rows * row_height;
            let mut remained_cols = rounded_cols - col_shift;
            let mut occupied_cols = item
                .preffered_size
                .cols
                .min(remained_cols)
                .min(rounded_cols);
            let occupied_rows = item.preffered_size.rows;
            remained_cols -= occupied_cols;
            let mut stretch_x = false;
            if let Some(next_item) = items.peek() {
                // THE NEXT ELEMENT
                //
                // If not enough space to put the next element then
                // fill the all remained space with the current element.
                if remained_cols < next_item.preffered_size.cols {
                    stretch_x = true;
                }
                if occupied_rows != next_item.preffered_size.rows {
                    stretch_x = true;
                }
            } else {
                // THE LAST ELEMENT
                //
                // If it's the last element that requires more space
                // than a single column.
                // Or if more than half of the row occupied.
                let more_than_one_col = occupied_cols > 1;
                let more_than_half_occupied = remained_cols < (col_shift + occupied_cols);
                if more_than_one_col || more_than_half_occupied {
                    stretch_x = true;
                }
            }
            let real_cols = occupied_cols; // Don't stretch the content!
            if stretch_x {
                occupied_cols += remained_cols;
            }
            let item_width = real_cols * col_width;
            //let item_width = occupied_cols * col_width;
            let item_shift = col_shift * col_width;
            item.relocate(row_num * row_height, item_shift);
            item.resize(item_width, item_height);
            col_shift += occupied_cols;
            if col_shift >= rounded_cols {
                row_num += occupied_rows;
                col_shift = 0;
            }
        }
        // Render unfinished row (if it has at least something)
        if col_shift > 0 {
            row_num += 1;
        }
        // At least one row needed
        row_num * row_height // total_height
    }
}

impl<K, V> AutoLayout<K, V>
where
    K: Debug + Eq + Hash,
{
    pub fn acquire(&mut self, key: K, item: Item<V>) {
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
    }

    pub fn release(&mut self, key: K) {
        if let Some(idx) = self.pointers.remove(&key) {
            let cell = self.items.get_mut(idx).unwrap();
            *cell = None;
            self.vacant.push(idx);
        } else {
            log::error!("No cell in frame with the key: {:?}", key);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrefferedSize {
    // TODO: Hide flelds and give methods instead
    pub cols: usize,
    pub rows: usize,
    pub priority: usize,
}

pub struct Item<T> {
    // TODO: Hide flelds and give methods instead
    pub preffered_size: PrefferedSize,
    pub width: usize,
    pub height: usize,
    pub top: usize,
    pub left: usize,
    pub record: T,
}

impl<T> Item<T> {
    pub fn new(preffered_size: PrefferedSize, record: T) -> Self {
        Self {
            preffered_size,
            width: 200,
            height: 100,
            top: 0,
            left: 0,
            record,
        }
    }

    pub fn by_priority<'r, 's>(a: &'r &mut Self, b: &'s &mut Self) -> Ordering
    where
        T: Ord,
    {
        let ord = Ord::cmp(&a.preffered_size.priority, &b.preffered_size.priority);
        if ord == Ordering::Equal {
            Ord::cmp(&a.record, &b.record)
        } else {
            ord
        }
    }

    fn relocate(&mut self, top: usize, left: usize) {
        self.top = top;
        self.left = left;
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}
