use anyhow::Error;
use derive_more::{From, FromStr};
use meio_connect::{Protocol, ProtocolCodec, ProtocolData};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;

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

/// An identifier in a hierarchy of the node/metadata/stream.
#[derive(Serialize, Deserialize, FromStr, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(String);

impl AsRef<str> for EntryId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for EntryId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EntryId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for EntryId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, From, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(Vec<EntryId>);

impl ToString for Path {
    fn to_string(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RillToServer {
    DeclareStreams(HashMap<Path, StreamId>),
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
