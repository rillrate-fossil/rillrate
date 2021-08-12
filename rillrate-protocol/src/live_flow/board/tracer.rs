use super::state::*;
use crate::base::list_flow::ListFlowTracer;
use crate::live_flow::auto_path::AutoPath;
use crate::manifest::Binded;

pub struct Board {
    tracer: Binded<ListFlowTracer<BoardSpec>>,
}

impl Board {
    pub fn new(auto_path: AutoPath) -> Self {
        let path = auto_path.into();
        let tracer = Binded::new(ListFlowTracer::new(path).0);
        Self { tracer }
    }

    pub fn set(&self, key: impl ToString, value: impl ToString) {
        self.tracer.add_record(key.to_string(), value.to_string());
    }
}
