use super::state::*;
use crate::base_flow::list_flow::ListFlowTracer;
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct DescriptionsListTracer {
    tracer: ListFlowTracer<DescriptionsListSpec>,
}

impl DescriptionsListTracer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = DescriptionsListSpec::path();
        Self {
            tracer: ListFlowTracer::new(path).0,
        }
    }
}
