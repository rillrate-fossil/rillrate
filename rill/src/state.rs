use crate::actors::worker::RillLink;
use once_cell::sync::OnceCell;

/// It used by tracers to register them into the state.
pub(crate) static RILL_LINK: OnceCell<RillState> = OnceCell::new();

pub(crate) struct RillState {
    pub link: RillLink,
}
