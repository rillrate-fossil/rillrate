use crate::encoding;
use crate::flow::data::DataFraction;
use crate::io::provider::{PackedEvent, StreamType};
use anyhow::Error;

pub trait Inflow: DataFraction {
    type Event: DataFraction;

    fn stream_type() -> StreamType;

    fn pack_event(event: &Self::Event) -> Result<PackedEvent, Error> {
        encoding::to_vec(event)
            .map_err(Error::from)
            .map(PackedEvent::from)
    }

    fn unpack_event(event: &PackedEvent) -> Result<Self::Event, Error> {
        encoding::from_slice(&event.0).map_err(Error::from)
    }
}
