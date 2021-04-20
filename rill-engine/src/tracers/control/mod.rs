//! Module contains watchers.

pub(crate) mod click;
pub use click::ClickWatcher;

pub(crate) mod selector;
pub use selector::SelectorWatcher;

pub(crate) mod sign_in;
pub use sign_in::SignInWatcher;

pub(crate) mod toggle;
pub use toggle::ToggleWatcher;
