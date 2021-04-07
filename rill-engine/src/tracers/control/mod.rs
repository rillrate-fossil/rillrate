//! Module contains watchers.

pub(crate) mod click;
pub use click::ClickWatcher;

pub(crate) mod selector;
pub use selector::SelectorWatcher;

pub(crate) mod toggle;
pub use toggle::ToggleWatcher;
