use anyhow::Error;
use derive_more::{Deref, From, FromStr, Index, Into};
use meio_connect::{Protocol, ProtocolCodec, ProtocolData};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;
use thiserror::Error;

pub const PORT: u16 = 1636;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    Direct(DirectId),
    Multicast(Vec<DirectId>),
    Broadcast,
}

impl Direction {
    pub fn broadcast() -> Self {
        Self::Broadcast
    }
}

impl From<&HashSet<DirectId>> for Direction {
    fn from(set: &HashSet<DirectId>) -> Self {
        let mut iter = set.iter();
        match iter.len() {
            0 => Self::Broadcast,
            1 => {
                let direct_id = iter.next().cloned().unwrap();
                Self::Direct(direct_id)
            }
            _ => {
                let ids = iter.cloned().collect();
                Self::Multicast(ids)
            }
        }
    }
}

impl From<DirectId> for Direction {
    fn from(direct_id: DirectId) -> Self {
        Self::Direct(direct_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub direct_id: DirectId,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WideEnvelope<T> {
    pub direction: Direction,
    pub data: T,
}

#[derive(Debug)]
pub struct RillProviderProtocol;

impl Protocol for RillProviderProtocol {
    type ToServer = WideEnvelope<RillToServer>;
    type ToClient = Envelope<RillToProvider>;
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

#[derive(
    Debug,
    Clone,
    Deref,
    From,
    Into,
    Index,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
// TODO: Consider to use `type Path = Vec<EntryId>` directly
pub struct Path(Vec<EntryId>);

impl Path {
    pub fn root() -> Self {
        Self(Vec::new())
    }

    pub fn concat(&self, other: &[EntryId]) -> Path {
        self.0
            .iter()
            .chain(other.iter())
            .cloned()
            .collect::<Vec<_>>()
            .into()
    }

    #[deprecated(since = "0.4.0", note = "Use `split` method instead.")]
    pub fn subpath(&self, drop_left: usize) -> Path {
        self.0[drop_left..]
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .into()
    }

    pub fn split(&self) -> (Option<EntryId>, Path) {
        let mut iter = self.0.iter().cloned();
        let entry_id = iter.next();
        let path = Path::from(iter.collect::<Vec<_>>());
        (entry_id, path)
    }
}

impl<'a> FromIterator<&'a EntryId> for Path {
    fn from_iter<I: IntoIterator<Item = &'a EntryId>>(iter: I) -> Self {
        Self(iter.into_iter().cloned().collect())
    }
}

impl AsRef<[EntryId]> for Path {
    fn as_ref(&self) -> &[EntryId] {
        &self.0
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Error, Debug)]
pub enum PathError {
    // Never constructed yet, because paths never fail now.
}

impl FromStr for Path {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entries: Vec<_> = s.split('.').map(EntryId::from).collect();
        Ok(Path::from(entries))
    }
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
    ListOf { path: Path },
    ControlStream { path: Path, active: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RillToServer {
    Declare { entry_id: EntryId },
    // TODO: Consider renaming to ListReady
    Entries { entries: Vec<EntryId> },
    // Snapshot { data: RillData },
    BeginStream,
    Data { data: RillData },
    EndStream,
}

#[derive(Debug, Clone, Copy, From, Into, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DirectId(pub u64);
