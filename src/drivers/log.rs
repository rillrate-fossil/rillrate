use crate::provider::{DynamicJoint, Joint};
use log::{Log, Metadata, Record};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct LogDriver {
    providers: RwLock<HashMap<String, DynamicJoint>>,
}

impl Log for LogDriver {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        if let Some(joint) = self.providers.read().unwrap().get(metadata.target()) {
            joint.provider().is_active()
        } else {
            let joint = DynamicJoint::create_and_register(metadata.target());
            let module = metadata.target().to_string();
            self.providers.write().unwrap().insert(module, joint);
            false
        }
    }

    fn log(&self, record: &Record<'_>) {
        todo!();
    }

    fn flush(&self) {}
}
