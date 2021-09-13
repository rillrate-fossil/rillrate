use rate_ui::shared_object::{RouterState, SharedObject};
use rate_ui::storage::typed_storage::Storable;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

thread_local! {
    pub static ALERTS: SharedObject<ToastState> = SharedObject::new();
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct TimedAlert {
    pub origin: String,
    pub ms: u64,
    pub message: String,
}

impl TimedAlert {
    pub fn new(origin: String, message: String) -> Self {
        let ms = js_sys::Date::now();
        Self {
            origin,
            ms: ms as u64,
            message,
        }
    }

    fn renew(&mut self) {
        self.ms = js_sys::Date::now() as u64;
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct ToastState {
    pub alerts: VecDeque<TimedAlert>,
}

impl Storable for ToastState {
    fn key() -> &'static str {
        module_path!()
    }
}

impl RouterState for ToastState {
    fn restored(&mut self) {
        // TODO: Renew items to show remained alerts again.
        for alert in &mut self.alerts {
            alert.renew();
        }
    }
}
