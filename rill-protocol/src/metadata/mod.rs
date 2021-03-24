pub mod entry;

use crate::io::provider::EntryId;

pub trait MetaMetric: Default {
    fn location() -> EntryId;
}
