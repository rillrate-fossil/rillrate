use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillEvent, StreamType};

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
}

#[derive(Debug)]
pub enum TableRecord {}

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
