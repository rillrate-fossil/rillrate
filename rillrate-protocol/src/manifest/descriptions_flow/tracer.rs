use super::state::*;
use crate::base::list_flow::{ListFlowTracer, ListFlowWatcher};
use once_cell::sync::Lazy;

// TODO: Use this flow in the every tracer high-level tracer.
pub static DESCRIPTIONS: Lazy<DescriptionsFlowTracer> =
    Lazy::new(|| DescriptionsFlowTracer::new().0);

pub type DescriptionsFlowTracer = ListFlowTracer<DescriptionsFlowSpec>;

pub type DescriptionsFlowWatcher = ListFlowWatcher<DescriptionsFlowSpec>;
