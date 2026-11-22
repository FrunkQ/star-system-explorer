//! Events (H5.4).

use serde::{Deserialize, Serialize};
use sse_core::cbor::{CborError, Value};
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;
use std::fmt;

/// Universe time: signed seconds (plus nanoseconds) from the universe's origin.
/// Integer, so ordering is exact; 2^63 s is about 290 billion years.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
pub struct UniverseTime {
    pub s: i64,
    #[serde(default)]
    pub ns: u32,
}

impl UniverseTime {
    pub const ORIGIN: UniverseTime = UniverseTime { s: 0, ns: 0 };
    pub fn seconds(s: i64) -> Self {
        UniverseTime { s, ns: 0 }
    }
    /// Whole Julian years (365.25 days) from the origin.
    pub fn years(y: i64) -> Self {
        UniverseTime {
            s: y * (sse_core::sheet::YR_JULIAN as i64),
            ns: 0,
        }
    }
}

/// When an event entered this chronicle. Ordering only; never physics.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
pub struct SessionStamp(pub u64);

/// How a fact came to be (H5.4). Ordered from most to least grounded.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Evidence {
    Catalogued,
    Derived,
    Resolved,
    Authored,
    AuthoredImpossible,
    Narrative,
}

/// Who published an event: pack, flow or atom, and version.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Publisher {
    pub pack: String,
    pub item: String,
    pub version: String,
}

impl Publisher {
    pub fn new(pack: &str, item: &str, version: &str) -> Self {
        Publisher {
            pack: pack.into(),
            item: item.into(),
            version: version.into(),
        }
    }
}

/// Built-in event kinds. Suites may add their own (`suite.<name>.<kind>`).
pub mod kinds {
    pub const RESOLVE: &str = "resolve";
    pub const AUTHORED_SET: &str = "authored.set";
    pub const AUTHORED_EVENT: &str = "authored.event";
    pub const DERIVE_MILESTONE: &str = "derive.milestone";
    pub const EVOLVE_TRANSITION: &str = "evolve.transition";
    pub const COMMIT_BRANCH: &str = "commit.branch";
    pub const OBSERVE_GRANT: &str = "observe.grant";
}

/// The content of an event. Its id is the hash of this.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventBody {
    pub kind: String,
    pub subject: Vec<Address>,
    pub occurred_at: UniverseTime,
    pub activated_at: UniverseTime,
    pub recognised_at: SessionStamp,
    pub causes: Vec<ContentHash>,
    pub payload: Payload,
    pub evidence: Evidence,
    pub publisher: Publisher,
}

/// Fact changes an event makes, plus free data for suites.
///
/// `set[address][field] = value` and `unset[address] = [fields]` are the only
/// parts the generic fold reads. `data` is carried and hashed but has no
/// effect on state unless a suite folds it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Payload {
    #[serde(default)]
    pub set: BTreeMap<Address, BTreeMap<String, Value>>,
    #[serde(default)]
    pub unset: BTreeMap<Address, Vec<String>>,
    #[serde(default)]
    pub data: Option<Value>,
}

/// An event with its content-hash id.
#[derive(Clone, PartialEq, Debug)]
pub struct Event {
    pub id: ContentHash,
    pub body: EventBody,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventError {
    Encoding(CborError),
    UnknownCause(ContentHash),
    Duplicate(ContentHash),
    StampNotIncreasing {
        last: SessionStamp,
        got: SessionStamp,
    },
    ActivatedBeforeOccurred,
    NoSubject,
    BadKind(String),
}
impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EventError {}

impl Event {
    pub fn new(body: EventBody) -> Result<Self, EventError> {
        if body.subject.is_empty() {
            return Err(EventError::NoSubject);
        }
        if body.activated_at < body.occurred_at {
            return Err(EventError::ActivatedBeforeOccurred);
        }
        if body.kind.is_empty()
            || !body
                .kind
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'_')
        {
            return Err(EventError::BadKind(body.kind));
        }
        let id = ContentHash::of(&body).map_err(EventError::Encoding)?;
        Ok(Event { id, body })
    }
    /// Fold order: activation time, then when it was recognised, then id.
    pub fn order_key(&self) -> (UniverseTime, SessionStamp, ContentHash) {
        (self.body.activated_at, self.body.recognised_at, self.id)
    }
}
