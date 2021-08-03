use super::state::*;
use crate::base::list_flow::ListFlowTracer;
use derive_more::{Deref, DerefMut};
//use rill_protocol::io::provider::Path;

// TODO: Use it internally in `Binder::join_group` method
#[derive(Deref, DerefMut)]
pub struct GroupsListTracer {
    tracer: ListFlowTracer<GroupsListSpec>,
}

impl GroupsListTracer {
    pub fn new() -> Self {
        Self {
            tracer: ListFlowTracer::new().0,
        }
    }

    /*
    pub fn add_group(&self, name: Path, paths: Vec<Path>) {
    }
    */
}
