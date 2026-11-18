//! The kernel message protocol (H9.1). These types are the single source:
//! TypeScript types are generated from them (`packages/contracts/src/generated`)
//! by `cargo test -p sse-kernel` via ts-rs. Never hand-write them in TS.
//!
//! Wire form is JSON text at the worker boundary (ADR 0003). Requests carry
//! `"op"`, responses carry `"type"`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// Cost class of an operation in deterministic work units (build plan 2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Cost {
    Trivial,
    Light,
    Medium,
    Heavy,
}

impl From<sse_flow::CostClass> for Cost {
    fn from(c: sse_flow::CostClass) -> Self {
        match c {
            sse_flow::CostClass::Trivial => Cost::Trivial,
            sse_flow::CostClass::Light => Cost::Light,
            sse_flow::CostClass::Medium => Cost::Medium,
            sse_flow::CostClass::Heavy => Cost::Heavy,
        }
    }
}

/// Capability tiers. A device measures its speed once in work units per
/// second (the client times the `benchmark` job; the kernel never reads a
/// clock) and gets the highest tier whose minimum it meets.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TierDef {
    pub tier: String,
    pub min_units_per_second: f64,
    /// Highest cost class this tier runs locally. Above it: ask the GM machine
    /// for recorded results when connected, or offer a reduced form.
    pub runs_up_to: Cost,
}

pub fn tiers() -> Vec<TierDef> {
    vec![
        TierDef {
            tier: "basic".into(),
            min_units_per_second: 0.0,
            runs_up_to: Cost::Light,
        },
        TierDef {
            tier: "standard".into(),
            min_units_per_second: 2.0e5,
            runs_up_to: Cost::Medium,
        },
        TierDef {
            tier: "capable".into(),
            min_units_per_second: 2.0e6,
            runs_up_to: Cost::Heavy,
        },
    ]
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OpInfo {
    pub op: String,
    pub cost: Cost,
    /// What a device below the cost class gets instead.
    pub edge: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InputSpec {
    pub value: f64,
    /// Unit id (see Units::standard and packs); SI if absent.
    #[serde(default)]
    #[ts(optional)]
    pub unit: Option<String>,
    /// catalogued | resolved | derived | authored | authored-impossible. Default authored.
    #[serde(default)]
    #[ts(optional)]
    pub source: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthoredEdit {
    pub address: String,
    pub field: String,
    /// Any JSON value without NaN.
    #[ts(type = "unknown")]
    pub value: serde_json::Value,
    /// Universe time of the edit, seconds from the origin.
    pub occurred_at_s: i64,
    /// GM marks an edit as physically impossible (allowed, tagged).
    #[serde(default)]
    pub impossible: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum JobSpec {
    /// Deterministic benchmark: `units` Kepler solves. Used for capability tiers.
    Benchmark { units: u32 },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "op", rename_all = "snake_case")]
#[ts(export)]
pub enum Request {
    Hello,
    /// Packs bottom to top, as `sse.pack/1` documents.
    LoadStack {
        #[ts(type = "unknown[]")]
        packs: Vec<serde_json::Value>,
    },
    OpenUniverse {
        id: String,
        seed: String,
    },
    SetAuthored {
        edit: AuthoredEdit,
    },
    History {
        address: String,
    },
    StateHash {
        #[serde(default)]
        #[ts(optional)]
        scope: Option<String>,
        at_s: i64,
    },
    Derive {
        flow: String,
        inputs: BTreeMap<String, InputSpec>,
    },
    Explain {
        flow: String,
        inputs: BTreeMap<String, InputSpec>,
        kind: String,
    },
    Draw {
        address: String,
        stream: String,
        counter: u32,
    },
    Branch {
        label: String,
    },
    BranchSet {
        branch: String,
        edit: AuthoredEdit,
    },
    BranchStateHash {
        branch: String,
        at_s: i64,
    },
    Commit {
        branch: String,
        at_s: i64,
    },
    Discard {
        branch: String,
    },
    StartJob {
        job: JobSpec,
    },
    /// Run up to `budget` work units of a job.
    StepJob {
        job: u32,
        budget: u32,
    },
    CancelJob {
        job: u32,
    },
}

impl Request {
    pub fn op(&self) -> &'static str {
        match self {
            Request::Hello => "hello",
            Request::LoadStack { .. } => "load_stack",
            Request::OpenUniverse { .. } => "open_universe",
            Request::SetAuthored { .. } => "set_authored",
            Request::History { .. } => "history",
            Request::StateHash { .. } => "state_hash",
            Request::Derive { .. } => "derive",
            Request::Explain { .. } => "explain",
            Request::Draw { .. } => "draw",
            Request::Branch { .. } => "branch",
            Request::BranchSet { .. } => "branch_set",
            Request::BranchStateHash { .. } => "branch_state_hash",
            Request::Commit { .. } => "commit",
            Request::Discard { .. } => "discard",
            Request::StartJob { .. } => "start_job",
            Request::StepJob { .. } => "step_job",
            Request::CancelJob { .. } => "cancel_job",
        }
    }
}

/// Every op with its cost class and edge behaviour.
pub fn ops() -> Vec<OpInfo> {
    let o = |op: &str, cost: Cost, edge: &str| OpInfo {
        op: op.into(),
        cost,
        edge: edge.into(),
    };
    let none = "runs on every device";
    vec![
        o("hello", Cost::Trivial, none),
        o("load_stack", Cost::Light, none),
        o("open_universe", Cost::Trivial, none),
        o(
            "set_authored",
            Cost::Trivial,
            "GM machine only in a session; players send intents",
        ),
        o("history", Cost::Trivial, none),
        o("state_hash", Cost::Light, none),
        o(
            "derive",
            Cost::Light,
            "per flow: the flow's highest atom cost class applies",
        ),
        o("explain", Cost::Light, none),
        o("draw", Cost::Trivial, none),
        o("branch", Cost::Trivial, none),
        o("branch_set", Cost::Trivial, none),
        o("branch_state_hash", Cost::Light, none),
        o("commit", Cost::Trivial, "GM machine only in a session"),
        o("discard", Cost::Trivial, none),
        o(
            "start_job",
            Cost::Trivial,
            "the job itself has a cost class by size",
        ),
        o(
            "step_job",
            Cost::Light,
            "budget bounds the work per call, so a step never blocks for long",
        ),
        o("cancel_job", Cost::Trivial, none),
    ]
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DerivedValue {
    /// SI value.
    pub value: f64,
    /// SI dimension as text, e.g. "kg s^-3".
    pub dim: String,
    pub validity: String,
    /// Depends on fictional physics (a pack flagged fictional, H7).
    pub fictional: bool,
    pub publisher: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EventSummary {
    pub id: String,
    pub kind: String,
    pub occurred_at_s: i64,
    pub evidence: String,
    pub subject: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum Response {
    Hello {
        kernel_version: String,
        protocol: String,
        harness_version: String,
        draw_algorithm: String,
        ops: Vec<OpInfo>,
        tiers: Vec<TierDef>,
    },
    StackLoaded {
        stack_hash: String,
        flows: Vec<String>,
        publishers: u32,
        fictional_kinds: Vec<String>,
    },
    StackRejected {
        report: String,
        issues: Vec<String>,
    },
    UniverseOpened {
        root: String,
        manifest_id: String,
    },
    Authored {
        event_id: String,
        affected: Vec<String>,
        state_hash: String,
    },
    History {
        events: Vec<EventSummary>,
    },
    StateHash {
        hash: String,
    },
    Derived {
        values: BTreeMap<String, DerivedValue>,
        work_units: u32,
    },
    Explanation {
        #[ts(type = "unknown")]
        explanation: serde_json::Value,
        fictional: bool,
    },
    Drawn {
        /// The raw u64 as 16 hex digits (JSON numbers cannot hold it exactly).
        hex: String,
        unit: f64,
    },
    BranchOpened {
        branch: String,
    },
    Committed {
        event_id: String,
        state_hash: String,
    },
    Discarded {
        branch: String,
    },
    JobStarted {
        job: u32,
        total_units: u32,
        cost: Cost,
    },
    JobProgress {
        job: u32,
        done_units: u32,
        total_units: u32,
    },
    JobDone {
        job: u32,
        done_units: u32,
        #[ts(type = "unknown")]
        result: serde_json::Value,
    },
    JobCancelled {
        job: u32,
    },
    Error {
        code: String,
        message: String,
    },
}

pub const PROTOCOL: &str = "sse.kernel/1";
