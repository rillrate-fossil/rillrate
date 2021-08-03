use super::state::*;
use crate::base::stat_flow::{StatFlowTracer, StatFlowWatcher};

pub type CounterStatTracer = StatFlowTracer<CounterStatSpec>;

pub type CounterStatWatcher = StatFlowWatcher<CounterStatSpec>;
