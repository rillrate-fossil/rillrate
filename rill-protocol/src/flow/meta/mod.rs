pub mod entry;
pub mod path;

use crate::io::provider::Path;

pub trait MetaFlow: Default {
    fn location() -> Path;
}
