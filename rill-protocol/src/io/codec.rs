use anyhow::Error;
use meio_protocol::{ProtocolCodec, ProtocolData};

pub struct RRCodec;

impl ProtocolCodec for RRCodec {
    fn decode<T: ProtocolData>(data: &[u8]) -> Result<T, Error> {
        flexbuffers::from_slice(data).map_err(Error::from)
    }

    fn encode<T: ProtocolData>(value: &T) -> Result<Vec<u8>, Error> {
        flexbuffers::to_vec(value).map_err(Error::from)
    }
}
