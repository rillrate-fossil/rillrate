use super::state::*;
use crate::base::list_flow::{ListFlowTracer, ListFlowWatcher};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Description, Path};
use std::sync::Arc;

pub type DescriptionsFlowTracer = ListFlowTracer<DescriptionsFlowSpec>;

pub type DescriptionsFlowWatcher = ListFlowWatcher<DescriptionsFlowSpec>;

#[derive(Debug, Clone)]
pub struct DescriptionBinder {
    inner: Arc<DescriptionBinderInner>,
}

impl DescriptionBinder {
    pub fn new<T: Flow>(tracer: &Tracer<T>) -> Self {
        let path = tracer.path().clone();
        let description = tracer.description().clone();
        let inner = DescriptionBinderInner::new(path, description);
        Self {
            inner: Arc::new(inner),
        }
    }
}

static DESCRIPTIONS: Lazy<DescriptionsFlowTracer> = Lazy::new(|| DescriptionsFlowTracer::new().0);

#[derive(Debug)]
struct DescriptionBinderInner {
    path: Path,
}

impl DescriptionBinderInner {
    fn new(path: Path, description: Description) -> Self {
        DESCRIPTIONS.add_record(path.clone(), description);
        Self { path }
    }
}

impl Drop for DescriptionBinderInner {
    fn drop(&mut self) {
        DESCRIPTIONS.remove_record(self.path.clone());
    }
}
