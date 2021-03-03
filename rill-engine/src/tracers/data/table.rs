use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{
    ColId, Description, Path, RillData, RillEvent, RowId, StreamType, TableUpdate,
};
use std::collections::HashMap;
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
        let update = TableUpdate::AddCol { col: id, alias };
        let data = TableRecord { update };
        self.tracer.send(data, None);
    }

    /// Deletes a column by id
    pub fn del_col(&self, id: ColId) {
        let update = TableUpdate::DelCol { col: id };
        let data = TableRecord { update };
        self.tracer.send(data, None);
    }

    /// Adds a new row
    pub fn add_row(&self, id: RowId, alias: Option<String>) {
        let update = TableUpdate::AddRow { row: id, alias };
        let data = TableRecord { update };
        self.tracer.send(data, None);
    }

    /// Deletes a row by id
    pub fn del_row(&self, id: RowId) {
        let update = TableUpdate::DelRow { row: id };
        let data = TableRecord { update };
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
        let update = TableUpdate::SetCell {
            row,
            col,
            value: value.to_string(),
        };
        let data = TableRecord { update };
        self.tracer.send(data, timestamp);
    }
}

#[derive(Debug)]
pub struct TableRecord {
    update: TableUpdate,
}

impl TracerEvent for TableRecord {
    type State = TableState;
}

#[derive(Debug)]
struct ColRecord {
    alias: Option<String>,
}

#[derive(Debug)]
struct RowRecord {
    alias: Option<String>,
    cols: HashMap<ColId, String>,
}

#[derive(Debug, Default)]
pub struct TableState {
    columns: HashMap<ColId, ColRecord>,
    rows: HashMap<RowId, RowRecord>,
}

impl TracerState for TableState {
    type Item = TableRecord;

    fn aggregate(
        &mut self,
        items: Vec<DataEnvelope<Self::Item>>,
        mut outgoing: Option<&mut Vec<RillEvent>>,
    ) {
        for item in items {
            let DataEnvelope::Event {
                timestamp: ts,
                data,
            } = item;
            match &data.update {
                TableUpdate::AddCol { col, alias } => {
                    let record = ColRecord {
                        alias: alias.clone(),
                    };
                    if let Some(prev) = self.columns.insert(*col, record) {
                        log::warn!("Column {:?} overriden. It was: {:?}.", col, prev);
                    }
                }
                TableUpdate::DelCol { col } => {
                    if self.columns.remove(col).is_none() {
                        log::warn!("Column {:?} was not defnined to remove it.", col);
                    }
                }
                TableUpdate::AddRow { row, alias } => {
                    let record = RowRecord {
                        alias: alias.clone(),
                        cols: HashMap::new(),
                    };
                    if let Some(prev) = self.rows.insert(*row, record) {
                        log::warn!("Row {:?} overriden. It was: {:?}.", row, prev);
                    }
                }
                TableUpdate::DelRow { row } => {
                    if self.rows.remove(row).is_none() {
                        log::warn!("Row {:?} was not defnined to remove it.", row);
                    }
                }
                TableUpdate::SetCell { row, col, value } => {
                    if let Some(record) = self.rows.get_mut(row) {
                        if let Some(cell) = record.cols.get_mut(col) {
                            *cell = value.clone();
                        } else {
                            log::warn!("Column {:?} has not defined yet.", col);
                        }
                    } else {
                        log::warn!("Row {:?} has not defined yet.", row);
                    }
                }
            }
            if let Some(outgoing) = outgoing.as_mut() {
                let data = RillData::TableUpdate(data.update);
                let event = RillEvent {
                    timestamp: ts,
                    data,
                };
                outgoing.push(event);
            }
        }
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        let mut events = Vec::new();
        for (col_id, col_record) in self.columns.iter() {
            let update = TableUpdate::AddCol {
                col: *col_id,
                alias: col_record.alias.clone(),
            };
        }
        events
    }
}
