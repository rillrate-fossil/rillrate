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
    pub fn new(path: Path, columns: Vec<(Col, String)>) -> Self {
        let columns = columns
            .into_iter()
            .map(|(col_id, title)| {
                let record = ColRecord { title };
                (col_id, record)
            })
            .collect();
        let metric = TableMetric { columns };
        let state = TableState::new();
        let tracer = Tracer::new(metric, state, path, None);
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
