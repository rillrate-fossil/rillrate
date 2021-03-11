use super::{ConvertError, Delta, State, TimedEvent};
use crate::io::provider::{ColId, RowId, StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

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

impl State for TableState {
    type Event = TableEvent;

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            TableEvent::AddCol { col, alias } => {
                let record = ColRecord { alias };
                self.columns.insert(col, record);
            }
            TableEvent::DelCol { col } => {
                self.columns.remove(&col);
                for (_row, record) in self.rows.iter_mut() {
                    record.cols.remove(&col);
                }
            }
            TableEvent::AddRow { row, alias } => {
                let record = RowRecord {
                    alias,
                    cols: BTreeMap::new(),
                };
                self.rows.insert(row, record);
            }
            TableEvent::DelRow { row } => {
                self.rows.remove(&row);
            }
            TableEvent::SetCell { row, col, value } => {
                if let Some(record) = self.rows.get_mut(&row) {
                    if self.columns.contains_key(&col) {
                        record.cols.insert(col, value);
                    }
                }
            }
        }
    }

    fn wrap(events: Delta<Self::Event>) -> StreamDelta {
        StreamDelta::from(events)
    }

    fn try_extract(delta: StreamDelta) -> Result<Delta<Self::Event>, ConvertError> {
        delta.try_into()
    }
}

pub type TableDelta = Vec<TimedEvent<TableEvent>>;

impl TryFrom<StreamDelta> for TableDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Table(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

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
