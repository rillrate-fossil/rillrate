use crate::provider::Joint;
use crate::StaticJoint;
use log::{Log, Metadata, Record};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct LogDriver {
    providers: RwLock<HashMap<String, StaticJoint>>,
}

impl Log for LogDriver {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        if let Some(joint) = self.providers.read().unwrap().get(metadata.target()) {
            joint.provider().is_active()
        } else {
            // TODO: Create a provider...
            false
        }
    }

    fn log(&self, record: &Record<'_>) {
        todo!();
    }

    fn flush(&self) {}
}
