pub trait State {
    type Delta: Delta;

    fn apply(&mut self, update: Self::Delta);
}

pub trait Delta {
    type Event;

    fn combine(&mut self, event: Self::Event);
}

pub mod counter {
    use super::{Delta, State};
    use crate::io::provider::Timestamp;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterState {
        event: CounterEvent,
    }

    impl State for CounterState {
        type Delta = CounterDelta;

        fn apply(&mut self, update: Self::Delta) {
            self.event = update.last_event;
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterDelta {
        last_event: CounterEvent,
    }

    impl Delta for CounterDelta {
        type Event = CounterEvent;

        fn combine(&mut self, event: Self::Event) {
            self.last_event = event;
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterEvent {
        timestamp: Timestamp,
        value: f64,
    }
}

pub mod gauge {
    use super::{Delta, State};
    use crate::frame::Frame;
    use crate::io::provider::Timestamp;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GaugeState {
        frame: Frame<GaugeEvent>,
    }

    impl State for GaugeState {
        type Delta = GaugeDelta;

        fn apply(&mut self, update: Self::Delta) {
            for event in update.events {
                self.frame.insert(event);
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GaugeDelta {
        events: Vec<GaugeEvent>,
    }

    impl Delta for GaugeDelta {
        type Event = GaugeEvent;

        fn combine(&mut self, event: Self::Event) {
            self.events.push(event);
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GaugeEvent {
        timestamp: Timestamp,
        value: f64,
    }
}

pub mod dict {
    use super::{Delta, State};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DictState {
        map: HashMap<String, String>,
    }

    impl State for DictState {
        type Delta = DictDelta;

        fn apply(&mut self, update: Self::Delta) {
            self.map.extend(update.map);
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DictDelta {
        map: HashMap<String, String>,
    }

    impl Delta for DictDelta {
        type Event = DictEvent;

        fn combine(&mut self, event: Self::Event) {
            match event {
                DictEvent::SetValue { key, value } => {
                    self.map.insert(key, value);
                }
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DictEvent {
        SetValue { key: String, value: String },
    }
}

pub mod table {
    use super::{Delta, State};
    use crate::io::provider::{ColId, RowId};
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TableState {
        columns: BTreeMap<ColId, ColRecord>,
        rows: BTreeMap<RowId, RowRecord>,
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
                            if let Some(cell) = record.cols.get_mut(&col) {
                                *cell = value;
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

    impl Delta for TableDelta {
        type Event = TableEvent;

        fn combine(&mut self, event: Self::Event) {
            let pointer;
            let action;
            match event {
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ColRecord {
        alias: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct RowRecord {
        alias: Option<String>,
        cols: BTreeMap<ColId, String>,
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
}

pub mod log {
    use super::{Delta, State};
    use crate::frame::Frame;
    use crate::io::provider::Timestamp;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LogState {
        frame: Frame<LogEvent>,
    }

    impl State for LogState {
        type Delta = LogDelta;

        fn apply(&mut self, update: Self::Delta) {
            for event in update.events {
                self.frame.insert(event);
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LogDelta {
        events: Vec<LogEvent>,
    }

    impl Delta for LogDelta {
        type Event = LogEvent;

        fn combine(&mut self, event: Self::Event) {
            self.events.push(event);
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LogEvent {
        timestamp: Timestamp,
        msg: String,
    }
}
