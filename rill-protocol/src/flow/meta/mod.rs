pub mod entry;
pub mod path;
pub mod readyboard;

use crate::io::provider::Path;

pub trait MetaFlow: Default {
    fn location() -> Path;
}
