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
    /// The open universe's run manifest as it stands now (H5.6): the stack
    /// it runs on and the authored chronicle hash.
    Manifest,
    /// A scene (H6.7, H8.1): what `audience` has been shown of everything at
    /// or under `scope` at `at_s`, after the campaign's data views. `gm` is
    /// the GM seat. `data_view` shows every subject through that one view
    /// (a lens or a preview) instead of the grants.
    Scene {
        audience: String,
        scope: String,
        #[serde(default)]
        #[ts(optional)]
        data_view: Option<String>,
        at_s: i64,
        /// A draft `provides.other.observe` setup to build with instead of
        /// the stack's (a preview before Apply). Nothing is recorded.
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        observe: Option<serde_json::Value>,
    },
    /// Describe the registered atoms (ports with quantity kinds, dimensions
    /// and valid ranges, parameters with defaults, ranges and descriptions,
    /// cost class, references, explain text), for editors and Orac. All of
    /// them, or only `ids`.
    Atoms {
        #[serde(default)]
        #[ts(optional)]
        ids: Option<Vec<String>>,
    },
    /// The Scanned button: flip `address` between the campaign's before and
    /// after views for `audience`. Chronicled; pressing again undoes. GM only.
    ScanToggle {
        address: String,
        audience: String,
        at_s: i64,
    },
    /// Grant `audience` the data view `view` of `address` and everything under
    /// it, or with no `view` take the grant back. With `default`, `address` is
    /// the universe root and the view is the audience's starting view. One
    /// `observe.grant` event. GM only.
    Grant {
        address: String,
        audience: String,
        #[serde(default)]
        #[ts(optional)]
        view: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        default: Option<bool>,
        at_s: i64,
    },
    /// Look ahead (T2): age a system to `target_myr` (the star's age, Myr)
    /// on a new chronicle branch and return its fate: dated transitions with
    /// their reasons, and the star and planets at every step. `system` is an
    /// `sse-time` SystemSpec (`tests/fixtures/time/solar-system.json` is one);
    /// its `system` address must be in the open universe. Settings come from
    /// the stack's `provides.other.time` unless `settings` is given. Nothing
    /// touches the main chronicle until `commit_look_ahead`; `discard` and
    /// `branch_state_hash` take the returned branch as for any branch.
    LookAhead {
        #[ts(type = "unknown")]
        system: serde_json::Value,
        target_myr: f64,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        settings: Option<serde_json::Value>,
    },
    /// Commit a look-ahead branch. With `move_now_myr`, the system's "now"
    /// moves to that star age (an authored `system.now_year` fact).
    CommitLookAhead {
        branch: String,
        #[serde(default)]
        #[ts(optional)]
        move_now_myr: Option<f64>,
        /// Also write the aged state as facts under the usual field names
        /// (`orbit.semi_major_axis`, `orbit.eccentricity`, `orbit.inclination`,
        /// `body.engulfed`, the star's `body.mass`, `body.radius`,
        /// `star.effective_temperature`, `star.luminosity`, `star.phase`; SI),
        /// dated at the snapshot at or before `move_now_myr`, else the last.
        #[serde(default)]
        #[ts(optional)]
        write_back: Option<bool>,
    },
    /// Import an SSE 3 save (the text of its `starmap.json` or `system.json`):
    /// the imported universe's facts and the difference report. With
    /// `reconstruct` (candidate histories to search, F9), each known system
    /// also gains a formation history solved around its planets, labelled
    /// reconstructed. Nothing is recorded in an open universe.
    ImportV3 {
        save: String,
        #[serde(default)]
        #[ts(optional)]
        universe_id: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        reconstruct: Option<u32>,
    },
    /// Generate one system through the single formation pipeline (F0 to F6,
    /// heat, F9 pins) and give every world its surface envelope (S0). The
    /// galaxy inheritance is either a cosmos `birth` (G0, read at `now_gyr`)
    /// or a patch `envelope`; neither means the stand-in solar neighbourhood.
    /// Nothing is recorded in an open universe.
    RunFormation {
        seed: String,
        /// Address of the system; default `u:generated/sys:0`.
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        /// `modern` (default), `classic` or `classic-corrected`.
        #[serde(default)]
        #[ts(optional)]
        block: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        birth: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        now_gyr: Option<f64>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        envelope: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        inherit_pins: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        star_pins: Option<serde_json::Value>,
        /// F9 `Pins`: worlds to honour, candidate histories to search.
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        pins: Option<serde_json::Value>,
    },
    /// A body's world stack (`sse.world-stack/1`: shells, regions, features)
    /// with its depth ladder and physics notes. Served from the P4 fixtures
    /// (by `fixture` name or by the fixture's `body` address) until surface
    /// stacks are built from history; a given `stack` is checked instead.
    ReadWorldStack {
        #[serde(default)]
        #[ts(optional)]
        body: Option<String>,
        /// `earth`, `europa`, `mars`, `tarn`, `titan` or `venus`.
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        stack: Option<serde_json::Value>,
    },
    /// Bring a system into the open universe as facts: an SSE 3 save
    /// (`import_v3`, its text) or a formation run (`formation`, the same
    /// fields as `run_formation`; its `system` must be in the universe). The
    /// kernel runs the import or the pipeline itself, so the facts are the
    /// ones those ops return. One event per evidence class (an import mixes
    /// authored, catalogued and authored-impossible facts); the first carries
    /// the report (import) or the request and run summary (formation) and the
    /// others cite it. Dated `occurred_at_s` (default the universe origin).
    Adopt {
        #[serde(default)]
        #[ts(optional)]
        import_v3: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        formation: Option<FormationRequest>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        occurred_at_s: Option<i64>,
    },
    /// One system as resolved now (or at `at_s`): its facts, a `SystemSpec`
    /// for `look_ahead`, an S0 surface envelope per world, and any S1 tiles
    /// asked for (GM machine: down to the fact floor). Formation-adopted
    /// systems use their run; imported or authored ones build the worlds from
    /// facts.
    ResolvedSystem {
        system: String,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        tiles: Option<Vec<TileRequest>>,
    },
    /// S2 to S7 in order for one world: a surface `fixture` (`Earth`,
    /// `Moon`, ...; no universe needed) or a `body` of an adopted `system`
    /// at `at_s`. `options` is sse-surface's `SurveyOptions` (rotation_hours,
    /// obliquity_deg, life, settle, reshape, names, renames, wind_m_s);
    /// `tiles` are S1 tiles of this world with S7's layers; `quick` uses a
    /// coarse plate grid and 32-sample tiles (tests, weak devices). `limit`
    /// caps the features and deposits listed (default 200, 0 for all):
    /// remarkable features first, then the named landscape, then craters by
    /// size; deposits by size. Totals are always given.
    SurveySurface {
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        body: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        options: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        tiles: Option<Vec<SurfaceTile>>,
        #[serde(default)]
        #[ts(optional)]
        quick: Option<bool>,
        #[serde(default)]
        #[ts(optional)]
        limit: Option<u32>,
    },
    /// The open universe's systems: every `<root>/sys:*` with facts, where it
    /// came from (`v3`, `formation` or `facts`), its name and adopt event.
    Systems,
    /// L1d's `LifeView` (biosphere view and life timeline) for a fixture
    /// world (`earth`, `earth-archean`, `europa`, `red-dwarf-desert`,
    /// `dead-earth`), or for a body once adopted systems carry habitat graphs.
    LifeView {
        #[serde(default)]
        #[ts(optional)]
        body: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
    },
    /// The open universe as a saveable log: root, seed text, manifest and
    /// stack ids, and every event body in order with its id.
    ExportChronicle,
    /// Reopen a universe from `export_chronicle`'s output: opens it, replays
    /// every event, checks each id and the head, returns the state hash.
    ImportChronicle {
        #[ts(type = "unknown")]
        export: serde_json::Value,
    },
}

/// `adopt`'s formation request: the `run_formation` fields.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FormationRequest {
    pub seed: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub block: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub birth: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub now_gyr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub envelope: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub inherit_pins: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub star_pins: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub pins: Option<serde_json::Value>,
}

/// One S1 tile of the world `survey_surface` is reading.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SurfaceTile {
    pub face: u8,
    pub level: u8,
    pub x: u32,
    pub y: u32,
}

/// One S1 tile of one world: cube face 0 to 5, level, x and y.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TileRequest {
    pub body: String,
    pub face: u8,
    pub level: u8,
    pub x: u32,
    pub y: u32,
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
            Request::Manifest => "manifest",
            Request::Scene { .. } => "scene",
            Request::ScanToggle { .. } => "scan_toggle",
            Request::Grant { .. } => "grant",
            Request::Atoms { .. } => "atoms",
            Request::LookAhead { .. } => "look_ahead",
            Request::CommitLookAhead { .. } => "commit_look_ahead",
            Request::ImportV3 { .. } => "import_v3",
            Request::RunFormation { .. } => "run_formation",
            Request::ReadWorldStack { .. } => "read_world_stack",
            Request::Adopt { .. } => "adopt",
            Request::ResolvedSystem { .. } => "resolved_system",
            Request::SurveySurface { .. } => "survey_surface",
            Request::Systems => "systems",
            Request::LifeView { .. } => "life_view",
            Request::ExportChronicle => "export_chronicle",
            Request::ImportChronicle { .. } => "import_chronicle",
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
        o("manifest", Cost::Trivial, none),
        o(
            "scene",
            Cost::Light,
            "linear in the facts within the scope; any device builds its own scenes",
        ),
        o("scan_toggle", Cost::Trivial, "GM machine only in a session"),
        o("grant", Cost::Trivial, "GM machine only in a session"),
        o("atoms", Cost::Trivial, none),
        o(
            "look_ahead",
            Cost::Medium,
            "GM edit: a Solar System run to a cold white dwarf is about 0.2 M work units; weak devices ask the GM machine",
        ),
        o("commit_look_ahead", Cost::Trivial, "GM machine only in a session"),
        o(
            "import_v3",
            Cost::Light,
            "linear in the save; with reconstruct, heavy (a formation run per candidate history per system)",
        ),
        o(
            "run_formation",
            Cost::Medium,
            "one formation pipeline run; with pins, one per candidate history",
        ),
        o(
            "read_world_stack",
            Cost::Light,
            "one stack: validation, the depth ladder and a few atom evaluations",
        ),
        o(
            "adopt",
            Cost::Medium,
            "GM machine only in a session; an import or one formation run, then one event per evidence class",
        ),
        o(
            "resolved_system",
            Cost::Medium,
            "a formation re-run for formation-adopted systems (cached), plus about 0.1 s per 256-sample tile; at most 24 tiles",
        ),
        o(
            "survey_surface",
            Cost::Heavy,
            "every surface stage for one world: about 1 to 5 s native at pack defaults (plates, wear, register), well under 1 s with quick; plus about 0.2 s per tile, at most 24",
        ),
        o("systems", Cost::Light, "linear in the universe's facts"),
        o("life_view", Cost::Light, "one fixture's signatures, epochs and hooks"),
        o("export_chronicle", Cost::Light, "linear in the events"),
        o(
            "import_chronicle",
            Cost::Light,
            "linear in the events; replaces the open universe",
        ),
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
        /// Set when a universe is open and the stack changed: the change is
        /// chronicled, so it moves the state hash and replays.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        adopted: Option<StackAdopted>,
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
        /// `commit_look_ahead` with `write_back`: the aged-state event and its star age.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        aged_event: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        aged_at_myr: Option<f64>,
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
    Manifest {
        #[ts(type = "unknown")]
        manifest: serde_json::Value,
        manifest_id: String,
    },
    /// The sse-observe `Scene`, unchanged in shape.
    Scene {
        #[ts(type = "unknown")]
        scene: serde_json::Value,
        scene_hash: String,
    },
    /// One JSON record per atom, in id order (see `Request::Atoms`).
    Atoms {
        #[ts(type = "unknown[]")]
        atoms: Vec<serde_json::Value>,
    },
    ScanToggled {
        event_id: String,
        /// The view now in force for the audience at that address.
        view: String,
        state_hash: String,
    },
    Granted {
        event_id: String,
        /// The view now in force for the audience at that address (grants and
        /// defaults; tag grants show in scenes).
        view: String,
        state_hash: String,
    },
    LookedAhead {
        branch: String,
        label: String,
        /// The system's address, its star's age now and at the end, Myr.
        system: String,
        now_myr: f64,
        target_myr: f64,
        /// Universe year of the star's birth (a star age t is year birth + t x 1e6).
        #[ts(type = "number")]
        birth_year: i64,
        /// Beyond this star age, futures are "one plausible future".
        horizon_myr: f64,
        events: Vec<FateEvent>,
        snapshots: Vec<FateSnapshot>,
        work_units: u32,
    },
    /// `import_v3`: `facts` maps address to field to `{value, evidence}`;
    /// `report` is the importer's difference report; `reconstructed` has one
    /// record per known system when asked (summary plus the solved run).
    V3Imported {
        universe: String,
        #[ts(type = "unknown")]
        facts: serde_json::Value,
        #[ts(type = "unknown")]
        report: serde_json::Value,
        #[ts(type = "unknown[]")]
        reconstructed: Vec<serde_json::Value>,
    },
    /// `run_formation`: the pipeline's `SystemRun`, the F9 fit when pins were
    /// given (`pinned`: attempt, seed, fit, matches, removed), and one S0
    /// surface envelope per world in orbit order.
    FormationRan {
        system: String,
        pipeline: String,
        #[ts(type = "unknown")]
        run: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        pinned: Option<serde_json::Value>,
        #[ts(type = "unknown[]")]
        surfaces: Vec<serde_json::Value>,
    },
    /// `read_world_stack`: `source` is `fixture:<name>` or `given`; `ladder`
    /// is the "above, on and within" strip (`depth`, `text`, `where`, `key`);
    /// `notes` what the atoms computed; `problems` what physics says cannot be
    /// and `invalid` structural faults (each `{at, message}`).
    WorldStackRead {
        body: String,
        source: String,
        #[ts(type = "unknown")]
        stack: serde_json::Value,
        #[ts(type = "unknown[]")]
        ladder: Vec<serde_json::Value>,
        notes: Vec<String>,
        #[ts(type = "unknown[]")]
        problems: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        invalid: Vec<serde_json::Value>,
    },
    /// `adopt`: the events written (the first carries the report), the
    /// systems adopted and the number of facts.
    Adopted {
        events: Vec<String>,
        systems: Vec<String>,
        facts: u32,
        state_hash: String,
    },
    /// `life_view`: `source` is `fixture:<name>` (or the body's address).
    LifeViewed {
        source: String,
        #[ts(type = "unknown")]
        view: serde_json::Value,
    },
    /// `export_chronicle`: a JSON document to save as is
    /// (`format` `sse.chronicle-export/1`).
    ChronicleExported {
        #[ts(type = "unknown")]
        export: serde_json::Value,
    },
    /// `import_chronicle`: the reopened universe. `stack_matches` says whether
    /// the loaded pack stack is the one the export was made on.
    ChronicleImported {
        root: String,
        events: u32,
        head: String,
        state_hash: String,
        manifest_id: String,
        stack_matches: bool,
    },
    /// `survey_surface`: `{body, label, source (fixture:<name>, formation, v3
    /// or facts), version, epoch_myr, at_s?, stages, envelope, deposit_kinds,
    /// climate?, air?, deposits?, deposits_total?, built?, built_kinds?, applied?, register? (with features_total),
    /// aerostat? or aerostat_refused, tiles, refused}`. A giant has no layers
    /// or register.
    SurfaceSurveyed {
        #[ts(type = "unknown")]
        survey: serde_json::Value,
    },
    /// `systems`: one entry per system, `{system, source, name?, addresses,
    /// adopted_event?}`.
    SystemsListed {
        #[ts(type = "unknown[]")]
        systems: Vec<serde_json::Value>,
    },
    /// `resolved_system`. `source`: `formation`, `v3` or `facts`. `facts`:
    /// address to field to value under the system. `evidence`: address to
    /// field to `{evidence, by}`: the label (`authored`, `derived`, ...) and
    /// the kind of the event that set it (`time.aged` for a write-back). `spec`: an `sse-time`
    /// SystemSpec (absent without a star mass and age). `surfaces`: S0
    /// envelopes. `tiles`: S1 tiles; `refused`: tiles not given, with why.
    SystemResolved {
        system: String,
        source: String,
        /// The date read, seconds; absent for "latest" (no date given and no
        /// `system.now_year`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[ts(type = "unknown")]
        facts: serde_json::Value,
        #[ts(type = "unknown")]
        evidence: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        spec: Option<serde_json::Value>,
        #[ts(type = "unknown[]")]
        surfaces: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        tiles: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        refused: Vec<serde_json::Value>,
    },
    Error {
        code: String,
        message: String,
    },
}

/// A named transition ahead (T2 `TimeEvent`), with its reason.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FateEvent {
    /// `oceans_lost`, `leaves_main_sequence`, `red_giant`, `engulfed`, `planetary_nebula`, `cooling_colour`, `pollution` (the base time pack).
    pub kind: String,
    pub age_myr: f64,
    /// Universe year.
    #[ts(type = "number")]
    pub year: i64,
    pub subjects: Vec<String>,
    pub why: String,
    /// Fidelity of the model behind it: `sketch` or `standard`.
    pub level: String,
    pub plausible_future: bool,
    pub numbers: BTreeMap<String, f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FateStar {
    pub phase: String,
    /// Solar units.
    pub luminosity: f64,
    pub radius_rsun: f64,
    pub teff: f64,
    pub mass: f64,
    pub colour: String,
    pub rgb: [u8; 3],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FatePlanet {
    pub name: String,
    pub a_au: f64,
    pub e: f64,
    pub i_deg: f64,
    pub engulfed: bool,
}

/// The system at one step of a look-ahead.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FateSnapshot {
    pub age_myr: f64,
    #[ts(type = "number")]
    pub year: i64,
    pub star: FateStar,
    pub planets: Vec<FatePlanet>,
}

/// A pack stack change recorded in the open universe.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StackAdopted {
    pub event_id: String,
    pub state_hash: String,
    pub manifest_id: String,
}

pub const PROTOCOL: &str = "sse.kernel/1";
