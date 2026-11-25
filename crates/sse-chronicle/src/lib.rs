//! SSE4 chronicle (H5.4 to H5.6): an append-only causal history, the state
//! folded from it, checkpoints (a cache, never the truth), branches for
//! look-ahead, stores, and the run manifest that identifies a universe.
#![forbid(unsafe_code)]

pub mod event;
pub mod manifest;

pub use event::{
    kinds, Event, EventBody, EventError, Evidence, Payload, Publisher, SessionStamp, UniverseTime,
};
pub use manifest::{PackRef, RunManifest};

use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::hash::Hasher;
use sse_core::{Address, ContentHash};
use std::collections::{BTreeMap, BTreeSet};

/// Facts per address: `state[address][field] = value`.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub facts: BTreeMap<Address, BTreeMap<String, Value>>,
}

impl State {
    pub fn apply(&mut self, e: &Event) {
        for (a, fields) in &e.body.payload.unset {
            if let Some(m) = self.facts.get_mut(a) {
                for f in fields {
                    m.remove(f);
                }
                if m.is_empty() {
                    self.facts.remove(a);
                }
            }
        }
        for (a, fields) in &e.body.payload.set {
            let m = self.facts.entry(a.clone()).or_default();
            for (f, v) in fields {
                m.insert(f.clone(), v.clone());
            }
        }
    }
    pub fn get(&self, a: &Address, field: &str) -> Option<&Value> {
        self.facts.get(a)?.get(field)
    }
    /// SHA-256 of the canonical encoding of all facts.
    pub fn hash(&self) -> ContentHash {
        ContentHash::of(self).expect("state holds only canonical values")
    }
    /// Hash of the facts at and under `scope`.
    pub fn hash_scope(&self, scope: &Address) -> ContentHash {
        let sub = State {
            facts: self
                .facts
                .iter()
                .filter(|(a, _)| scope.contains(a))
                .map(|(a, m)| (a.clone(), m.clone()))
                .collect(),
        };
        sub.hash()
    }
}

/// Fold events (in fold order) whose activation is at or before `at`.
pub fn fold<'a>(events: impl IntoIterator<Item = &'a Event>, at: UniverseTime) -> State {
    let mut sel: Vec<&Event> = events
        .into_iter()
        .filter(|e| e.body.activated_at <= at)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut s = State::default();
    for e in sel {
        s.apply(e);
    }
    s
}

/// A content-hashed snapshot of state at a time, over a prefix of the chronicle.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub at: UniverseTime,
    /// Number of chronicle events this covers.
    pub covers: u64,
    /// Chronicle head hash after those events.
    pub head: ContentHash,
    pub state: State,
    pub state_hash: ContentHash,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChronicleError {
    Event(EventError),
    CheckpointStale,
    UnknownBranch(ContentHash),
}
impl std::fmt::Display for ChronicleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ChronicleError {}
impl From<EventError> for ChronicleError {
    fn from(e: EventError) -> Self {
        ChronicleError::Event(e)
    }
}

/// The append-only history.
#[derive(Clone, Debug, Default)]
pub struct Chronicle {
    events: Vec<Event>,
    ids: BTreeSet<ContentHash>,
    /// heads[i] = hash chain after i events; heads[0] = genesis.
    heads: Vec<ContentHash>,
    branches: BTreeMap<ContentHash, Branch>,
}

fn genesis() -> ContentHash {
    Hasher::new("sse.chronicle.v1/genesis").finish()
}
fn chain(prev: &ContentHash, id: &ContentHash) -> ContentHash {
    let mut h = Hasher::new("sse.chronicle.v1/chain");
    h.update(&prev.0).update(&id.0);
    h.finish()
}

/// A look-ahead: events on top of a fork point of the main chronicle.
#[derive(Clone, Debug)]
pub struct Branch {
    pub id: ContentHash,
    pub label: String,
    pub base: u64,
    pub events: Vec<Event>,
}

impl Chronicle {
    pub fn new() -> Self {
        Chronicle {
            heads: vec![genesis()],
            ..Default::default()
        }
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn len(&self) -> u64 {
        self.events.len() as u64
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    pub fn head(&self) -> ContentHash {
        *self.heads.last().unwrap()
    }
    pub fn contains(&self, id: &ContentHash) -> bool {
        self.ids.contains(id)
    }
    pub fn next_stamp(&self) -> SessionStamp {
        SessionStamp(self.events.last().map_or(0, |e| e.body.recognised_at.0 + 1))
    }

    fn check(
        &self,
        e: &Event,
        extra: &BTreeSet<ContentHash>,
        last: Option<SessionStamp>,
    ) -> Result<(), EventError> {
        if self.ids.contains(&e.id) || extra.contains(&e.id) {
            return Err(EventError::Duplicate(e.id));
        }
        for c in &e.body.causes {
            if !self.ids.contains(c) && !extra.contains(c) {
                return Err(EventError::UnknownCause(*c));
            }
        }
        if let Some(l) = last {
            if e.body.recognised_at <= l {
                return Err(EventError::StampNotIncreasing {
                    last: l,
                    got: e.body.recognised_at,
                });
            }
        }
        Ok(())
    }

    /// Append one event. Causes must already be present; stamps must increase.
    pub fn append(&mut self, e: Event) -> Result<ContentHash, ChronicleError> {
        self.check(
            &e,
            &BTreeSet::new(),
            self.events.last().map(|x| x.body.recognised_at),
        )?;
        let id = e.id;
        self.heads.push(chain(&self.head(), &id));
        self.ids.insert(id);
        self.events.push(e);
        Ok(id)
    }

    /// State at time `at` from the whole chronicle.
    pub fn state_at(&self, at: UniverseTime) -> State {
        fold(&self.events, at)
    }

    pub fn checkpoint(&self, at: UniverseTime) -> Checkpoint {
        let state = self.state_at(at);
        Checkpoint {
            at,
            covers: self.len(),
            head: self.head(),
            state_hash: state.hash(),
            state,
        }
    }

    /// State at `at >= cp.at` from a checkpoint plus later events. Refuses if
    /// the checkpoint is not from this chronicle or if a later event activates
    /// before the checkpoint time (then the checkpoint is stale: rebuild it).
    pub fn state_from(&self, cp: &Checkpoint, at: UniverseTime) -> Result<State, ChronicleError> {
        let covers = cp.covers as usize;
        if covers > self.events.len()
            || self.heads[covers] != cp.head
            || at < cp.at
            || cp.state.hash() != cp.state_hash
        {
            return Err(ChronicleError::CheckpointStale);
        }
        if self.events[covers..]
            .iter()
            .any(|e| e.body.activated_at < cp.at)
        {
            return Err(ChronicleError::CheckpointStale);
        }
        let mut sel: Vec<&Event> = self.events[..covers]
            .iter()
            .filter(|e| e.body.activated_at > cp.at && e.body.activated_at <= at)
            .chain(
                self.events[covers..]
                    .iter()
                    .filter(|e| e.body.activated_at <= at),
            )
            .collect();
        sel.sort_by_key(|e| e.order_key());
        // Events at exactly cp.at from the tail sort after the checkpointed ones
        // (their stamps are later), so applying them on top is the same fold.
        let mut s = cp.state.clone();
        for e in sel {
            s.apply(e);
        }
        Ok(s)
    }

    // ---- branches (H6.3) ----

    /// Open a branch at the current head.
    pub fn branch(&mut self, label: &str) -> ContentHash {
        let mut h = Hasher::new("sse.chronicle.v1/branch");
        h.update(&self.head().0).update(label.as_bytes());
        let id = h.finish();
        self.branches.insert(
            id,
            Branch {
                id,
                label: label.into(),
                base: self.len(),
                events: Vec::new(),
            },
        );
        id
    }

    pub fn branch_append(
        &mut self,
        branch: &ContentHash,
        e: Event,
    ) -> Result<ContentHash, ChronicleError> {
        let b = self
            .branches
            .get(branch)
            .ok_or(ChronicleError::UnknownBranch(*branch))?;
        let extra: BTreeSet<ContentHash> = b.events.iter().map(|x| x.id).collect();
        let last = b
            .events
            .last()
            .or_else(|| {
                b.base
                    .checked_sub(1)
                    .and_then(|i| self.events.get(i as usize))
            })
            .map(|x| x.body.recognised_at);
        // causes may only be in the main prefix the branch was cut from, or in the branch
        let prefix: BTreeSet<ContentHash> = self.events[..b.base as usize]
            .iter()
            .map(|x| x.id)
            .collect();
        for c in &e.body.causes {
            if !prefix.contains(c) && !extra.contains(c) {
                return Err(EventError::UnknownCause(*c).into());
            }
        }
        if extra.contains(&e.id) || prefix.contains(&e.id) {
            return Err(EventError::Duplicate(e.id).into());
        }
        if let Some(l) = last {
            if e.body.recognised_at <= l {
                return Err(EventError::StampNotIncreasing {
                    last: l,
                    got: e.body.recognised_at,
                }
                .into());
            }
        }
        let id = e.id;
        self.branches.get_mut(branch).unwrap().events.push(e);
        Ok(id)
    }

    pub fn branch_state_at(
        &self,
        branch: &ContentHash,
        at: UniverseTime,
    ) -> Result<State, ChronicleError> {
        let b = self
            .branches
            .get(branch)
            .ok_or(ChronicleError::UnknownBranch(*branch))?;
        Ok(fold(
            self.events[..b.base as usize].iter().chain(b.events.iter()),
            at,
        ))
    }

    /// The branch read as a main line: the prefix it was cut from, then its
    /// own events, and no branches. For reads that take a whole chronicle
    /// (a what-if's resolved system); nothing is committed.
    pub fn branch_as_main(&self, branch: &ContentHash) -> Result<Chronicle, ChronicleError> {
        let b = self
            .branches
            .get(branch)
            .ok_or(ChronicleError::UnknownBranch(*branch))?;
        let n = b.base as usize;
        let mut c = Chronicle {
            events: self.events[..n].to_vec(),
            ids: self.events[..n].iter().map(|e| e.id).collect(),
            heads: self.heads[..=n].to_vec(),
            branches: BTreeMap::new(),
        };
        for e in &b.events {
            c.append(e.clone())?;
        }
        Ok(c)
    }

    pub fn branch_info(&self, branch: &ContentHash) -> Option<&Branch> {
        self.branches.get(branch)
    }

    /// Drop a branch. The main chronicle is untouched.
    pub fn discard(&mut self, branch: &ContentHash) -> Result<(), ChronicleError> {
        self.branches
            .remove(branch)
            .map(|_| ())
            .ok_or(ChronicleError::UnknownBranch(*branch))
    }

    /// Commit a branch: its events are re-stamped onto the main chronicle in
    /// order (ids change only by stamp), followed by a `commit.branch` event
    /// whose causes are the committed events. Returns the commit event id.
    pub fn commit(
        &mut self,
        branch: &ContentHash,
        at: UniverseTime,
        subject: Vec<Address>,
        publisher: Publisher,
    ) -> Result<ContentHash, ChronicleError> {
        let b = self
            .branches
            .remove(branch)
            .ok_or(ChronicleError::UnknownBranch(*branch))?;
        let mut remap: BTreeMap<ContentHash, ContentHash> = BTreeMap::new();
        let mut committed = Vec::new();
        for e in b.events {
            let mut body = e.body;
            body.recognised_at = self.next_stamp();
            body.causes = body
                .causes
                .iter()
                .map(|c| *remap.get(c).unwrap_or(c))
                .collect();
            let ne = Event::new(body)?;
            remap.insert(e.id, ne.id);
            committed.push(self.append(ne)?);
        }
        let mut data = BTreeMap::new();
        data.insert("branch".to_string(), Value::Text(b.id.hex()));
        data.insert("label".to_string(), Value::Text(b.label));
        let ev = Event::new(EventBody {
            kind: kinds::COMMIT_BRANCH.into(),
            subject,
            occurred_at: at,
            activated_at: at,
            recognised_at: self.next_stamp(),
            causes: committed,
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Authored,
            publisher,
        })?;
        self.append(ev)
    }

    // ---- stores (H5.5) ----

    /// The authored store: the GM's edits.
    pub fn authored(&self) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(|e| {
            matches!(
                e.body.evidence,
                Evidence::Authored | Evidence::AuthoredImpossible
            )
        })
    }
    /// The resolved store: facts fixed by resolving clouds.
    pub fn resolved(&self) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(|e| e.body.kind == kinds::RESOLVE)
    }
    /// Hash chain over the authored events only (for the run manifest).
    pub fn authored_hash(&self) -> ContentHash {
        self.authored().fold(genesis(), |h, e| chain(&h, &e.id))
    }
}

/// Derived cache: flow outputs keyed by address, quantity kind and the hash of
/// their inputs. Always discardable.
#[derive(Clone, Debug, Default)]
pub struct DerivedCache {
    entries: BTreeMap<(Address, String), (ContentHash, Value)>,
}
impl DerivedCache {
    pub fn get(&self, a: &Address, kind: &str, inputs: &ContentHash) -> Option<&Value> {
        self.entries
            .get(&(a.clone(), kind.to_string()))
            .filter(|(h, _)| h == inputs)
            .map(|(_, v)| v)
    }
    pub fn put(&mut self, a: Address, kind: &str, inputs: ContentHash, v: Value) {
        self.entries.insert((a, kind.to_string()), (inputs, v));
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests;
