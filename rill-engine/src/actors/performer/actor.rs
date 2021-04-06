use rill_protocol::inflow::action;
use std::sync::Arc;

pub struct Performer<T: action::Inflow> {
    description: Arc<T>,
}
