use super::{ConvertError, Delta, Event, State, TimedEvent};
use crate::io::provider::{ColId, RowId, StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;

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
    type Delta = TableDelta;

    fn apply(&mut self, delta: Self::Delta) {
        for pair in delta.updates {
            match pair {
                (TablePointer::Col(col), TableAction::Add { alias }) => {
                    let record = ColRecord { alias };
                    self.columns.insert(col, record);
                }
                (TablePointer::Col(col), TableAction::Del) => {
                    self.columns.remove(&col);
                    for (_row, record) in self.rows.iter_mut() {
                        record.cols.remove(&col);
                    }
                }
                (TablePointer::Row(row), TableAction::Add { alias }) => {
                    let record = RowRecord {
                        alias,
                        cols: BTreeMap::new(),
                    };
                    self.rows.insert(row, record);
                }
                (TablePointer::Row(row), TableAction::Del) => {
                    self.rows.remove(&row);
                }
                (TablePointer::Cell { row, col }, TableAction::Set { value }) => {
                    if let Some(record) = self.rows.get_mut(&row) {
                        if self.columns.contains_key(&col) {
                            record.cols.insert(col, value);
                        }
                    }
                }
                (pointer, action) => {
                    log::error!("Incorrect pair of the {:?} and {:?}", pointer, action);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TablePointer {
    Col(ColId),
    Row(RowId),
    Cell { row: RowId, col: ColId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableAction {
    Add { alias: Option<String> },
    Del,
    Set { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDelta {
    updates: BTreeMap<TablePointer, TableAction>,
}

impl TryFrom<StreamDelta> for TableDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Table(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

impl Delta for TableDelta {
    type Event = TableEvent;

    fn produce(event: TimedEvent<Self::Event>) -> Self {
        let mut this = Self {
            updates: BTreeMap::new(),
        };
        this.combine(event);
        this
    }

    fn combine(&mut self, event: TimedEvent<Self::Event>) {
        let pointer;
        let action;
        match event.event {
            TableEvent::AddCol { col, alias } => {
                pointer = TablePointer::Col(col);
                action = TableAction::Add { alias };
            }
            TableEvent::DelCol { col } => {
                pointer = TablePointer::Col(col);
                action = TableAction::Del;
            }
            TableEvent::AddRow { row, alias } => {
                pointer = TablePointer::Row(row);
                action = TableAction::Add { alias };
            }
            TableEvent::DelRow { row } => {
                pointer = TablePointer::Row(row);
                action = TableAction::Del;
            }
            TableEvent::SetCell { row, col, value } => {
                pointer = TablePointer::Cell { row, col };
                action = TableAction::Set { value };
            }
        }
        self.updates.insert(pointer, action);
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

impl Event for TableEvent {
    type State = TableState;
    type Delta = TableDelta;
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
