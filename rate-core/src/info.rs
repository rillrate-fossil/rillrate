use once_cell::sync::Lazy;
use rill_engine::tracers::meta::AlertTracer;
use rill_protocol::flow::meta::alert::ALERTS;

pub struct GlobalTracers {
    pub alerts: AlertTracer,
}

impl GlobalTracers {
    fn new() -> Self {
        Self {
            alerts: AlertTracer::new(ALERTS.root()),
        }
    }

    /// Mehtod to initialize `Lazy` cell.
    pub fn touch(&self) {}
}

pub static TRACERS: Lazy<GlobalTracers> = Lazy::new(GlobalTracers::new);
