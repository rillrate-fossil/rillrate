use crate::encoding;
use anyhow::Error;
use meio_protocol::{ProtocolCodec, ProtocolData};

pub struct BinaryCodec;

impl ProtocolCodec for BinaryCodec {
    fn decode<T: ProtocolData>(data: &[u8]) -> Result<T, Error> {
        encoding::from_slice(data).map_err(Error::from)
    }

    fn encode<T: ProtocolData>(value: &T) -> Result<Vec<u8>, Error> {
        encoding::to_vec(value).map_err(Error::from)
    }
}
