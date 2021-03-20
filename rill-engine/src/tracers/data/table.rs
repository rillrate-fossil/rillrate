use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::table::{ColRecord, TableEvent, TableMetric, TableState};
use rill_protocol::io::provider::{Col, Path, Row};
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct TableTracer {
    tracer: Tracer<TableMetric>,
}

impl TableTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path, columns: Vec<(Col, Option<String>)>) -> Self {
        let columns = columns
            .into_iter()
            .map(|(col_id, alias)| {
                let record = ColRecord { alias };
                (col_id, record)
            })
            .collect();
        let metric = TableMetric { columns };
        let state = TableState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /*
    /// Adds a new column
    pub fn add_col(&self, id: Col, alias: Option<String>) {
        let event = TableEvent::AddCol { col: id, alias };
        self.tracer.send(event, None);
    }

    /// Deletes a column by id
    pub fn del_col(&self, id: Col) {
        let event = TableEvent::DelCol { col: id };
        self.tracer.send(event, None);
    }
    */

    /// Adds a new row
    pub fn add_row(&self, id: Row, alias: Option<String>) {
        let event = TableEvent::AddRow { row: id, alias };
        self.tracer.send(event, None);
    }

    /// Deletes a row by id
    pub fn del_row(&self, id: Row) {
        let event = TableEvent::DelRow { row: id };
        self.tracer.send(event, None);
    }

    /// Sets a value to the cell
    pub fn set_cell(
        &self,
        row: Row,
        col: Col,
        value: impl ToString,
        timestamp: Option<SystemTime>,
    ) {
        let event = TableEvent::SetCell {
            row,
            col,
            value: value.to_string(),
        };
        self.tracer.send(event, timestamp);
    }
}
