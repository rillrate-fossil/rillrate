//! Contains generic implementation of a watcher.

use rill_protocol::inflow::action;

pub(crate) enum WatcherMode<T: action::Inflow> {
    Push { callback: Box<dyn FnMut(T::Event)> },
}

pub struct Watcher<T: action::Inflow> {
    mode: WatcherMode<T>,
}
