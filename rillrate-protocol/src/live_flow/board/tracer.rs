use super::state::*;
use crate::auto_path::AutoPath;
use crate::base_flow::list_flow::ListFlowTracer;
use crate::manifest::Binded;

pub struct Board {
    tracer: Binded<ListFlowTracer<BoardSpec>>,
}

impl Board {
    pub fn new(auto_path: impl Into<AutoPath>) -> Self {
        let path = auto_path.into();
        let tracer = Binded::new(ListFlowTracer::new(path.into()));
        Self { tracer }
    }

    pub fn set(&self, key: impl ToString, value: impl ToString) {
        self.tracer.add_record(key.to_string(), value.to_string());
    }
}
