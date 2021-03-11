use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::table::{TableEvent, TableState};
use rill_protocol::io::provider::{ColId, Description, Path, RowId, StreamType};
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct TableTracer {
    tracer: Tracer<TableState>,
}

impl TableTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let info = format!("{} table", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::TableStream,
        };
        let tracer = Tracer::new(description, None);
        Self { tracer }
    }

    /// Adds a new column
    pub fn add_col(&self, id: ColId, alias: Option<String>) {
        let event = TableEvent::AddCol { col: id, alias };
        self.tracer.send(event, None);
    }

    /// Deletes a column by id
    pub fn del_col(&self, id: ColId) {
        let event = TableEvent::DelCol { col: id };
        self.tracer.send(event, None);
    }

    /// Adds a new row
    pub fn add_row(&self, id: RowId, alias: Option<String>) {
        let event = TableEvent::AddRow { row: id, alias };
        self.tracer.send(event, None);
    }

    /// Deletes a row by id
    pub fn del_row(&self, id: RowId) {
        let event = TableEvent::DelRow { row: id };
        self.tracer.send(event, None);
    }

    /// Sets a value to the cell
    pub fn set_cell(
        &self,
        row: RowId,
        col: ColId,
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
