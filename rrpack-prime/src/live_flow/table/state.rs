use crate::manifest::description::{Layer, PackFlow};
use derive_more::{From, Into};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSpec {
    #[serde(with = "vectorize")]
    pub columns: BTreeMap<Col, ColRecord>,
}

/// Id of a column in a table.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, From, Into, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Col(pub u64);

impl fmt::Display for Col {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<usize> for Col {
    type Error = <u64 as TryFrom<usize>>::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

/// Id of a row in a table.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, From, Into, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Row(pub u64);

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<usize> for Row {
    type Error = <u64 as TryFrom<usize>>::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableState {
    pub spec: TableSpec,
    // MUTABLE:
    #[serde(with = "vectorize")]
    pub rows: BTreeMap<Row, RowRecord>,
}

impl From<TableSpec> for TableState {
    fn from(spec: TableSpec) -> Self {
        Self {
            spec,
            rows: BTreeMap::new(),
        }
    }
}

impl PackFlow for TableState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for TableState {
    type Action = ();
    type Event = TableEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            TableEvent::AddRow { row } => {
                let record = RowRecord {
                    cols: BTreeMap::new(),
                };
                self.rows.insert(row, record);
            }
            TableEvent::DelRow { row } => {
                self.rows.remove(&row);
            }
            TableEvent::SetCell { row, col, value } => {
                if let Some(record) = self.rows.get_mut(&row) {
                    if self.spec.columns.contains_key(&col) {
                        record.cols.insert(col, value);
                    }
                }
            }
        }
    }
}

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
