use super::state::*;
use crate::base::list_flow::{ListFlowTracer, ListFlowWatcher};

pub type DescriptionsFlowTracer = ListFlowTracer<DescriptionsFlowSpec>;

pub type DescriptionsFlowWatcher = ListFlowWatcher<DescriptionsFlowSpec>;
