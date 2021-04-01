pub mod connection;
pub mod entry;
pub mod path;
pub mod ready_board;

use crate::io::provider::Path;

pub trait MetaFlow: Default {
    fn location() -> Path;
}
