use super::state::*;
use crate::base::list_flow::ListFlowTracer;
use crate::live_data::live_path::LivePath;
use crate::manifest::Binded;

pub struct BoardListTracer {
    tracer: Binded<ListFlowTracer<BoardListSpec>>,
}

impl BoardListTracer {
    pub fn new(live_path: LivePath) -> Self {
        let path = live_path.into();
        let tracer = Binded::new(ListFlowTracer::new(path).0);
        Self { tracer }
    }

    pub fn set(&self, key: String, value: String) {
        self.tracer.add_record(key, value);
    }
}
