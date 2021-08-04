use super::state::*;
use crate::base::list_flow::ListFlowTracer;
use crate::manifest::Binded;
use rill_protocol::io::provider::EntryId;

pub struct BoardListTracer {
    tracer: Binded<ListFlowTracer<BoardListSpec>>,
}

impl BoardListTracer {
    pub fn new(group: EntryId, name: EntryId) -> Self {
        let path = vec![group, name].into();
        let tracer = Binded::new(ListFlowTracer::new(path).0);
        Self { tracer }
    }
}
