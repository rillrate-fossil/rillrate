use super::{Metric, TimedEvent};
use crate::io::codec::vectorize;
use crate::io::provider::{ColId, RowId, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TableMetric;

impl Metric for TableMetric {
    type State = TableState;
    type Event = TableEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.table.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            TableEvent::AddCol { col, alias } => {
                let record = ColRecord { alias };
                state.columns.insert(col, record);
            }
            TableEvent::DelCol { col } => {
                state.columns.remove(&col);
                for (_row, record) in state.rows.iter_mut() {
                    record.cols.remove(&col);
                }
            }
            TableEvent::AddRow { row, alias } => {
                let record = RowRecord {
                    alias,
                    cols: BTreeMap::new(),
                };
                state.rows.insert(row, record);
            }
            TableEvent::DelRow { row } => {
                state.rows.remove(&row);
            }
            TableEvent::SetCell { row, col, value } => {
                if let Some(record) = state.rows.get_mut(&row) {
                    if state.columns.contains_key(&col) {
                        record.cols.insert(col, value);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableState {
    #[serde(with = "vectorize")]
    pub columns: BTreeMap<ColId, ColRecord>,
    #[serde(with = "vectorize")]
    pub rows: BTreeMap<RowId, RowRecord>,
}

#[allow(clippy::new_without_default)]
impl TableState {
    pub fn new() -> Self {
        Self {
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
        }
    }
}

pub type TableDelta = Vec<TimedEvent<TableEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableEvent {
    AddCol {
        col: ColId,
        alias: Option<String>,
    },
    DelCol {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColRecord {
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowRecord {
    pub alias: Option<String>,
    #[serde(with = "vectorize")]
    pub cols: BTreeMap<ColId, String>,
}
