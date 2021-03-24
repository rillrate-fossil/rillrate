pub mod entry;

use crate::io::provider::EntryId;

pub trait MetaFlow: Default {
    fn location() -> EntryId;
}
