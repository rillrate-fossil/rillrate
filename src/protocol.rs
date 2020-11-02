use anyhow::Error;
use derive_more::From;
use meio_ws::{Protocol, ProtocolCodec, ProtocolData};
use serde::{Deserialize, Serialize};

pub const PORT: u16 = 1636;

#[derive(Debug)]
pub struct RillServerProtocol;

impl Protocol for RillServerProtocol {
    type ToServer = RillToServer;
    type ToClient = RillToProvider;
    type Codec = JsonCodec;
}

pub struct JsonCodec;

impl ProtocolCodec for JsonCodec {
    fn decode<T: ProtocolData>(data: &[u8]) -> Result<T, Error> {
        serde_json::from_slice(data).map_err(Error::from)
    }

    fn encode<T: ProtocolData>(value: &T) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(value).map_err(Error::from)
    }
}

#[derive(Debug, Clone, From, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(Vec<String>);

impl ToString for Path {
    fn to_string(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RillToServer {
    DeclareStream { stream_id: StreamId, path: Path },
    Data { stream_id: StreamId, data: RillData },
}

pub type Timestamp = i64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RillData {
    LogRecord {
        timestamp: Timestamp,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RillToProvider {
    ControlStream { stream_id: StreamId, active: bool },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StreamId(pub u64);
