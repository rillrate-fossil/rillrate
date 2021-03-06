use crate::codec::JsonCodec;
use crate::transport::{DirectId, Envelope, Origin, WideEnvelope};
use derive_more::{Deref, From, FromStr, Index, Into};
use meio_protocol::Protocol;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;

pub type ProviderReqId = DirectId<ProviderProtocol>;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathPattern {
    pub path: Path,
}

impl<'de> Deserialize<'de> for PathPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let path: Path = FromStr::from_str(&s).map_err(de::Error::custom)?;
        Ok(PathPattern { path })
    }
}

impl Into<Path> for PathPattern {
    fn into(self) -> Path {
        self.path
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
    pub fn single(entry_id: impl Into<EntryId>) -> Self {
        Self(vec![entry_id.into()])
    }

    pub fn root() -> Self {
        Self(Vec::new())
    }

    pub fn add_root(&self, entry_id: &EntryId) -> Path {
        std::iter::once(entry_id.clone())
            .chain(self.0.iter().cloned())
            .collect::<Vec<_>>()
            .into()
    }

    pub fn concat(&self, entry_id: impl Into<EntryId>) -> Path {
        let mut cloned = self.clone();
        cloned.0.push(entry_id.into());
        cloned
    }

    /*
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
        self.0[drop_left..].to_vec().into()
    }
    */

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

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = false;
        for entry in self.0.iter() {
            if prefix {
                ".".fmt(f)?;
            } else {
                prefix = true;
            }
            entry.fmt(f)?;
        }
        Ok(())
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

// `i64` used, becuase it's widely supported as UTC timestamp
// and for example it's used as timestamp value in BSON format.
#[derive(
    Serialize, Deserialize, From, Into, Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Timestamp(pub i64);

impl From<Duration> for Timestamp {
    fn from(duration: Duration) -> Self {
        // TODO: Use `try_into` here?
        Self(duration.as_millis() as i64)
    }
}

// TODO: Change to `Into` when possible.
// When `from_millis(i64)` will be supported.
impl TryInto<Duration> for Timestamp {
    type Error = std::num::TryFromIntError;

    fn try_into(self) -> Result<Duration, Self::Error> {
        self.0.try_into().map(Duration::from_millis)
    }
}

impl Timestamp {
    // TODO: Maybe just impl `ToPrimitive`?
    pub fn to_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn as_secs(&self) -> i64 {
        self.0 / 1_000
    }

    pub fn as_millis(&self) -> i64 {
        self.0
    }
}

// TODO: Rename to `ProviderProtocol`
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProviderProtocol;

impl Protocol for ProviderProtocol {
    type ToServer = WideEnvelope<Self, ProviderToServer>;
    type ToClient = Envelope<Self, ServerToProvider>;
    type Codec = JsonCodec;
}

impl Origin for ProviderProtocol {}

/* ? TODO: Remove
pub type ServerRequest = Envelope<ProviderProtocol, ServerToProvider>;

pub type ProviderResponse = WideEnvelope<ProviderProtocol, ProviderToServer>;
*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RillData {
    /// Use empty strings if value is not provided.
    ///
    /// For `module` and `level` use `Path`s hierarchy.
    // TODO: Fix names...
    LogRecord {
        message: String,
    },
    CounterRecord {
        value: f64,
    },
    GaugeValue {
        value: f64,
    },
    // TODO: Join with aggregated
    DictUpdate(DictUpdate),
    EntryUpdate(EntryUpdate),
    TableUpdate(TableUpdate),
}

// TODO: Rename to `DictDelta`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DictUpdate {
    // TODO: Use `DictAction {Add, Del}` as a value
    pub map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryUpdate {
    Add { name: EntryId },
    Remove { name: EntryId },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColId(pub u64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TableUpdate {
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

#[derive(Debug, Error)]
pub enum RillDataError {
    #[error("can't prase float: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unapplicable: {0}")]
    Unapplicable(&'static str),
}

/// This convertion used by exporters, because most of them support
/// gauge/counter types only.
impl TryInto<f64> for RillData {
    type Error = RillDataError;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Self::LogRecord { message } => message.parse().map_err(RillDataError::from),
            Self::CounterRecord { value } => Ok(value),
            Self::GaugeValue { value } => Ok(value),
            // TODO: Add extracting rules to pattern/exporter/config
            Self::DictUpdate(_) => Err(RillDataError::Unapplicable(
                "can't convert dict into a single value",
            )),
            Self::EntryUpdate(_) => Err(RillDataError::Unapplicable(
                "can't convert entry into a single value",
            )),
            Self::TableUpdate(_) => Err(RillDataError::Unapplicable(
                "can't convert table into a single value",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RillEvent {
    pub timestamp: Timestamp,
    pub data: RillData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToProvider {
    ListOf {
        path: Path,
    },
    /// Turns on notifications about every added path
    Describe {
        active: bool,
    },
    // TODO: Add `StartStream { path }` and `StopStream`,
    // because the `Path` is not needed to stop the stream.
    ControlStream {
        path: Path,
        active: bool,
    },
    GetSnapshot {
        path: Path,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntryType {
    Node,
    Container,
    Provider,
    Stream(StreamType),
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Node => "node",
            Self::Container => "container",
            Self::Provider => "provider",
            Self::Stream(stream_type) => {
                return write!(f, "stream/{}", stream_type);
            }
        };
        value.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StreamType {
    LogStream,
    CounterStream,
    GaugeStream,
    DictStream,
    TableStream,
    EntryStream,
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::LogStream => "log",
            Self::CounterStream => "counter",
            Self::GaugeStream => "gauge",
            Self::DictStream => "dict",
            Self::TableStream => "table",
            Self::EntryStream => "entry",
        };
        value.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub path: Path,
    pub info: String,
    pub stream_type: StreamType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderToServer {
    Declare {
        entry_id: EntryId,
    },
    Description {
        list: Vec<Description>,
    },
    // TODO: Consider renaming to ListReady
    Entries {
        entries: HashMap<EntryId, EntryType>,
    },
    // TODO: Join it with `BeginStream`?
    SnapshotReady {
        snapshot: Option<RillEvent>,
    },
    /// The response to `ControlStream { active: true }` request
    BeginStream {
        /// Reproduced events.
        snapshot: Vec<RillEvent>,
    },
    Data {
        /// Aggregated events.
        batch: Vec<RillEvent>,
    },
    EndStream,
    Error {
        reason: String,
    },
}
