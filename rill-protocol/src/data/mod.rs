use crate::io::provider::{StreamDelta, StreamState, Timestamp};
use serde::{Deserialize, Serialize};

pub trait State: Into<StreamState> + Clone + Default + Send + 'static {
    type Delta: Delta;

    fn apply(&mut self, update: Self::Delta);
}

pub trait Delta: Into<StreamDelta> + Clone {
    type Event: Event;

    fn produce(event: TimedEvent<Self::Event>) -> Self;
    fn combine(&mut self, event: TimedEvent<Self::Event>);
}

pub trait Event: Send + 'static {
    type State: State<Delta = Self::Delta>;
    type Delta: Delta<Event = Self>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedEvent<T> {
    pub timestamp: Timestamp,
    pub event: T,
}

pub mod counter {
    use super::{Delta, Event, State, TimedEvent};
    use crate::io::provider::Timestamp;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterState {
        timestamp: Option<Timestamp>,
        value: f64,
    }

    impl Default for CounterState {
        fn default() -> Self {
            Self {
                timestamp: None,
                value: 0.0,
            }
        }
    }

    impl State for CounterState {
        type Delta = CounterDelta;

        fn apply(&mut self, delta: Self::Delta) {
            self.timestamp = Some(delta.timestamp);
            self.value += delta.delta;
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CounterDelta {
        timestamp: Timestamp,
        delta: f64,
    }

    impl Delta for CounterDelta {
        type Event = CounterEvent;

        fn produce(event: TimedEvent<Self::Event>) -> Self {
            let delta;
            match event.event {
                CounterEvent::Increment(value) => {
                    delta = value;
                }
            }
            Self {
                timestamp: event.timestamp,
                delta,
            }
        }

        fn combine(&mut self, event: TimedEvent<Self::Event>) {
            self.timestamp = event.timestamp;
            match event.event {
                CounterEvent::Increment(value) => {
                    self.delta += value;
                }
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CounterEvent {
        Increment(f64),
    }

    impl Event for CounterEvent {
        type State = CounterState;
        type Delta = CounterDelta;
    }
}

pub mod gauge {
    use super::{Delta, Event, State, TimedEvent, Timestamp};
    use crate::frame::Frame;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GaugePoint {
        timestamp: Timestamp,
        value: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GaugeState {
        frame: Frame<GaugePoint>,
        value: f64,
    }

    impl Default for GaugeState {
        fn default() -> Self {
            Self {
                frame: Frame::new(30),
                value: 0.0,
            }
        }
    }

    impl State for GaugeState {
        type Delta = GaugeDelta;

        fn apply(&mut self, delta: Self::Delta) {
            for event in delta.events {
                match event.event {
                    GaugeEvent::Increment(delta) => {
                        self.value += delta;
                    }
                    GaugeEvent::Decrement(delta) => {
                        self.value -= delta;
                    }
                    GaugeEvent::Set(value) => {
                        self.value = value;
                    }
                }
                let point = GaugePoint {
                    timestamp: event.timestamp,
                    value: self.value,
                };
                self.frame.insert(point);
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GaugeDelta {
        events: Vec<TimedEvent<GaugeEvent>>,
    }

    impl Delta for GaugeDelta {
        type Event = GaugeEvent;

        fn produce(event: TimedEvent<Self::Event>) -> Self {
            Self {
                events: vec![event],
            }
        }

        fn combine(&mut self, event: TimedEvent<Self::Event>) {
            self.events.push(event);
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum GaugeEvent {
        Increment(f64),
        Decrement(f64),
        Set(f64),
    }

    impl Event for GaugeEvent {
        type State = GaugeState;
        type Delta = GaugeDelta;
    }
}

pub mod dict {
    use super::{Delta, Event, State, TimedEvent};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DictState {
        map: HashMap<String, String>,
    }

    impl Default for DictState {
        fn default() -> Self {
            Self {
                map: HashMap::new(),
            }
        }
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

        fn produce(event: TimedEvent<Self::Event>) -> Self {
            let mut this = Self {
                map: HashMap::new(),
            };
            this.combine(event);
            this
        }

        fn combine(&mut self, event: TimedEvent<Self::Event>) {
            match event.event {
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

    impl Event for DictEvent {
        type State = DictState;
        type Delta = DictDelta;
    }
}

pub mod table {
    use super::{Delta, Event, State, TimedEvent};
    use crate::io::provider::{ColId, RowId};
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TableState {
        columns: BTreeMap<ColId, ColRecord>,
        rows: BTreeMap<RowId, RowRecord>,
    }

    impl Default for TableState {
        fn default() -> Self {
            Self {
                columns: BTreeMap::new(),
                rows: BTreeMap::new(),
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

pub mod logger {
    use super::{Delta, Event, State, TimedEvent};
    use crate::frame::Frame;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LogState {
        frame: Frame<TimedEvent<LogEvent>>,
    }

    impl Default for LogState {
        fn default() -> Self {
            Self {
                frame: Frame::new(30),
            }
        }
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
        events: Vec<TimedEvent<LogEvent>>,
    }

    impl Delta for LogDelta {
        type Event = LogEvent;

        fn produce(event: TimedEvent<Self::Event>) -> Self {
            Self {
                events: vec![event],
            }
        }

        fn combine(&mut self, event: TimedEvent<Self::Event>) {
            self.events.push(event);
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LogEvent {
        // TODO: Replace with enum
        pub msg: String,
    }

    impl Event for LogEvent {
        type State = LogState;
        type Delta = LogDelta;
    }
}
