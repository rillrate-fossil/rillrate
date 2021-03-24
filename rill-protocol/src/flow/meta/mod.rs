pub mod entry;
pub mod path;

use crate::io::provider::EntryId;

pub trait MetaFlow: Default {
    // TODO: Use `Path` here?
    fn location() -> EntryId;
}
