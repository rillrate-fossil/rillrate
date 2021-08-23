use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Table {
    tracer: BindedTracer<TableState>,
}

impl Table {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: impl Into<TableSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    /// Adds a new row
    pub fn add_row(&self, row: Row) {
        let event = TableEvent::AddRow { row };
        self.tracer.send(event, None);
    }

    /// Deletes a row by id
    pub fn del_row(&self, row: Row) {
        let event = TableEvent::DelRow { row };
        self.tracer.send(event, None);
    }

    /// Sets a value to the cell
    pub fn set_cell(&self, row: Row, col: Col, value: impl ToString) {
        let event = TableEvent::SetCell {
            row,
            col,
            value: value.to_string(),
        };
        self.tracer.send(event, None);
    }
}
