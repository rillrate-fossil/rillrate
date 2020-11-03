use crate::provider::DynamicJoint;
use log::{Log, Metadata, Record};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct LogDriver {
    providers: RwLock<HashMap<String, DynamicJoint>>,
}

impl LogDriver {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }
}

impl Log for LogDriver {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        {
            if let Some(joint) = self.providers.read().unwrap().get(metadata.target()) {
                return joint.is_active();
            }
        }
        let joint = DynamicJoint::create_and_register(metadata.target());
        let module = metadata.target().to_string();
        {
            self.providers.write().unwrap().insert(module, joint);
        }
        false
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let s = format!("{}", record.args());
            if let Some(joint) = self
                .providers
                .read()
                .unwrap()
                .get(record.metadata().target())
            {
                joint.log(s);
            }
        }
    }

    fn flush(&self) {}
}
