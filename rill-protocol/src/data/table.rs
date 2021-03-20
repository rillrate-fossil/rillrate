use super::{Metric, TimedEvent};
use crate::io::codec::vectorize;
use crate::io::provider::{Col, Row, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableMetric {
    #[serde(with = "vectorize")]
    pub columns: BTreeMap<Col, ColRecord>,
}

impl Metric for TableMetric {
    type State = TableState;
    type Event = TableEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.table.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            TableEvent::AddRow { row } => {
                let record = RowRecord {
                    cols: BTreeMap::new(),
                };
                state.rows.insert(row, record);
            }
            TableEvent::DelRow { row } => {
                state.rows.remove(&row);
            }
            TableEvent::SetCell { row, col, value } => {
                if let Some(record) = state.rows.get_mut(&row) {
                    if self.columns.contains_key(&col) {
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
    pub rows: BTreeMap<Row, RowRecord>,
}

#[allow(clippy::new_without_default)]
impl TableState {
    pub fn new() -> Self {
        Self {
            rows: BTreeMap::new(),
        }
    }
}

pub type TableDelta = Vec<TimedEvent<TableEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableEvent {
    AddRow { row: Row },
    DelRow { row: Row },
    SetCell { row: Row, col: Col, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColRecord {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowRecord {
    #[serde(with = "vectorize")]
    pub cols: BTreeMap<Col, String>,
}
