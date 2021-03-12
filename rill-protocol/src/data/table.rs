use super::{ConvertError, Metric, TimedEvent};
use crate::io::provider::{ColId, RowId, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct TableMetric;

impl Metric for TableMetric {
    type State = TableState;
    type Event = TableEvent;

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
    pub columns: BTreeMap<ColId, ColRecord>,
    pub rows: BTreeMap<RowId, RowRecord>,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
        }
    }
}

impl TryFrom<StreamState> for TableState {
    type Error = ConvertError;

    fn try_from(state: StreamState) -> Result<Self, ConvertError> {
        match state {
            StreamState::Table(state) => Ok(state),
            _ => Err(ConvertError),
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
    pub cols: BTreeMap<ColId, String>,
}

#[test]
fn test_table_pointer_ord() {
    use std::collections::BTreeSet;
    use TablePointer::*;

    let col_1 = Col(5.into());
    let col_2 = Col(10.into());
    let row_1 = Row(1.into());
    let row_2 = Row(7.into());
    let cell_1_1 = Cell {
        row: 1.into(),
        col: 1.into(),
    };
    let cell_1_2 = Cell {
        row: 1.into(),
        col: 2.into(),
    };
    let cell_2_1 = Cell {
        row: 2.into(),
        col: 1.into(),
    };
    let cell_2_2 = Cell {
        row: 2.into(),
        col: 2.into(),
    };

    let mut set = BTreeSet::new();
    set.insert(cell_1_2);
    set.insert(row_2);
    set.insert(col_1);
    set.insert(cell_2_1);
    set.insert(col_2);
    set.insert(row_1);
    set.insert(cell_1_1);
    set.insert(cell_2_2);

    let expected = vec![
        col_1, col_2, row_1, row_2, cell_1_1, cell_1_2, cell_2_1, cell_2_2,
    ];

    assert_eq!(expected, set.into_iter().collect::<Vec<_>>());
}
