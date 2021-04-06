use crate::watchers::watcher::Watcher;
use rill_protocol::inflow::action::click::ClickInflow;

pub struct ClickWatcher {
    watcher: Watcher<ClickInflow>,
}
