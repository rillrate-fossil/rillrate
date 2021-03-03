use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillEvent, StreamType};
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct TableTracer {
    tracer: Tracer<TableRecord>,
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
        let tracer = Tracer::new(description);
        Self { tracer }
    }

    /// Adds a new column
    pub fn add_col(&self, id: ColId, alias: Option<String>) {
        let data = TableRecord::AddColumn { col: id, alias };
        self.tracer.send(data, None);
    }

    /// Deletes a column by id
    pub fn del_col(&self, id: ColId) {
        let data = TableRecord::DelColumn { col: id };
        self.tracer.send(data, None);
    }

    /// Adds a new row
    pub fn add_row(&self, id: RowId, alias: Option<String>) {
        let data = TableRecord::AddRow { row: id, alias };
        self.tracer.send(data, None);
    }

    /// Deletes a row by id
    pub fn del_row(&self, id: RowId) {
        let data = TableRecord::DelRow { row: id };
        self.tracer.send(data, None);
    }

    /// Sets a value to the cell
    pub fn set_cell(
        &self,
        col: ColId,
        row: RowId,
        value: impl ToString,
        timestamp: Option<SystemTime>,
    ) {
        let data = TableRecord::SetCell {
            row,
            col,
            value: value.to_string(),
        };
        self.tracer.send(data, timestamp);
    }
}

#[derive(Debug)]
pub struct ColId(pub u64);

#[derive(Debug)]
pub struct RowId(pub u64);

#[derive(Debug)]
pub enum TableRecord {
    AddColumn {
        col: ColId,
        alias: Option<String>,
    },
    DelColumn {
        col: ColId,
    },
    AddRow {
        row: RowId,
        alias: Option<String>,
    },
    DelRow {
        row: RowId,
    },
    SetCell {
        row: RowId,
        col: ColId,
        value: String,
    },
}

impl TracerEvent for TableRecord {
    type State = TableState;
}

#[derive(Debug, Default)]
pub struct TableState {}

impl TracerState for TableState {
    type Item = TableRecord;

    fn aggregate(
        &mut self,
        _items: Vec<DataEnvelope<Self::Item>>,
        _outgoing: Option<&mut Vec<RillEvent>>,
    ) {
        todo!()
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        todo!()
    }
}
