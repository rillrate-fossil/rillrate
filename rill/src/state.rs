use crate::actors::worker::RillLink;
use once_cell::sync::OnceCell;

/// It used by tracers to register them into the state.
pub(crate) static RILL_LINK: OnceCell<RillLink> = OnceCell::new();
