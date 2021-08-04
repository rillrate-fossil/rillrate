use super::state::*;
use crate::base::list_flow::ListFlowTracer;
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct DescriptionsListTracer {
    tracer: ListFlowTracer<DescriptionsListSpec>,
}

impl DescriptionsListTracer {
    pub fn new() -> Self {
        Self {
            tracer: ListFlowTracer::new().0,
        }
    }
}
