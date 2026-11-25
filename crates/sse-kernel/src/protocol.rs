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

/// A field of view for `sky_from_here`: galactic longitude and latitude
/// of its centre, degrees, and its half angle.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SkyCone {
    pub l_deg: f64,
    pub b_deg: f64,
    pub half_angle_deg: f64,
}

/// One edit of a batch (`set_authored_batch`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BatchEdit {
    pub address: String,
    pub field: String,
    #[ts(type = "unknown")]
    pub value: serde_json::Value,
    #[serde(default)]
    #[ts(optional)]
    pub impossible: Option<bool>,
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
    /// Many authored edits at one time, chronicled as ONE `authored.set`
    /// event (a paste on the Build desk is one undo step). The whole batch
    /// is refused if any address is outside the universe or any value is
    /// malformed. Impossible and possible edits go in separate batches.
    /// `evidence` is `authored` (the default) or `catalogued`, for facts
    /// entered from a published catalogue (the start sheet's Sol); a
    /// catalogued batch may hold no impossible edit. `source` is the
    /// citation, kept on the event and shown in `resolved_system`'s evidence.
    SetAuthoredBatch {
        edits: Vec<BatchEdit>,
        occurred_at_s: i64,
        #[serde(default)]
        #[ts(optional)]
        evidence: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        source: Option<String>,
    },
    /// Withdraw a subject and everything under it from `at_s` on (Undo of
    /// a paste): it drops out of scene, resolved_system and systems. Nothing
    /// is deleted; the history keeps the event.
    Withdraw {
        address: String,
        at_s: i64,
    },
    /// Bring a withdrawn subject back from `at_s` on (Redo), with every fact
    /// it had.
    Restore {
        address: String,
        at_s: i64,
    },
    History {
        address: String,
        /// Also give each event's fact changes at exactly this address (the ship's Log names what a refit
        /// changed). Off by default, so the plain answer is unchanged.
        #[serde(default)]
        #[ts(optional)]
        changes: Option<bool>,
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
        /// DM6: the campaign date the blocks are asked at (seconds on the
        /// campaign clock). A constant with a history on the sheet is read
        /// at that date; with none, the date changes nothing. Left out, the
        /// campaign's now.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    Explain {
        flow: String,
        inputs: BTreeMap<String, InputSpec>,
        kind: String,
        /// DM6: as `derive`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        at_s: Option<i64>,
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
    /// SV1: what each audience knows of `address` at `at_s` (absent: now):
    /// per audience the rung (the data view in force), the landmark
    /// (`gm_only`, `heard_of`, `known`), the thing's one-field exceptions,
    /// cover stories (SV5) and a future-dated change still waiting. With
    /// `field`, each landmark is that field's. GM only.
    KnownTo {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        field: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// SV1: set what `audience` knows: a thing's landmark (`landmark` null
    /// takes its own grant back) or, with `field`, one field's exception
    /// (null drops it). `scope` is `this` (the default), `children` (the
    /// thing and everything under it that has its own setting) or
    /// `{list: [addresses]}`. `from_s` is when the news reaches them (later
    /// waits on the campaign clock; earlier: known since). ONE
    /// `observe.grant` event, so one undo. GM only.
    SetKnown {
        address: String,
        audience: String,
        #[ts(type = "string | null")]
        landmark: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        field: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        scope: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        from_s: Option<i64>,
        at_s: i64,
    },
    /// SV1: put back what one `set_known` changed (its `undo` id), as one
    /// more dated event. GM only.
    UndoKnown {
        undo: String,
        at_s: i64,
    },
    /// SV1: for each audience, how many things at or under `bounds` stand at
    /// each landmark at `at_s` (absent: now), counted only over what the
    /// GM's scene holds (the table ledger, SV4). GM only.
    KnownIn {
        bounds: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
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
    /// "Make this system this old, today" (top-level design 12.2 and 17.1):
    /// the campaign's now stays put and this system's star is made `age_myr`
    /// old at it (older or younger), by an authored `system.age_shift_myr`.
    /// The kernel then works back one plausible birthplace and path for the
    /// star in the galaxy (a `system.origin` event, evidence resolved), unless
    /// the GM has pinned one (`origin.pinned`). `composition`: `follow`
    /// (default: its [Fe/H] becomes the gas of that birthplace, written as
    /// `star.metallicity_z`; the planets are not re-derived yet) or `keep`
    /// (birthplaces weighed by it, marked unusual when no birthplace of that
    /// age explains it). IMPOSSIBLE (allowed, stamped with why) only when no
    /// origin fits. GM machine only.
    SetSystemAge {
        system: String,
        age_myr: f64,
        #[serde(default)]
        #[ts(optional)]
        composition: Option<String>,
    },
    /// Commit a look-ahead branch (top-level design 12.2). `accept` names
    /// what accepting means, at star age `move_now_myr` (default the
    /// look-ahead's target):
    /// - `campaign_now`: "make this the campaign's now". The one campaign
    ///   clock moves on by the years between, so every system is that much
    ///   older; the branch is kept as this system's future.
    /// - `system_age`: "make this system this old, today". The clock stays;
    ///   this star's birth moves earlier instead (the look-ahead is run again
    ///   from the earlier birth, so its events fall before today), with a
    ///   reconstructed origin as `set_system_age`. The default when only
    ///   `move_now_myr` is given.
    ///
    /// With neither, the branch is kept as history and nothing moves.
    /// (T2's per-system `system.now_year` is no longer written.)
    CommitLookAhead {
        branch: String,
        #[serde(default)]
        #[ts(optional)]
        accept: Option<String>,
        /// For `system_age`: `follow` (default) or `keep`, as `set_system_age`.
        #[serde(default)]
        #[ts(optional)]
        composition: Option<String>,
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
    /// Weather from the rule-based weather block (weather track): `want` is
    /// `climate`, `systems`, `field` (a cube-sphere `tile` [face, level, x,
    /// y], `n` × `n` cells), `report`, `column` (descent hazards) or `sound`,
    /// at `day` (days since the campaign epoch) and `lat`/`lon` where a place
    /// is needed. The world is a weather fixture by `world` label, given
    /// `conditions`, or a `body` of an adopted `system` (read at `at_s`, like
    /// `survey_surface`), whose conditions are built by the same function the
    /// surface's wear and cover use (W7a, one climate); `pins` are recorded or
    /// GM-authored systems to honour.
    Weather {
        want: String,
        #[serde(default)]
        #[ts(optional)]
        world: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        body: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        conditions: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        day: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        tile: Option<[u32; 4]>,
        #[serde(default)]
        #[ts(optional)]
        n: Option<u32>,
        #[serde(default)]
        #[ts(optional)]
        lat: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        lon: Option<f64>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        pins: Option<Vec<serde_json::Value>>,
        /// Smoke, ash and smog sources (volcanoes, eruptions, impacts,
        /// fires, industry), `sse_weather::aerosols::Sources`.
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        sources: Option<serde_json::Value>,
        /// Which weather block, `{id, version, level}`; default the rule block.
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        block: Option<serde_json::Value>,
    },
    /// Light from bodies (O2): each body's spectrum (own glow, reflected
    /// light, auroral radio, discs, coronae, radiators, transmitters, lamps)
    /// and its power in every band, from conditions (`sse_light::emit::Conditions`,
    /// keyed on conditions, never on kind). `fixture: "solar-system"` serves
    /// the Solar System; given `bodies` are described instead.
    ReadLight {
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        bodies: Option<Vec<serde_json::Value>>,
    },
    /// Sensing (O9): for each target (light given as `conditions` or a ready
    /// `light`, distance, dust as A_V, bright neighbours with their sky
    /// separation) and each sensor (band, aperture, noise, stare,
    /// threshold), whether it is detected, S/N, the stare needed and the
    /// range at which it would be. With `radar`, how far a radar sees by its
    /// echo against how far its pulse is heard.
    Detect {
        #[ts(type = "unknown[]")]
        targets: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        sensors: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        radar: Option<serde_json::Value>,
        /// The setting's range bands (a measure, RK11): each detection's
        /// distance and range are labelled with them.
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        range_bands: Option<Vec<serde_json::Value>>,
    },
    /// Air optics (light track, for the renderer's sky): for each air (gas
    /// fractions by number, surface pressure, temperature, gravity), Rayleigh
    /// cross-section, scattering per metre and optical depth at
    /// `wavelengths_m` (default 680, 550 and 440 nm) and across the visible
    /// slices, with scale height and surface density.
    ReadAirOptics {
        #[ts(type = "unknown[]")]
        airs: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        wavelengths_m: Option<Vec<f64>>,
    },
    /// Light through gas (ND2): each passage is a light (`light`, a light
    /// description, or `t_k`, a black body) and the gas it crosses
    /// (`layers` of temperature, depth and number density per sheet gas,
    /// and a `path_factor`: 1 for a glow seen straight on, 2 for light down
    /// to a cloud deck and back). Each gas's absorption (measured bands,
    /// pressure-widened lines, colliding pairs) comes from its row on the
    /// data sheet. Returns the light that comes out at `wavelengths_m`
    /// (default: the eye's), what each feature did, and the colour to the
    /// eye before and after.
    LightThroughGas {
        #[ts(type = "unknown[]")]
        passages: Vec<serde_json::Value>,
    },
    /// Rainbows and halos (VP-L4): each of `items` is `{id, n | substance
    /// (+ wavelength_m), kind}` with
    /// `kind` "drop" (rain of a liquid of refractive index `n`: its bows)
    /// or "crystal" (ice-like crystals: their 22° and 46° style halos).
    ReadBows {
        #[ts(type = "unknown[]")]
        items: Vec<serde_json::Value>,
    },
    /// Phase tables per kind of particle (K-e): each of `items` is
    /// `sse_light::scatter::Particle` `{id, radius_m, wavelength_m, n and k
    /// | substance (+ phase), medium_n?, spread?, angles_deg?}`, worked by
    /// Mie theory.
    ReadScattering {
        #[ts(type = "unknown[]")]
        items: Vec<serde_json::Value>,
    },
    /// Fog of war from sensors (RK14, Atomic Rockets plan section 17), the
    /// sums: each seat gives `seat`, its instruments (`sensors`, or `fit`
    /// `{tech, hull_length_m}` for the free default set), optionally `own`
    /// (its craft, for the "noticed out to here" ring) and `contacts`
    /// (`sse_sense::fog::Contact`: a craft read from its power flow, its
    /// distance, and when it was last seen). Each contact comes back
    /// `ink`, `dotted` or `unseen`; writing the marks as `observe.grant`
    /// events is the caller's.
    FogOfWar {
        #[ts(type = "unknown[]")]
        seats: Vec<serde_json::Value>,
    },
    /// The limb law (ND2 part 2b): how each disc's brightness and colour
    /// change from centre (`mu` 1) to edge (`mu` 0). A glowing disc gives
    /// `teff_k` (Eddington-Barbier in a grey atmosphere: the layer seen,
    /// its temperature, brightness per wavelength against the centre and
    /// the colour to the eye); a lit disc gives `minnaert_k` (one, or one
    /// per wavelength) and `mu0`, the sunlight's cosine.
    ReadLimb {
        #[ts(type = "unknown[]")]
        discs: Vec<serde_json::Value>,
    },
    /// Effect at range (RK12, Atomic Rockets plan section 13): each of
    /// `asks` is `sse_sense::reach::Ask`, a weapon (beam, burst or
    /// projectile), a target (its skin as a sheet material or a typed
    /// figure, its size and how hard it can dodge) and a distance. Returns
    /// what arrives, what the skin can take, the reading (in range,
    /// marginal, out of range), the chance of a hit from light lag, the
    /// distances where the reading changes for a ring, and a beam's load on
    /// the power ledger. Nothing is fired.
    EffectAtRange {
        #[ts(type = "unknown[]")]
        asks: Vec<serde_json::Value>,
    },
    /// Spectrum as it reaches an observer (O12): each target is one or more
    /// components (`conditions` as for `read_light`, or a ready `light`
    /// description), each with a line-of-sight speed; the observer gives the
    /// wavelength ranges its instruments cover (none: everything) and a
    /// resolving power. Returns flux samples with coverage, band fluxes,
    /// emission and absorption lines where they arrive, and readings
    /// (temperature from the peak, speeds, doubled lines) with Why.
    ReadSpectrum {
        #[ts(type = "unknown[]")]
        targets: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        observer: Option<serde_json::Value>,
    },
    /// Stellar activity (O10): for each star (conditions: mass, radius,
    /// power, temperature, age, and spin or a close companion's orbit), its
    /// activity level, cycle, spots and coronal X-rays, the state at `at_s`
    /// (default: the star's age), and, with `window`, its flares and mass
    /// ejections as keyed events, each ejection hitting or missing the
    /// star's `targets` by geometry.
    ReadActivity {
        #[ts(type = "unknown[]")]
        stars: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        seed: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<f64>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        window: Option<serde_json::Value>,
    },
    /// What stops the wind at each body (MF6 part 1): for each of `bodies`
    /// (radius, dipole moment, air, the star's wind there, the host for a
    /// moon, conducting layers), the obstacle (own field, air or bare
    /// surface), the bubble's flaring, whether a moon sits inside, outside or
    /// crosses its host's bubble, and each layer's induced field. Conditions
    /// are handed in until MF2 publishes each world's field as facts, when
    /// `system` and `body` addressing joins. `fixture: "solar-system"`
    /// reads the Solar System's six magnetised planets as measured
    /// (`packs/base-sol`, `tests/fixtures/magnetic/measured-fields.json`,
    /// `crates/sse-field/data/sol-field.json`) and adds each one's `name`
    /// and `field` (reference radius, Gauss coefficients g10, g11, h11 in T,
    /// tilt, offset, polarity, grade and sources).
    ReadField {
        #[serde(default)]
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        /// An adopted system: every planet read from its facts, each with a
        /// `glance` (what holds off the wind, bubble size, shielded or not,
        /// the ground's dose as a share of open space, belt extent, and
        /// whether the field is measured, computed, none or unresolved).
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// A body's elements (MP6): every element's kilograms, worked out on a
    /// click from the body's materials and the sheet (each element rides in
    /// the material it condenses with). `level` "orbit" (default) gives
    /// totals; "survey" also says where each is (core, mantle, crust, sea,
    /// air), with the same totals. `body` picks one world of `system`; left
    /// out, every world. No surface stage runs.
    ReadComposition {
        system: String,
        #[serde(default)]
        #[ts(optional)]
        body: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        level: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// Natural resource inventory (TG11, MP8b): for each world of `system`
    /// (or `body`), the kilograms of each resource kind in `kinds` (pack
    /// kinds from `provides.other.resources`, or any element id or `h2o`
    /// straight from the sheet; left out, every pack kind) within the
    /// kind's reach (crust, sea, air, surface or whole), read off
    /// `read_composition` at Survey. No dice; no surface stage runs.
    ReadInventory {
        system: String,
        #[serde(default)]
        #[ts(optional)]
        body: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        kinds: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// Magnitudes in photometric bands (night sky Y2, light side): for each
    /// of `stars` (a photosphere: `temperature_k` with `radius_m` or
    /// `luminosity_w`, and a sight line: `distance_m`, dust `a_v` and `r_v`),
    /// AB and Vega magnitudes per band, the dust's dimming per band, and
    /// colours with their dust excess. `bands` picks bands by id (U, B, V,
    /// R, I, J, H, Ks); empty means all. The sky draw uses the same path.
    ReadPhotometry {
        #[ts(type = "unknown[]")]
        stars: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        bands: Option<Vec<String>>,
    },
    /// Planet finding from afar (O13): one star, its planets, an observer
    /// (direction and distance), an instrument and a watch time. Returns
    /// what each method (transit, wobble, position wobble, direct imaging,
    /// timing) finds, with what players would measure and why. Mass from a
    /// wobble is the minimum mass, as in reality. GM-side: the planets are
    /// the truth the survey is run against.
    SurveyPlanets {
        #[ts(type = "unknown")]
        survey: serde_json::Value,
    },
    /// Nebulae at a date (O3): planetary nebulae from stars' deaths (or given
    /// inputs), supernova remnants and star-forming glows, each a descriptor
    /// (shells, lobes, surroundings, lighting stars, light, decoration seed).
    /// Nebulae not yet born or already faded are listed in `absent`.
    ReadNebulae {
        at_s: f64,
        #[serde(default)]
        #[ts(optional)]
        seed: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        deaths: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        planetary: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        remnants: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        glows: Option<Vec<serde_json::Value>>,
        /// Cool giants losing mass (`sse_nebula::outflow::GiantInput`): their
        /// dusty shells (O3b).
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        giants: Option<Vec<serde_json::Value>>,
        /// Young stars still accreting (`sse_nebula::outflow::YoungStarInput`):
        /// their jets (O3b).
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        young: Option<Vec<serde_json::Value>>,
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
    /// Preview a class atlas scheme over the open campaign (GM machine):
    /// label every body in `scope` (default the whole universe) at `at_s`
    /// (default the latest state) with `scheme`, from the atlas of the
    /// candidate stack `packs` (bottom to top, as `load_stack`). Nothing is
    /// loaded or written: the stack in use is unchanged. Also reports any
    /// class example the candidate atlas gets wrong.
    ClassifyPreview {
        #[ts(type = "unknown[]")]
        packs: Vec<serde_json::Value>,
        scheme: String,
        #[serde(default)]
        #[ts(optional)]
        scope: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
    },
    /// NAV (T3): candidate courses between two places, or the ways out
    /// for a drifting ship (no `to`). The system is a NAV `fixture` (`sol`),
    /// an inline `nav` system, or an adopted `system` read at `at_s` (its
    /// planets; `ports` names bodies with ports for rescue). `ship` is a
    /// template id from the base-nav pack or a ship object; `from` and `to`
    /// are a body address or name, or `{r_m, v_m_s}` around the star;
    /// `prefer` is `fastest`, `save-fuel` or `balanced`.
    PlanRoute {
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        ports: Option<Vec<String>>,
        #[ts(type = "unknown")]
        ship: serde_json::Value,
        #[ts(type = "unknown")]
        from: serde_json::Value,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        to: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        depart_s: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        window_days: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        prefer: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        rescue: Option<serde_json::Value>,
    },
    /// A ship at each of some worlds (aerobraking and landing on the
    /// physics): aerocapture, entry and landing, runway landing, powered
    /// landing and take-off, each with its figures, dangers (figure against
    /// limit, level, rough chance, what happens if it goes wrong, effects on
    /// kept ship state), what would make it safe, and why. Nothing is
    /// refused and nothing rolled: the GM judges. The system is a NAV
    /// `fixture` (`sol`), an inline `nav` system (bodies may carry `air`), or
    /// an adopted `system` read at `at_s`; `ship` is a template id or a ship
    /// object (optional `shield`, `aero`, `landing`, `crew`, `structure_g`);
    /// `worlds` are body names or addresses; `v_inf_m_s` is the speed
    /// arriving from far away (default: a Hohmann transfer from the next body in).
    ShipAtWorld {
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[ts(type = "unknown")]
        ship: serde_json::Value,
        worlds: Vec<String>,
        #[serde(default)]
        #[ts(optional)]
        v_inf_m_s: Option<f64>,
    },
    /// Where an FTL leg may begin (or end, `end: true`) in a system at `at_s`
    /// under a pack's FTL `rule` (example packs 4A: clear of bodies by a
    /// multiple of their size, gravity below or above a level, Lagrange
    /// points, the star's poles, gates, anywhere): spheres to be clear of,
    /// points, gates asked for, gravity levels and the rule in words, for the
    /// chart's HUD layer and the voyage finder. Same sources as `ship_at_world`.
    JumpMap {
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[ts(type = "unknown")]
        rule: serde_json::Value,
        #[serde(default)]
        #[ts(optional)]
        end: Option<bool>,
    },
    /// The story finder (U8) and the agents' sift: run sifters over every
    /// address at or under `scope` (default the universe) at `at_s` (default
    /// now). `sifters` holds names of the stack's sifters
    /// (`provides.other.sifters`) and inline sifter objects: with no names,
    /// every stack sifter runs, plus the inline ones; with names, only those
    /// named, plus the inline ones. With `audience`, only what that
    /// audience's scene shows is sifted and event patterns are skipped
    /// (players never see the chronicle). Hits come best first, each with its
    /// evidence and significance. GM machine only.
    Sift {
        #[serde(default)]
        #[ts(optional)]
        scope: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        sifters: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional)]
        audience: Option<String>,
    },
    /// The Window (V8): what a `sensor` (`eye`, `binoculars`, `telescope` in
    /// base-nav) picks up at `at_s` from beside body `from`, or from the point
    /// `at_au` around the star, in a NAV `fixture` or inline `nav` system:
    /// each object's sky position, size, brightness, phase, whether it is
    /// seen and whether its disc shows. Read only.
    WindowView {
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        from: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        at_au: Option<[f64; 3]>,
        #[serde(default)]
        #[ts(optional)]
        sensor: Option<String>,
        /// The system an inline `nav` is, so the view's `frame` can be
        /// keyed (needs an open universe); the Sol fixture's is measured.
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
    },
    /// G1 to G4 for the galaxy chart (U6): one galaxy grown from the open
    /// universe's seed, a named `halo` (`dwarf`, `milky-way-like`, `group`) or
    /// a halo `mass_msun`; its history curves, rings now (abundances, deaths,
    /// rates), green band through time, the cosmic epochs, and the
    /// conditions at `place_kpc` (and `place_phi`, radians) at `at_gyr`. Read only.
    GalaxyChart {
        #[serde(default)]
        #[ts(optional)]
        halo: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        mass_msun: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        at_gyr: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        place_kpc: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        place_phi: Option<f64>,
        /// History edits to preview (galaxy design 11, G6): `{kind: "merger", t_gyr, ratio}`,
        /// `{kind: "remove_merger", index}`, `{kind: "starburst", t_gyr, gas_fraction}`,
        /// `{kind: "quiet_nucleus", from_gyr, to_gyr}`.
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        edits: Option<Vec<serde_json::Value>>,
        /// A halo cell's index (from `halo_cell`): `halo` is then one of its
        /// catalogued haloes (`halo:3` or its full address), with its own history.
        #[serde(default)]
        #[ts(optional)]
        cell: Option<[i64; 3]>,
    },
    /// G1 for the foam and cluster maps (V9): a square slice of the cosmic
    /// web from the open universe's seed, `n` samples a side over
    /// `extent_mpc` around `centre` (Mpc, default the origin), at redshift
    /// `z` or cosmic time `t_gyr` (default now). With `cube: true` it is a
    /// cube for the 3D view instead: n^3 parcels (n 4 to 48) moved by the
    /// truncated Zel'dovich approximation, each with its environment, its
    /// starting density and the room it takes today. With `haloes: true` as
    /// well it gives the same cube's `galaxies` instead (V9w): G2's haloes
    /// from the pack's web light floor up that started inside it, moved by
    /// the same step, each with its G3 galaxy's stars and star formation
    /// then; ask for the parcels and the galaxies apart so the parcels show
    /// first. Read only.
    WebSlice {
        #[serde(default)]
        #[ts(optional)]
        cube: Option<bool>,
        #[serde(default)]
        #[ts(optional)]
        haloes: Option<bool>,
        /// With `haloes`: the smallest halo drawn, solar masses (never below
        /// the pack's floor), so a bigger cube can keep to the brighter ones.
        #[serde(default)]
        #[ts(optional)]
        galaxy_floor_msun: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        z: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        t_gyr: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        centre: Option<[f64; 3]>,
        extent_mpc: f64,
        n: u32,
    },
    /// Time-lapse frames: a world's surface (a `fixture`, or a `body` of an
    /// adopted `system` at `at_s`) as keyframes on the S2 plate grid at its
    /// plate snapshot dates, from its first plates to the epoch, with dated
    /// events. `detail` is `orbit` (the coarse grid, default) or `survey`;
    /// `frames_max` thins the frames evenly (the last always kept).
    /// Read only.
    SurfaceFrames {
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
        detail: Option<String>,
        /// At most this many frames, evenly spaced, always ending on the epoch.
        #[serde(default)]
        #[ts(optional)]
        frames_max: Option<u32>,
    },
    /// S14a's cave survey (Site level): survey a world as `survey_surface`
    /// does (a `fixture`, or a `body` of an adopted `system` at `at_s`) and
    /// place the lava-tube passages within `radius_km` (default 5, at most
    /// the pack's window) of `lat_deg`, `lon_deg`. The same tube is the same
    /// from any site. Read only.
    SurveyCaves {
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
        quick: Option<bool>,
        lat_deg: f64,
        lon_deg: f64,
        #[serde(default)]
        #[ts(optional)]
        radius_km: Option<f64>,
    },
    /// S5's descent scene: survey a world as `survey_surface` does (a
    /// `fixture`, or a `body` of an adopted `system` at `at_s`), judge the
    /// landing site at `lat_deg`, `lon_deg` (slope, roughness, liquid) and
    /// fly `lander` (a base-nav template id, a lander object, or `{ship}`: a
    /// ship id or nav ship landed on its own numbers) down to it
    /// from `entry` (`{altitude_m, speed_m_s, flight_path_deg}`; default the
    /// top of the air at orbital speed, or a low orbit on an airless world).
    PlanDescent {
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
        quick: Option<bool>,
        #[ts(type = "unknown")]
        lander: serde_json::Value,
        lat_deg: f64,
        lon_deg: f64,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        entry: Option<serde_json::Value>,
    },
    /// S2 to S7 in order for one world: a surface `fixture` (`Earth`,
    /// `Moon`, ...; no universe needed) or a `body` of an adopted `system`
    /// at `at_s`. `options` is sse-surface's `SurveyOptions` (rotation_hours,
    /// obliquity_deg, life, settle, reshape, names, renames, wind_m_s);
    /// `tiles` are S1 tiles of this world with S7's layers; `quick` is the
    /// Orbit detail level (S16): the coarse plate run a full survey refines,
    /// so the same continents, and 32-sample tiles (tests, weak devices). `limit`
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
    /// The record (J1): write a note at `address`, linked to `links`. The
    /// audience is `"everyone"`, `"gm"` (the default) or a list of audience
    /// ids; with `tier` it shows only where the audience's view shows that
    /// tier. `supersedes` corrects an earlier note (empty `text` withdraws
    /// it). Notes never change state; send the event only to the seats the
    /// response's `audience` includes.
    AddNote {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        links: Option<Vec<String>>,
        text: String,
        #[serde(default)]
        #[ts(optional)]
        tags: Option<Vec<String>>,
        /// The writing seat (default `gm`).
        #[serde(default)]
        #[ts(optional)]
        by: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "\"everyone\" | \"gm\" | string[]")]
        audience: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        tier: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        supersedes: Option<String>,
        /// Written by an agent: evidence `narrative`.
        #[serde(default)]
        #[ts(optional)]
        narrative: Option<bool>,
        at_s: i64,
        /// When the news reaches its audience (default `at_s`); not shown or
        /// disclosed before then.
        #[serde(default)]
        #[ts(optional)]
        arrives_at_s: Option<i64>,
    },
    /// The record: change a kept value (`provides.other.record.kept`, or
    /// `misc.<name>` for the GM). `value` null clears it. Refused unless
    /// `writer` (default `gm`) is one of the kind's writers.
    SetKept {
        address: String,
        kind: String,
        #[ts(type = "unknown")]
        value: serde_json::Value,
        #[serde(default)]
        #[ts(optional)]
        writer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        reason: Option<String>,
        at_s: i64,
    },
    /// The record: an actor (empire, guild, noble, party, crew) at
    /// `<root>/actor:<id>` or under `parent`.
    AddActor {
        id: String,
        #[serde(default)]
        #[ts(optional)]
        parent: Option<String>,
        name: String,
        kind: String,
        at_s: i64,
    },
    /// The record at `address` as of `at_s` (absent: latest), for the GM:
    /// live notes at or linked to it, kept values with their history, and
    /// actors under it. Never send this to a seat; seats get `disclosures`.
    Record {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// OV3: a holding on a map layer, one dated record event. `status` is
    /// `holds` (one holder per place per layer), `claims` (by `by`) or
    /// `influences` (with `strength` 0 to 1). `extent` is `{reach: {value,
    /// unit}}`, `{band: [q, q]}` or `{outline: [[x, y]...], frame}`.
    /// `withdraw` ends the holding in that slot. The layer is a pack's area
    /// layer or a GM's own `misc.*` one.
    Hold {
        place: String,
        layer: String,
        key: String,
        status: String,
        #[serde(default)]
        #[ts(optional)]
        strength: Option<f64>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        extent: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        by: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        tier: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        audience: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        writer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        reason: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        source: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        withdraw: Option<bool>,
        at_s: i64,
    },
    /// OV3: who holds a place (or a point drawn on a place: `point` is
    /// `{frame, xy}`, in `body-latlon` or `galaxy-plane-pc`) on a layer at
    /// `at_s` (absent: latest), seen by `audience` (absent: the GM), drawn
    /// as `according_to` (an actor whose claims read as held). The answer
    /// has `via` (held, inherited, rolled-up, rule), `divided` below a 0.6
    /// share, the claims and influences there (`also`) and `why`.
    WhoHolds {
        place: String,
        layer: String,
        #[serde(default)]
        #[ts(optional)]
        point: Option<HoldPoint>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        audience: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        according_to: Option<String>,
    },
    /// OV3: the answers for `bounds` and every place held, claimed or
    /// influenced inside it on a layer, as `who_holds` gives them; with
    /// `history`, also every holding there that started by `at_s`, with
    /// `until_s` on those that ended (the ruler's border changes).
    HoldingsIn {
        bounds: String,
        layer: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        audience: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        according_to: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        history: Option<bool>,
    },
    /// Space highways: a link between two places in the universe, one dated
    /// record event. `nav` is the travel part in sse-nav's `Link` shape
    /// (each end's `place` conditions as `{a: {place}, b: {place}}`, an
    /// optional `rule` of its own, `counted` measures); without a rule the
    /// link is a charted lane. `withdraw` ends it.
    SetLink {
        id: String,
        a: String,
        b: String,
        #[serde(default)]
        #[ts(optional)]
        name: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        kind: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        one_way: Option<bool>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        tier: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        audience: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        writer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        reason: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        source: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        withdraw: Option<bool>,
        at_s: i64,
    },
    /// Space highways: the links in force at `at_s` (absent: latest) that
    /// `audience` knows (absent: the GM), with an end inside `bounds`
    /// (absent: the whole universe); with `history`, every one by date.
    LinksIn {
        #[serde(default)]
        #[ts(optional)]
        bounds: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        audience: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        history: Option<bool>,
    },
    /// FT2: a civilisation's technology, one dated record event on an actor
    /// (or on the universe root, for the whole setting): a `level` on the
    /// setting's ladder, advances gained early (`gain`, "Develop it") or
    /// never taken up (`lose`). Plain data; nothing grows on its own.
    SetTech {
        holder: String,
        #[serde(default)]
        #[ts(optional)]
        level: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        gain: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        lose: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        writer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        reason: Option<String>,
        at_s: i64,
    },
    /// FT2: a civilisation's technology at `at_s` (absent: latest), with
    /// what it inherits from its parents and the setting: level, gained and
    /// lost advances (each with who said it, and when), and every advance it
    /// holds.
    TechOf {
        holder: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// FT2: the availability reading of a design through a civilisation:
    /// `parts` are `{part, needs: {figure: value}}`. Per need: `can build`,
    /// `short` (levels short and the prototype's size) or `past known`, with
    /// `fictional` naming the law where a need passes a ceiling; overall the
    /// worst, the parts' size factors and the advances it leans on.
    TechReading {
        holder: String,
        #[ts(type = "unknown[]")]
        parts: Vec<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// What one audience may be sent of the record at `at_s` (record design
    /// 8A): per subject, the notes it may see (already arrived) and the kept
    /// values its view shows, at that view's precision. No event ids, no
    /// causes; note ids (`n1`, `n2`, ...) mean something only in this stream.
    /// Built on the GM machine; send each seat its own.
    Disclosures {
        audience: String,
        at_s: i64,
    },
    /// Step the stack's trigger rules (J4, `provides.other.triggers`) to a time boundary:
    /// fire what is due, pass each firing's effects through the gate under its
    /// approval policy, and run further passes for what they set off (up to
    /// the pass limit, then the chain is reported). Boundaries run forwards.
    /// GM machine only: never run on a seat.
    RulesStep {
        at_s: i64,
    },
    /// Proposals waiting for the GM (`ask` and `ask_if` effects).
    RulesPending,
    /// The GM answers a proposal; yes applies its effects at `at_s` (default:
    /// the proposal's time).
    RulesDecide {
        proposal: String,
        approve: bool,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// Change a rule's approval policy without editing its pack:
    /// `{"policy":"auto"}`, `{"policy":"ask"}` or `{"policy":"ask_if","condition":{..}}`.
    RulesPolicy {
        rule: String,
        #[ts(type = "unknown")]
        approval: serde_json::Value,
        at_s: i64,
    },
    /// The loaded rules with their effective policies, and the record's
    /// `rules.*` trail (GM only).
    Rules,
    /// Bodies at or under `system` placed on their orbits at each time
    /// (universe seconds, at most 64): position relative to the host in
    /// metres, longitude in degrees, period. Reads the universe's facts, or
    /// `bodies` (`[{address, facts}]`) when given. A body with no dated phase
    /// takes a keyed draw (`phase_source: resolved`); one missing anything
    /// else comes back unplaced with `missing` named, never guessed.
    OrbitsAt {
        system: String,
        at_s: Vec<i64>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        bodies: Option<Vec<serde_json::Value>>,
    },
    /// RK9: a ship's power books (the four-bus ledger, `sse-nav` power):
    /// what its sources make at `flux_w_m2` (default 1361, the Earth's) and
    /// `at_s` (default 0), what its loads and engines take, heat by the
    /// temperature it leaves at, what came from nowhere with why, and one
    /// balance shaped like `tech_reading`'s. Fitted engines set `accel_g`
    /// and `exhaust_km_s`. `ship` is a template id or a ship in full.
    ShipPower {
        #[ts(type = "unknown")]
        ship: serde_json::Value,
        #[serde(default)]
        #[ts(optional)]
        flux_w_m2: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// V8 flight log: write one entry of a construct's log on the record,
    /// written at `at_s` (the campaign clock). Give one of `piece` (sse-nav
    /// `Piece`: `{from_s, to_s?, state, mark, src?, why?}`), `event`
    /// (`LogEvent`: `{at_s, kind, mark, ...}`), `name` and `about`, or
    /// `withdraw` (an earlier entry's id). A `split` event starts each new
    /// construct's log in the state its parent was in then; a `join` docks
    /// the construct it names to this one.
    LogFlight {
        construct: String,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        piece: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        event: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        name: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        about: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        withdraw: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        writer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        reason: Option<String>,
        at_s: i64,
    },
    /// V8 flight log: a construct's log as written by `at_s` (absent: all of
    /// it), and with `where_at` and a system (a NAV `fixture`, an inline
    /// `nav` system or an adopted universe `system`), where the craft was
    /// then.
    FlightLog {
        construct: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        where_at: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
    },
    /// ST1: whether the orbits at or under `system` last (Hill spacing,
    /// angular momentum deficit, moon limits, Kozai-Lidov swings, binary
    /// limits), each check a formula block shown with its numbers. Reads the
    /// universe's facts at `at_s` (absent: all of them), or `bodies`
    /// (`[{address, facts}]`); `add` (same shape) is laid over them, and
    /// then only the checks that concern the added bodies come back (the
    /// card's verdict on add).
    OrbitStability {
        system: String,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        bodies: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        add: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// Set the campaign's now (`campaign.now_s` on the universe root, a
    /// `time.campaign_now` event). It moves forwards or back.
    SetCampaignNow {
        at_s: i64,
    },
    /// The campaign's now (absent if never set).
    CampaignNow,
    /// The top pack's calendar (`provides.other.calendar`), checked at load.
    Calendar,
    /// The intent registry (J5): built-in kinds and the stack's
    /// (`provides.other.intents`), with their fields, who applies them and
    /// whether they ever apply without the GM; the seat plan in force; and
    /// intents waiting for the GM. GM machine only.
    Intents,
    /// Store the seat plan (as the chart room's `SeatPlan.seats`, plus
    /// `program` for a program-driven seat). A GM seat is always present.
    SetSeats {
        #[ts(type = "unknown[]")]
        seats: Vec<serde_json::Value>,
        at_s: i64,
    },
    /// A seat sends an intent (`{kind, ...fields}`). The kernel checks it
    /// against the registry and the seat, then applies it, queues it for the
    /// GM, or refuses it, as the chart room's `decide`. `check_only` decides
    /// and writes nothing.
    SendIntent {
        seat: String,
        #[ts(type = "unknown")]
        intent: serde_json::Value,
        at_s: i64,
        #[serde(default)]
        #[ts(optional)]
        check_only: Option<bool>,
    },
    /// The stack's templates (J7, `provides.other.templates`).
    Templates,
    /// Make an authored object from a template under `under`
    /// (`<under>/<kind>:<key>`, the next free number when no key): its facts,
    /// tags and programs in one universe event, its kept values in the record.
    /// GM machine only.
    Create {
        template: String,
        under: String,
        #[serde(default)]
        #[ts(optional)]
        key: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        name: Option<String>,
        at_s: i64,
    },
    /// Take an object made from a template out of play (`retired`). Nothing is deleted.
    Retire {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        reason: Option<String>,
        at_s: i64,
    },
    /// RS4: split a construct at `at_s`. The named parts (`part.<id>.*`
    /// facts) and `mass_kg` more move to a new construct of the same kind
    /// beside it (`key`, else the next free number; `name`, else the old
    /// name and the first part), which carries `split.from`. `construct.mass_kg`
    /// drops by what moved.
    SplitConstruct {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        key: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        name: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        parts: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        mass_kg: Option<f64>,
        at_s: i64,
    },
    /// RS4: join `with` to `address` at `at_s`: `dock` (the default; `with`
    /// carries `docked.to`), `undock`, or `merge` (its parts and mass move
    /// across and it is retired with `joined.into`).
    JoinConstructs {
        address: String,
        with: String,
        #[serde(default)]
        #[ts(optional, type = "\"dock\" | \"undock\" | \"merge\"")]
        how: Option<sse_trigger::split::How>,
        at_s: i64,
    },
    /// The GM answers a queued intent; yes applies it at `at_s` (default: when it was sent).
    IntentDecide {
        proposal: String,
        approve: bool,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// The stack's programs (`provides.other.programs`, J6), the campaign's
    /// limit and the programs running at `at_s` (default: latest).
    Programs {
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// Start `program` on `address` (an object whose template lists it),
    /// driving `seat` (a program seat naming that program). GM machine only.
    AttachProgram {
        address: String,
        program: String,
        seat: String,
        at_s: i64,
    },
    /// Stop the program on `address`. GM machine only.
    DetachProgram {
        address: String,
        at_s: i64,
    },
    /// One tick for every running program at `at_s`: follow transitions,
    /// then sense through the seat's audience, pick and send the state's
    /// intent; then run the rules so its effects land. GM machine only.
    ProgramsStep {
        at_s: i64,
    },
    /// Game-system codes (handoff 7A part 1) for every body at or under
    /// `scope` at `at_s`, read by a pack's code sheet (`sheet`, or the first
    /// one in the stack): each digit with its why, the lines (a UWP), and the
    /// rule groups (trade codes, zones). Count digits count bodies in the
    /// same system. Read-only; GM machine (it reads every fact).
    Codes {
        scope: String,
        #[serde(default)]
        #[ts(optional)]
        sheet: Option<String>,
        at_s: i64,
    },
    /// Import a community sector file (handoff 7A part 4) into the open
    /// universe through a code sheet: UWP lines, legacy SEC or T5
    /// tab-delimited text (`format`: `t5_tab`, `sec`, `uwp_lines`; detected
    /// when left out). Without `apply` it is a preview and changes nothing;
    /// with it every world becomes authored events at `at_s` (one for its
    /// facts, one more for any impossible ones, kept and marked), so the
    /// import can be undone and replayed.
    ImportSector {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        format: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        sheet: Option<String>,
        #[serde(default)]
        apply: bool,
        /// With `apply`: grow each world's whole system first (formation
        /// pinned to the coded main world, giant and belt counts searched),
        /// adopt it, and write the coded facts on the grown main world.
        #[serde(default)]
        grow: bool,
        at_s: i64,
    },
    /// Write a campaign's mapped systems (those with `map.hex`) at or under
    /// `scope` as a sector file (`format`: `t5_tab`, `sec`, `uwp_lines`):
    /// each system's main world (the body carrying `provenance.code`, or
    /// tagged `main_world`) read through the code sheet at `at_s`.
    ExportSector {
        scope: String,
        format: String,
        #[serde(default)]
        #[ts(optional)]
        sheet: Option<String>,
        at_s: i64,
    },
    /// The hex map's worlds (handoff 7A part 4): each mapped system
    /// (`map.hex`) at or under `scope`, its main world read through the code
    /// sheet (`sheet`, or the first in the stack) at `at_s`. Read only; GM
    /// machine.
    SectorMap {
        scope: String,
        #[serde(default)]
        #[ts(optional)]
        sheet: Option<String>,
        at_s: i64,
    },
    /// G2 for V9's cluster map: one halo cell (8 Mpc a side at pack
    /// defaults) of the open universe, its haloes biggest first down to
    /// `min_mass_msun` (default 1e10, the catalogue floor), at most `limit` (default 300), with
    /// the mass function they were drawn from. Read only.
    HaloCell {
        #[serde(default)]
        #[ts(optional)]
        cell: Option<[i64; 3]>,
        #[serde(default)]
        #[ts(optional)]
        min_mass_msun: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        limit: Option<u32>,
    },
    /// DM3: one halo cell's matter ledger today: dark matter in catalogued
    /// haloes, in haloes below the catalogue floor and never collapsed;
    /// ordinary matter as stars, cold gas, hot halo gas, thrown-out gas,
    /// black holes and gas between galaxies; and the cell's volume and matter
    /// by web environment. Read only.
    MatterBudget {
        #[serde(default)]
        #[ts(optional)]
        cell: Option<[i64; 3]>,
    },
    /// G5 through the kernel (night sky design, Y7a): one 10 pc star cell of
    /// the campaign's galaxy at `t_myr` (default the campaign's now), its
    /// fill as today's rules draw it and the campaign's promoted stars read
    /// back as stored. With no `cell`, the cell at the Sun's radius. Read only.
    StarCell {
        #[serde(default)]
        #[ts(optional)]
        cell: Option<[i64; 3]>,
        #[serde(default)]
        #[ts(optional)]
        t_myr: Option<f64>,
    },
    /// Y2 (night sky design 4.3): the stars seen from a place at a time down
    /// to a limiting magnitude. The observer is a star system's address, or
    /// a galactic position in pc, or (neither) the Sun's radius. Each luminosity
    /// class is walked through the count tree inside the distance where its
    /// brightest member reaches the limit; each star found gets band
    /// magnitudes through its own dust column and is kept when it reaches
    /// the limit in `band` (default V). `cone` narrows the sky to a field.
    /// Read only.
    SkyFromHere {
        #[serde(default)]
        #[ts(optional)]
        observer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        position_pc: Option<[f64; 3]>,
        #[serde(default)]
        #[ts(optional)]
        t_myr: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        limit_mag: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        band: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        bands: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        cone: Option<SkyCone>,
        /// Most stars listed, brightest first (default 20,000, the phone
        /// tier of ruling 12; desktops ask 200,000). A cut list says
        /// `truncated` and gives `kept_to_mag`, the faintest star kept: ask
        /// `sky_glow` with that limit and the stars cut off are in the glow.
        #[serde(default)]
        #[ts(optional)]
        max_stars: Option<u32>,
        /// Y4: the other galaxies too (default true).
        #[serde(default)]
        #[ts(optional)]
        galaxies: Option<bool>,
        /// The stars (default true); false gives galaxies only.
        #[serde(default)]
        #[ts(optional)]
        stars: Option<bool>,
    },
    /// Y3: the glow of every star the sky draw does not show, as a HEALPix
    /// map (RING order, galactic l and b from the observer) per passband:
    /// each class's light beyond the distance it is drawn to, plus the
    /// stars inside it too faint for the limit, dimmed by the dust in front.
    /// Ask it with the same `limit_mag` and `band` as `sky_from_here` and
    /// the two add up to the whole sky. `nside` a power of two up to 64
    /// (default 32). Read
    /// only.
    SkyGlow {
        #[serde(default)]
        #[ts(optional)]
        position_pc: Option<[f64; 3]>,
        #[serde(default)]
        #[ts(optional)]
        t_myr: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        limit_mag: Option<f64>,
        #[serde(default)]
        #[ts(optional)]
        band: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        bands: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        nside: Option<u64>,
    },
    /// Voyages between stars (V8 part 3): the cheapest chain of legs for one
    /// ship from a star system `from` to `to`, or with no `to` the nearest
    /// system not in `visited` (ties go to the shorter path, then to the one
    /// farther from `home`). Legs are jumps (the ship's jump drive, a pack
    /// rule) or flights there for real (special relativity). `known_fuel`
    /// maps system addresses to the fuel the crew knows of there; `surveyed`
    /// lists systems known to be dry; `sensed_fuel` gives the odds (0 to 1)
    /// of fuel where the ship's sensors have looked (else the star's own
    /// giant-planet odds); `gambles` lets a plan hope for fuel in unsurveyed
    /// systems at odds above the pack's floor. `minimise` is `home_time`, `ship_time`, `fuel` or
    /// `legs`. Inside one system (a `fixture`, `nav` or `system` given) the
    /// places are its bodies and belts and the legs are NAV's courses with
    /// their windows. Read only; refusals come back in words.
    PlanVoyage {
        from: String,
        #[serde(default)]
        #[ts(optional)]
        to: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        home: Option<String>,
        #[ts(type = "unknown")]
        ship: serde_json::Value,
        #[serde(default)]
        #[ts(optional)]
        minimise: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        visited: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        known_fuel: Option<std::collections::BTreeMap<String, Vec<String>>>,
        #[serde(default)]
        #[ts(optional)]
        sensed_fuel: Option<std::collections::BTreeMap<String, f64>>,
        #[serde(default)]
        #[ts(optional)]
        surveyed: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        gambles: Option<bool>,
        /// In-system voyages (part 3b): give one of a NAV `fixture`, an
        /// inline `nav` system or an adopted `system` (read at `at_s`, with
        /// `ports`), and `from` and `to` are bodies (or `belt:<name>`).
        #[serde(default)]
        #[ts(optional)]
        fixture: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "unknown")]
        nav: Option<serde_json::Value>,
        #[serde(default)]
        #[ts(optional)]
        system: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        ports: Option<Vec<String>>,
        #[serde(default)]
        #[ts(optional)]
        depart_s: Option<f64>,
        /// Links the crew may use beyond the record's (sse-nav `Link`s), and
        /// whether every leg must run along one. The record's links come in
        /// as `audience` knows them (absent: the GM).
        #[serde(default)]
        #[ts(optional, type = "unknown[]")]
        links: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        #[ts(optional)]
        only_links: Option<bool>,
        #[serde(default)]
        #[ts(optional)]
        audience: Option<String>,
    },
    /// Make a fill star permanent: its whole drawn description (address,
    /// cell, frame and slot, birth, mass, companions, abundances, orbit) is
    /// written to the record, so no later change to the fill rules can move
    /// or change it (Y7a). Promoting a pinned star again writes nothing.
    /// GM machine only.
    PromoteStar {
        address: String,
    },
    /// The campaign's own tag declarations (TG1), in the shape of a pack's
    /// `provides.other.tags`: tags the GM typed freely join it on first use.
    CampaignTags,
    /// The master data sheet (MP1, `provides.reference`): with `table` and
    /// `id`, one row with its sources, the packs that set it and its
    /// worked-out values; with `id` alone, the first table holding it; with
    /// neither, every table's row ids. Read-only, any seat.
    ReadMatter {
        #[serde(default)]
        #[ts(optional)]
        table: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        id: Option<String>,
    },
    /// The phase of a substance at a pressure and temperature (MP17, matter
    /// pack design section 4.3A), with the phase's row: read from the
    /// substance's phase table, or from its triple point and vapour curve
    /// when it has none. Read-only, any seat.
    PhaseAt {
        substance: String,
        p_pa: f64,
        t_k: f64,
    },
    /// The reach report for a changed constant (MP1c, design section 7A.3):
    /// every block in exactly one of three groups. `follows` (a formula that
    /// reads the constant from the one handle), `estimated` (a table
    /// rescaled by a cited law) and `cannot_follow` (computed with the real
    /// constants). `value`, when given, is shown beside the stack's value.
    /// Read-only, any seat.
    ReadReach {
        constant: String,
        #[serde(default)]
        #[ts(optional)]
        value: Option<f64>,
    },
    /// Dressing candidates (TG12, `provides.other.dressing`) for every world
    /// at or under `scope` at `at_s` (default now): what the stack's universe
    /// packs would place, each with its rule, template, where, explanation
    /// and the facts its condition read. Stored nowhere; held-off and adopted
    /// ones are left out. `orphans` lists placed objects whose pack or rule
    /// left the stack. GM machine only.
    Dressing {
        scope: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
    },
    /// Adopt one dressing candidate: an authored object from its template
    /// under the world, marked with the pack, rule and candidate. GM only.
    AdoptDressing {
        candidate: String,
        #[serde(default)]
        #[ts(optional)]
        name: Option<String>,
        at_s: i64,
    },
    /// Hold a dressing candidate off for its world (`held` false lets it
    /// back). Kept in the record. GM only.
    HoldDressing {
        candidate: String,
        #[serde(default)]
        #[ts(optional)]
        held: Option<bool>,
    },
    /// Sample names from a scheme on the stack (NM1, naming design 4.12): up
    /// to 10,000 names for one kind at sample addresses from `start`, the
    /// first twelve with their faces and parts, and a digest of them all
    /// (the same on every device). Read-only, any seat; names never feed
    /// physics.
    NameSample {
        scheme: String,
        kind: String,
        #[serde(default)]
        #[ts(optional)]
        layer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        count: Option<u32>,
        #[serde(default)]
        #[ts(optional)]
        start: Option<u32>,
        #[serde(default)]
        #[ts(optional)]
        seed: Option<String>,
        /// The parent's name, for forms that follow it.
        #[serde(default)]
        #[ts(optional)]
        parent: Option<String>,
        /// Facts as words, for forms that read them (`colour`: `red`).
        #[serde(default)]
        #[ts(optional)]
        facts: Option<BTreeMap<String, String>>,
    },
    /// The names at `address` (NM2, naming design 4.2): every layer's shown
    /// name with where it came from (`typed`, `fixed`, `drawn`), the scheme
    /// in force, its parts and faces, and under a typed name the one it
    /// would have otherwise. With `audience`, only the layers that
    /// audience's data view discloses there (a name is disclosed like any
    /// fact, as the field `name.<layer>`). Read-only.
    ReadNames {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        at_s: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        audience: Option<String>,
    },
    /// Set the scheme that names `layer` at `address` and beneath (only
    /// things of `kind`, when given), from `at_s` on (design 4.3). No
    /// `scheme` takes the setting away; an empty one means no names here.
    /// Names not yet fixed change at once. With `rename_fixed`, fixed names
    /// beneath that the new scheme now governs are renamed from it in the
    /// same event, dated `at_s` (the history keeps the old ones). GM only.
    SetNaming {
        address: String,
        layer: String,
        #[serde(default)]
        #[ts(optional)]
        scheme: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        kind: Option<String>,
        at_s: i64,
        #[serde(default)]
        #[ts(optional)]
        rename_fixed: Option<bool>,
    },
    /// Type a name for `address` in `layer` (default `common`) from `at_s`
    /// on: plain text that always wins and never follows its parent. No
    /// `text` takes the typed name away, so the drawn or fixed name shows
    /// again ("follow the parent again"). GM only.
    Rename {
        address: String,
        #[serde(default)]
        #[ts(optional)]
        layer: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        text: Option<String>,
        at_s: i64,
    },
    /// Fix the drawn names at `address` and beneath from `at_s` on (design
    /// 4.4): their forms and parts are stored, so no later scheme version
    /// can change them. A name that clashes with one already fixed in the
    /// same system moves to its next draw. GM only.
    FixNames {
        address: String,
        at_s: i64,
    },
    /// ND6 (new-physics demo 2.5): each pack's sources, fudge list
    /// (declared, plus every fitted or assumed Model number its flows and
    /// boxes hold) and reproduction report (published numbers beside the
    /// pack's, worked out when the stack loaded), for the stack in force or
    /// one pack of it. Read in a trial, it shows them before the pack is
    /// added. Read-only.
    PackReports {
        #[serde(default)]
        #[ts(optional)]
        pack: Option<String>,
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
            Request::SetAuthoredBatch { .. } => "set_authored_batch",
            Request::Withdraw { .. } => "withdraw",
            Request::Restore { .. } => "restore",
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
            Request::KnownTo { .. } => "known_to",
            Request::SetKnown { .. } => "set_known",
            Request::UndoKnown { .. } => "undo_known",
            Request::KnownIn { .. } => "known_in",
            Request::Atoms { .. } => "atoms",
            Request::LookAhead { .. } => "look_ahead",
            Request::CommitLookAhead { .. } => "commit_look_ahead",
            Request::SetSystemAge { .. } => "set_system_age",
            Request::ImportV3 { .. } => "import_v3",
            Request::RunFormation { .. } => "run_formation",
            Request::ReadWorldStack { .. } => "read_world_stack",
            Request::Weather { .. } => "weather",
            Request::ReadLight { .. } => "read_light",
            Request::ReadNebulae { .. } => "read_nebulae",
            Request::SurveyPlanets { .. } => "survey_planets",
            Request::ReadActivity { .. } => "read_activity",
            Request::ReadField { .. } => "read_field",
            Request::ReadComposition { .. } => "read_composition",
            Request::ReadInventory { .. } => "read_inventory",
            Request::ReadPhotometry { .. } => "read_photometry",
            Request::ReadSpectrum { .. } => "read_spectrum",
            Request::ReadAirOptics { .. } => "read_air_optics",
            Request::LightThroughGas { .. } => "light_through_gas",
            Request::EffectAtRange { .. } => "effect_at_range",
            Request::ReadLimb { .. } => "read_limb",
            Request::FogOfWar { .. } => "fog_of_war",
            Request::ReadBows { .. } => "read_bows",
            Request::ReadScattering { .. } => "read_scattering",
            Request::Detect { .. } => "detect",
            Request::Adopt { .. } => "adopt",
            Request::ResolvedSystem { .. } => "resolved_system",
            Request::SurveySurface { .. } => "survey_surface",
            Request::PlanRoute { .. } => "plan_route",
            Request::ShipAtWorld { .. } => "ship_at_world",
            Request::JumpMap { .. } => "jump_map",
            Request::GalaxyChart { .. } => "galaxy_chart",
            Request::WebSlice { .. } => "web_slice",
            Request::WindowView { .. } => "window_view",
            Request::Sift { .. } => "sift",
            Request::ClassifyPreview { .. } => "classify_preview",
            Request::PlanDescent { .. } => "plan_descent",
            Request::SurveyCaves { .. } => "survey_caves",
            Request::SurfaceFrames { .. } => "surface_frames",
            Request::Systems => "systems",
            Request::LifeView { .. } => "life_view",
            Request::ExportChronicle => "export_chronicle",
            Request::ImportChronicle { .. } => "import_chronicle",
            Request::AddNote { .. } => "add_note",
            Request::SetKept { .. } => "set_kept",
            Request::AddActor { .. } => "add_actor",
            Request::Record { .. } => "record",
            Request::Hold { .. } => "hold",
            Request::WhoHolds { .. } => "who_holds",
            Request::HoldingsIn { .. } => "holdings_in",
            Request::SetLink { .. } => "set_link",
            Request::LinksIn { .. } => "links_in",
            Request::SetTech { .. } => "set_tech",
            Request::TechOf { .. } => "tech_of",
            Request::TechReading { .. } => "tech_reading",
            Request::Disclosures { .. } => "disclosures",
            Request::RulesStep { .. } => "rules_step",
            Request::RulesPending => "rules_pending",
            Request::RulesDecide { .. } => "rules_decide",
            Request::RulesPolicy { .. } => "rules_policy",
            Request::Rules => "rules",
            Request::OrbitsAt { .. } => "orbits_at",
            Request::OrbitStability { .. } => "orbit_stability",
            Request::ShipPower { .. } => "ship_power",
            Request::LogFlight { .. } => "log_flight",
            Request::FlightLog { .. } => "flight_log",
            Request::SetCampaignNow { .. } => "set_campaign_now",
            Request::CampaignNow => "campaign_now",
            Request::Calendar => "calendar",
            Request::Intents => "intents",
            Request::SetSeats { .. } => "set_seats",
            Request::SendIntent { .. } => "send_intent",
            Request::IntentDecide { .. } => "intent_decide",
            Request::Templates => "templates",
            Request::Create { .. } => "create",
            Request::Retire { .. } => "retire",
            Request::SplitConstruct { .. } => "split_construct",
            Request::JoinConstructs { .. } => "join_constructs",
            Request::Programs { .. } => "programs",
            Request::AttachProgram { .. } => "attach_program",
            Request::DetachProgram { .. } => "detach_program",
            Request::ProgramsStep { .. } => "programs_step",
            Request::Codes { .. } => "codes",
            Request::ImportSector { .. } => "import_sector",
            Request::ExportSector { .. } => "export_sector",
            Request::SectorMap { .. } => "sector_map",
            Request::HaloCell { .. } => "halo_cell",
            Request::MatterBudget { .. } => "matter_budget",
            Request::StarCell { .. } => "star_cell",
            Request::SkyFromHere { .. } => "sky_from_here",
            Request::SkyGlow { .. } => "sky_glow",
            Request::PlanVoyage { .. } => "plan_voyage",
            Request::PromoteStar { .. } => "promote_star",
            Request::CampaignTags => "campaign_tags",
            Request::ReadMatter { .. } => "read_matter",
            Request::PhaseAt { .. } => "phase_at",
            Request::ReadReach { .. } => "read_reach",
            Request::Dressing { .. } => "dressing",
            Request::AdoptDressing { .. } => "adopt_dressing",
            Request::HoldDressing { .. } => "hold_dressing",
            Request::NameSample { .. } => "name_sample",
            Request::ReadNames { .. } => "read_names",
            Request::SetNaming { .. } => "set_naming",
            Request::Rename { .. } => "rename",
            Request::FixNames { .. } => "fix_names",
            Request::PackReports { .. } => "pack_reports",
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
        o(
            "set_authored_batch",
            Cost::Light,
            "GM machine only in a session; one event for the whole batch",
        ),
        o(
            "withdraw",
            Cost::Trivial,
            "GM machine only in a session; one authored event, nothing deleted",
        ),
        o(
            "restore",
            Cost::Trivial,
            "GM machine only in a session; one authored event",
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
        o(
            "known_to",
            Cost::Trivial,
            "GM machine only; one scene of one address per audience",
        ),
        o(
            "set_known",
            Cost::Trivial,
            "GM machine only; one grant event over the scope's addresses",
        ),
        o("undo_known", Cost::Trivial, "GM machine only; one grant event"),
        o(
            "known_in",
            Cost::Light,
            "GM machine only; one scene of the bounds per audience",
        ),
        o("atoms", Cost::Trivial, none),
        o(
            "look_ahead",
            Cost::Medium,
            "GM edit: a Solar System run to a cold white dwarf is about 0.2 M work units; weak devices ask the GM machine",
        ),
        o(
            "commit_look_ahead",
            Cost::Trivial,
            "GM machine only in a session; with accept system_age, the look-ahead again plus set_system_age",
        ),
        o(
            "set_system_age",
            Cost::Medium,
            "GM machine only; the campaign's galaxy grown once (about a second native) and a few hundred ring reads",
        ),
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
            "weather",
            Cost::Light,
            "rules and keyed draws only: a 16 x 16 field tile or a report is milliseconds on any device",
        ),
        o(
            "detect",
            Cost::Light,
            "per target and sensor: two band integrals and closed-form noise",
        ),
        o(
            "read_air_optics",
            Cost::Trivial,
            "per air and wavelength: a few closed-form blocks",
        ),
        o(
            "light_through_gas",
            Cost::Light,
            "per passage: a few formula blocks per wavelength per absorption feature, at the eye's 81 wavelengths and any asked for",
        ),
        o(
            "read_bows",
            Cost::Trivial,
            "per item: one or two formula blocks",
        ),
        o(
            "read_scattering",
            Cost::Light,
            "per item: the Mie series, about x + 4x^(1/3) terms for each angle and each of up to nine sizes",
        ),
        o(
            "fog_of_war",
            Cost::Light,
            "per seat, contact and instrument: one light description and one detection",
        ),
        o(
            "read_limb",
            Cost::Trivial,
            "per disc and point: one formula block and a Planck sum at the eye's 81 wavelengths",
        ),
        o(
            "effect_at_range",
            Cost::Light,
            "per ask: a few formula blocks, and four bisections of about eighty steps for the rings",
        ),
        o(
            "read_spectrum",
            Cost::Light,
            "per target: the grid, band integrals and a 400-point peak scan per covered range",
        ),
        o(
            "read_activity",
            Cost::Light,
            "per star: a few block evaluations; flares are one draw per time bin per energy decade, capped at 2000 listed",
        ),
        o(
            "read_field",
            Cost::Trivial,
            "per body: a few closed-form blocks",
        ),
        o(
            "read_composition",
            Cost::Trivial,
            "per body: a few hundred multiplications over the sheet's elements",
        ),
        o(
            "read_inventory",
            Cost::Trivial,
            "per body: read_composition at Survey, then one sum per kind",
        ),
        o(
            "read_photometry",
            Cost::Trivial,
            "per star: one Planck evaluation and one extinction block per band",
        ),
        o(
            "survey_planets",
            Cost::Light,
            "per planet and method: closed-form signal and noise, no integration",
        ),
        o(
            "read_nebulae",
            Cost::Light,
            "per nebula: a few block evaluations; a remnant's fading date is a 200-step bisection",
        ),
        o(
            "read_light",
            Cost::Light,
            "per body: one small flow run and about 2,000 Planck evaluations",
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
            "ship_at_world",
            Cost::Light,
            "per world closed-form passes and burns, plus one descent at 0.1 s steps where there is ground and air (tens of ms native)",
        ),
        o(
            "jump_map",
            Cost::Light,
            "closed form per body: a shell, five points or a gravity level each",
        ),
        o(
            "plan_route",
            Cost::Medium,
            "about a thousand Lambert solves on the departure and flight-time grid plus torch flights; drift asks plan once per body and port (about 0.1 to 1 s native)",
        ),
        o(
            "galaxy_chart",
            Cost::Medium,
            "one galaxy grown over cosmic time plus 24 rings of chemical evolution and a green band every half Gyr (about 0.1 to 0.5 s native)",
        ),
        o(
            "web_slice",
            Cost::Medium,
            "n by n web samples at four octaves (n = 128: about 0.2 s native); a cube is (n + 2)^3 potentials (n = 32: about 0.3 s native)",
        ),
        o(
            "sift",
            Cost::Light,
            "GM machine only; linear in addresses x sifters within the scope (a campaign of a few systems: milliseconds), plus one scene for an audience",
        ),
        o(
            "classify_preview",
            Cost::Light,
            "GM machine only; one atlas load plus one label per body in scope (a campaign of a few systems: milliseconds)",
        ),
        o(
            "window_view",
            Cost::Light,
            "one Kepler solve and one brightness per body (microseconds)",
        ),
        o(
            "plan_descent",
            Cost::Heavy,
            "a survey as survey_surface (cached for the same world), one site read of about 30 heights, and a descent at 0.1 s steps (well under 0.1 s native)",
        ),
        o(
            "surface_frames",
            Cost::Heavy,
            "the plate run recorded at every snapshot, the impacts, and one climate per frame: about 1 to 2 s native at the Orbit level; the Survey grid is four times the cells",
        ),
        o(
            "survey_caves",
            Cost::Heavy,
            "a survey as survey_surface (cached for the same world), then up to 40 steps of 5 height reads per passage near the site (tens of ms native)",
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
        o("add_note", Cost::Light, "one event"),
        o("set_kept", Cost::Light, "one fold and one event"),
        o("add_actor", Cost::Light, "one fold and one event"),
        o("record", Cost::Light, "linear in the record events"),
        o(
            "disclosures",
            Cost::Light,
            "one scene for the audience plus the record's notes",
        ),
        o(
            "rules_step",
            Cost::Light,
            "GM machine only; one fold per pass, conditions linear in watched subjects (pairs: subjects x observers)",
        ),
        o("rules_pending", Cost::Trivial, "GM machine only"),
        o("rules_decide", Cost::Light, "GM machine only; one fold and the proposal's effects"),
        o("rules_policy", Cost::Trivial, "GM machine only; one record event"),
        o("rules", Cost::Light, "GM machine only; linear in the record events"),
        o(
            "orbits_at",
            Cost::Light,
            "linear in bodies x times (at most 64 times); 16 Newton steps each",
        ),
        o("ship_power", Cost::Trivial, "linear in the ship's parts"),
        o("log_flight", Cost::Trivial, "one record event, and one for each construct a split or join touches"),
        o("flight_log", Cost::Light, "linear in the record events; with where_at, one Kepler or Lambert solve"),
        o(
            "orbit_stability",
            Cost::Light,
            "linear in bodies, plus the outer bodies of each (Kozai); 16 Newton steps per neighbour pair",
        ),
        o("set_campaign_now", Cost::Trivial, "one universe event"),
        o("campaign_now", Cost::Trivial, "a scan of the universe events"),
        o("calendar", Cost::Trivial, "read at load"),
        o("intents", Cost::Light, "GM machine only; linear in the record events"),
        o("set_seats", Cost::Trivial, "GM machine only; one record event"),
        o(
            "send_intent",
            Cost::Light,
            "GM machine only; one fold for a validity condition, then the intent's own request",
        ),
        o("intent_decide", Cost::Light, "GM machine only; the intent's own request"),
        o("templates", Cost::Trivial, "read at load"),
        o("create", Cost::Light, "GM machine only; one universe event and one record event per kept value"),
        o("retire", Cost::Trivial, "GM machine only; one universe event"),
        o("split_construct", Cost::Trivial, "GM machine only; one universe event"),
        o("join_constructs", Cost::Trivial, "GM machine only; one universe event"),
        o("programs", Cost::Trivial, "read at load, plus a fold of the record"),
        o("attach_program", Cost::Trivial, "GM machine only; one record event"),
        o("detach_program", Cost::Trivial, "GM machine only; one record event"),
        o(
            "programs_step",
            Cost::Light,
            "GM machine only; per running program (at most the campaign's limit) one scene, a few conditions and one intent, then a rules step",
        ),
        o(
            "codes",
            Cost::Light,
            "GM machine only; one classification per digit per body, rules per body, linear in the system",
        ),
        o(
            "import_sector",
            Cost::Light,
            "GM machine only; one code-sheet plan per world line; with grow, up to 6 pinned formation runs per world (heavy for a whole sector)",
        ),
        o(
            "export_sector",
            Cost::Light,
            "GM machine only; one code reading per mapped system",
        ),
        o(
            "sector_map",
            Cost::Light,
            "GM machine only; one code reading per mapped system",
        ),
        o(
            "halo_cell",
            Cost::Medium,
            "one cell's haloes drawn from the mass function, each placed on the web and given a growth history and mergers (about 0.1 to 1 s native)",
        ),
        o(
            "matter_budget",
            Cost::Medium,
            "one cell's haloes, G3 once per half dex of halo mass, the mass function below the catalogue floor and the cell's environments (about 0.1 to 1 s native)",
        ),
        o(
            "sky_from_here",
            Cost::Medium,
            "grows the campaign's galaxy, walks the count tree inside each class's visibility radius and gives each star found its magnitudes through dust; read only",
        ),
        o(
            "sky_glow",
            Cost::Medium,
            "integrates the light of the undrawn stars and the dust in front along every HEALPix pixel of the sky (nside 32: 12,288 lines of 96 steps); read only",
        ),
        o(
            "star_cell",
            Cost::Medium,
            "grows the campaign's galaxy, then draws one 10 pc cell (tens of systems) with its pins; read only",
        ),
        o(
            "plan_voyage",
            Cost::Medium,
            "between stars: draws the star cells around the start and the destination (up to 125), then searches legs with fuel carried; in a system: one NAV course plan per place per step (about 0.1 to 2 s native); read only",
        ),
        o(
            "campaign_tags",
            Cost::Light,
            "reads the campaign's own tag declarations from the record",
        ),
        o(
            "promote_star",
            Cost::Medium,
            "GM machine only in a session; draws the star once and stores its whole description in the record",
        ),
        o(
            "read_matter",
            Cost::Light,
            "reads the master data sheet loaded with the stack; nothing is worked out but molar mass",
        ),
        o(
            "phase_at",
            Cost::Light,
            "one lookup in a table built at pack-build time (or the familiar three phases)",
        ),
        o(
            "read_reach",
            Cost::Light,
            "reads the blocks' declared reach; nothing is computed",
        ),
        o(
            "dressing",
            Cost::Light,
            "GM machine only; one condition per rule per world in the scope (a system: well under a millisecond each)",
        ),
        o(
            "adopt_dressing",
            Cost::Light,
            "GM machine only; one template creation",
        ),
        o(
            "hold_dressing",
            Cost::Trivial,
            "GM machine only; one record fact",
        ),
        o(
            "name_sample",
            Cost::Light,
            "up to 10,000 keyed draws from one name scheme, each a few table picks; read only",
        ),
        o(
            "read_names",
            Cost::Light,
            "one address's names, drawing its sibling group once per layer; read only",
        ),
        o("set_naming", Cost::Light, "GM machine only; one authored event"),
        o("rename", Cost::Trivial, "GM machine only; one authored event"),
        o(
            "fix_names",
            Cost::Light,
            "GM machine only; draws the names under one address, one event",
        ),
        o(
            "pack_reports",
            Cost::Trivial,
            "reads reports worked out when the stack loaded; read only",
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
    /// MP1c: with constants that differ from ours, the cited law each
    /// rescaled block it rests on used (ESTIMATED). Left out otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub estimated: Option<Vec<String>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EventSummary {
    pub id: String,
    pub kind: String,
    pub occurred_at_s: i64,
    pub evidence: String,
    pub subject: Vec<String>,
    /// The facts this event set or unset at the asked address, by field (only when `history` asked for changes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub changes: Option<Vec<FactChange>>,
}

/// One fact an event set (`value`) or unset (`unset: true`). A value over `FactChange::LARGE` bytes of JSON (a
/// picture) is left out and marked `large`, so a history stays small.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FactChange {
    pub field: String,
    #[ts(type = "unknown")]
    #[serde(default)]
    pub value: serde_json::Value,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(type = "boolean | undefined")]
    pub unset: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(type = "boolean | undefined")]
    pub large: bool,
}

impl FactChange {
    pub const LARGE: usize = 4096;
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
        /// A freely typed tag (`tag.<name>`) the stack did not declare: it
        /// joined the campaign's own tags (TG1).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        declared_tag: Option<String>,
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
        /// `accept: campaign_now`: the campaign's new now, seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        campaign_now_s: Option<i64>,
        /// `accept: system_age`: what `set_system_age` answers.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        system_age: Option<serde_json::Value>,
    },
    /// `set_system_age`: ages before and after (Myr), the system's age shift,
    /// the authored event, the origin (`origin`: label, birth_gyr, age_gyr,
    /// birth_r_kpc, birth_phi_deg, now_r_kpc, migrated_kpc, sigma_kpc,
    /// feh_gas, feh, composition, unusual?, impossible?, path [[t_gyr, r_kpc,
    /// phi_deg]], why, references) and its event, `impossible` when no origin
    /// fits, and the composition change.
    SystemAgeSet {
        system: String,
        before_myr: f64,
        age_myr: f64,
        shift_myr: f64,
        event_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        origin_event: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        origin: Option<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        impossible: Option<String>,
        /// `{mode, feh_before, feh_after, z_before, z_after, rederive}`.
        #[ts(type = "unknown")]
        composition: serde_json::Value,
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
    /// `known_to`: `value` is `{address, at_s, audiences: [{audience, label,
    /// rung, rung_label, landmark, top, exceptions: [{field, landmark}],
    /// cover: [{field, told}], withheld: [fields not sent with a value], from_s?}]}`.
    KnownTo {
        #[ts(type = "unknown")]
        value: serde_json::Value,
    },
    /// `set_known` and `undo_known`: the one event written (`undo` is its id;
    /// absent when nothing changed) and how many things it changed.
    KnownSet {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        undo: Option<String>,
        changed: u32,
        state_hash: String,
    },
    /// `known_in`: per audience `{audience, label, known, heard_of, gm_only}`.
    KnownCounted {
        bounds: String,
        #[ts(type = "unknown[]")]
        audiences: Vec<serde_json::Value>,
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
    /// `weather`: the data asked for (`want`), the world's label, and which
    /// block made it (`provenance`: block id, version, level, evidence).
    WeatherRead {
        want: String,
        world: String,
        #[ts(type = "unknown")]
        provenance: serde_json::Value,
        #[ts(type = "unknown")]
        data: serde_json::Value,
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
    /// `read_light`: `source` is `fixture:<name>` or `given`; `table` is the
    /// band set, visible slices and line table in force; each of `bodies` is
    /// `sse_light::emit::BodyLight` (address, name, facts, light: description,
    /// bands, visible_slices, total_w, samples, peak_m).
    LightRead {
        source: String,
        #[ts(type = "unknown")]
        table: serde_json::Value,
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
    },
    /// `detect`: each of `detections` is `sse_sense::Detection` (target by
    /// sensor, in order); `radar` is `sse_sense::RadarRanges` when asked.
    Detected {
        #[ts(type = "unknown[]")]
        detections: Vec<serde_json::Value>,
        #[ts(optional, type = "unknown")]
        radar: Option<serde_json::Value>,
    },
    /// `read_air_optics`: each of `airs` is `sse_light::air::AirOptics`.
    AirOpticsRead {
        #[ts(type = "unknown[]")]
        airs: Vec<serde_json::Value>,
    },
    /// `light_through_gas`: each of `passages` is `{ passed, colour_in,
    /// colour_out }` with `sse_light::gas::Passed` and
    /// `sse_light::colour::Colour`.
    GasPassed {
        #[ts(type = "unknown[]")]
        passages: Vec<serde_json::Value>,
    },
    /// `read_bows`: each of `items` is `sse_light::bows::Bows`.
    BowsRead {
        #[ts(type = "unknown[]")]
        items: Vec<serde_json::Value>,
    },
    /// `read_scattering`: each of `items` is `sse_light::scatter::Scattering`.
    ScatteringRead {
        #[ts(type = "unknown[]")]
        items: Vec<serde_json::Value>,
    },
    /// `fog_of_war`: each of `seats` is `{seat, sensors, noticed_to_m?,
    /// marks}` with `sse_sense::fog::Mark`.
    FogRead {
        #[ts(type = "unknown[]")]
        seats: Vec<serde_json::Value>,
    },
    /// `read_limb`: each of `discs` is `{ points, references }` with
    /// `sse_light::limb::LimbPoint`.
    LimbRead {
        #[ts(type = "unknown[]")]
        discs: Vec<serde_json::Value>,
    },
    /// `effect_at_range`: each of `effects` is `sse_sense::reach::Effect`.
    EffectsAtRange {
        #[ts(type = "unknown[]")]
        effects: Vec<serde_json::Value>,
    },
    /// `read_spectrum`: each of `spectra` is `sse_light::observe::Spectrum`.
    SpectrumRead {
        #[ts(type = "unknown[]")]
        spectra: Vec<serde_json::Value>,
    },
    /// `read_activity`: each of `stars` is `{ activity, moment, flares? }`
    /// with `sse_activity::Activity`, `Moment` and `Flares`.
    ActivityRead {
        #[ts(type = "unknown[]")]
        stars: Vec<serde_json::Value>,
    },
    /// `read_field`: each of `bodies` is `sse_field::FieldRead` (obstacle
    /// candidates and winner, flaring, host bubble, induced layers, why,
    /// stand-ins, references).
    FieldRead {
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
    },
    /// `read_composition`: each of `bodies` is `sse_surface::composition::Composition`
    /// (bulk materials, separation strength, elements heaviest first with
    /// reservoirs at Survey, stand-ins).
    CompositionRead {
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
    },
    /// `read_inventory`: each of `bodies` is a world's address, mass and
    /// kinds (id, matter, reach, kg, per_kg, by_reservoir, note), with
    /// stand-ins.
    InventoryRead {
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
    },
    /// `read_photometry`: each of `stars` is `sse_light::photometry::Photometry`
    /// (magnitudes per band with extinction, colours with excess); `references`
    /// cite the magnitude system, band table, extinction curve and Planck's law.
    PhotometryRead {
        #[ts(type = "unknown[]")]
        stars: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        references: Vec<serde_json::Value>,
    },
    /// `survey_planets`: `result` is `sse_survey::SurveyResult` (findings
    /// per planet and method with verdict, signal to noise, measured values
    /// and why; the addresses found; references).
    PlanetsSurveyed {
        #[ts(type = "unknown")]
        result: serde_json::Value,
    },
    /// `read_nebulae`: each of `nebulae` is `sse_nebula::Descriptor`;
    /// `absent` lists the addresses asked about that do not exist at `at_s`.
    NebulaeRead {
        at_s: f64,
        #[ts(type = "unknown[]")]
        nebulae: Vec<serde_json::Value>,
        absent: Vec<String>,
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
    /// `galaxy_chart`: `{halo, key, halo_mass_msun, morphology, barred,
    /// t_start_gyr, now_gyr, at_gyr, now, history, active_nuclei,
    /// sun_radius_kpc, rings, core_collapse_per_century, ia_per_century,
    /// green_bands, epochs, place, rotation_curve {structure, points, at_sun,
    /// dark_density_at_sun_msun_pc3, v_max_kms, r_v_max_kpc}, model}`.
    GalaxyCharted {
        #[ts(type = "unknown")]
        chart: serde_json::Value,
    },
    /// `web_slice`: `{t_gyr, z, now_gyr, slice, model}` (`slice` is sse-cosmos's
    /// `Slice`), or with `cube` `{t_gyr, z, now_gyr, volume, model}` (`volume`
    /// is sse-cosmos's `Volume`), or with `haloes` too `{t_gyr, z, now_gyr,
    /// centre, extent, galaxies, galaxy_floor_msun, model}` (`galaxies` is a
    /// list of sse-cosmos's `WebGalaxy`).
    WebSliced {
        #[ts(type = "unknown")]
        web: serde_json::Value,
    },
    /// `window_view`: sse-nav's `WindowView` (`seen`: lon/lat, distance,
    /// angular diameter, magnitude, phase, lit fraction, detected, resolved,
    /// light time, why; `sensor`; `missed`; `model`), plus `frame` (as
    /// `resolved_system`'s; null for an inline system with no `system` or
    /// no open universe).
    /// `sift`: hits best first (`sifter`, `label`, `subject`, `evidence`
    /// [{address, field, value, event}], `significance` {score, persistence_s,
    /// reach, reversibility, rarity_p, components}), the sifters run, the scope.
    /// With an `audience`, `skipped` names the sifters not run for it.
    Sifted {
        scope: String,
        sifters: Vec<String>,
        #[ts(type = "unknown")]
        hits: serde_json::Value,
        #[ts(optional)]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        audience: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        skipped: Vec<String>,
    },
    WindowViewed {
        #[ts(type = "unknown")]
        view: serde_json::Value,
    },
    /// `plan_route`: sse-nav's `RoutePlan` (`candidates` best first by the
    /// preference, each with burns, days, Δv, propellant and reserve, peak g,
    /// dose, solar-storm chance, peak flux, delays, hazards, difficulty with
    /// reasons and a track; `refused` with why; `drift` for ways out).
    RoutePlanned {
        #[ts(type = "unknown")]
        plan: serde_json::Value,
    },
    /// `ship_at_world`: `{system, ship, worlds (body, name, gravity_m_s2,
    /// orbit_m_s, escape_m_s, v_inf_m_s, arrival, air?, checks (id, words,
    /// verdict safe|risky|dangerous|not-here, figures, dangers (id, subject,
    /// role, figure, limit, unit, ratio, level, chance {p, basis}, if_wrong,
    /// effects, why), would_pass, why)), model}`.
    ShipAtWorldRead {
        #[ts(type = "unknown")]
        report: serde_json::Value,
    },
    /// `jump_map`: `report {system, rule (the FTL rule as read), map {t_s, keepouts
    /// (body, name, centre_m, radius_m), points (body, name, what, at_m,
    /// within_m), gates, gravity ([m_s2, above]), anywhere, words}}`; positions
    /// are metres from the star in NAV's frame.
    JumpMapped {
        #[ts(type = "unknown")]
        report: serde_json::Value,
    },
    /// `surface_frames`: `{body, label, source, at_s?, frames (version, detail,
    /// epoch_myr, grid {n, cells}, dirs (x y z per cell), frames (t_myr,
    /// sea_level_m, plate, continental, elevation_m, vent, arc, ridge, ice_m, temp_k,
    /// rain_m_yr, runoff_m_yr, why), events (t_myr, kind, text, dir?,
    /// plates?, radius_km?, diameter_km?, why), pending, why)}`.
    FramesMade {
        #[ts(type = "unknown")]
        frames: serde_json::Value,
    },
    /// `survey_caves`: `{body, label, source, at_s?, caves (version, lat_deg,
    /// lon_deg, radius_km, level, spacing_m, fields, passages (id, field,
    /// length_km, width_m, roof_m, depth_m, nodes (lat, lon, x_m, y_m,
    /// ground_m, floor_m, entrance?), segments (from, to, length_m, width_m,
    /// height_m, roofed, open?), entrances, speculative, cause), why)}`.
    CavesSurveyed {
        #[ts(type = "unknown")]
        caves: serde_json::Value,
    },
    /// `plan_descent`: `{body, label, source, at_s?, lander, entry, site
    /// (level, spacing, height, slope, local slope, roughness, relief,
    /// under_liquid, ok, why), air_at_site, gravity_m_s2, hazards (kind,
    /// level, why), verdict, descent (outcome, events, telemetry)}`.
    DescentPlanned {
        #[ts(type = "unknown")]
        descent: serde_json::Value,
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
    /// `add_note`: the note's event and who it is for (send the event to
    /// ordinary seats of these audiences only; the GM sees every note).
    NoteAdded {
        note_id: String,
        #[ts(type = "\"everyone\" | \"gm\" | string[]")]
        audience: serde_json::Value,
        /// The universe state hash: record activity never changes it.
        state_hash: String,
        /// The record's own hash (GM only).
        record_hash: String,
    },
    /// `set_kept`: the change's id, the value before (null: none) and both hashes.
    KeptSet {
        change_id: String,
        #[ts(type = "unknown")]
        previous: serde_json::Value,
        state_hash: String,
        record_hash: String,
    },
    ActorAdded {
        address: String,
        state_hash: String,
        record_hash: String,
    },
    /// `record`: notes `{id, subjects, at_s, text, tags?, by, audience,
    /// tier?, supersedes?, narrative?}`, kept values `{kind, value, kept_by?,
    /// orphan?, describes?, unit?, history: [{event, at_s, value, writer,
    /// reason?, delta?}]}` and actors `{address, name, kind}`.
    RecordRead {
        address: String,
        #[ts(type = "unknown[]")]
        notes: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        kept: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        actors: Vec<serde_json::Value>,
    },
    /// `hold`: the holding as stored.
    Held {
        holding_id: String,
        #[ts(type = "unknown")]
        holding: serde_json::Value,
        record_hash: String,
    },
    /// `who_holds`: the answer, or null for bare paper.
    WhoHeld {
        #[ts(type = "unknown")]
        answer: serde_json::Value,
    },
    /// `holdings_in`: `holdings` are answers as `who_holds` gives them;
    /// `changes` (with `history`) every holding there by date.
    HoldingsListed {
        layer: String,
        #[ts(type = "unknown[]")]
        holdings: Vec<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(optional, type = "unknown[]")]
        changes: Vec<serde_json::Value>,
    },
    /// `set_link`: the link as stored.
    LinkSet {
        link: String,
        event_id: String,
        #[ts(type = "unknown")]
        record: serde_json::Value,
        record_hash: String,
    },
    /// `links_in`: each link with its travel part (`nav`) as an object.
    LinksListed {
        #[ts(type = "unknown[]")]
        links: Vec<serde_json::Value>,
    },
    /// `set_tech`: the event's id.
    TechSet {
        change_id: String,
        record_hash: String,
    },
    /// `tech_of`: `{level?: [n, said], gained, lost, holds: [advance ids]}`.
    TechRead {
        holder: String,
        #[ts(type = "unknown")]
        tech: serde_json::Value,
    },
    /// `tech_reading`: the availability reading.
    TechAvailability {
        holder: String,
        #[ts(type = "unknown")]
        reading: serde_json::Value,
    },
    /// `disclosures`: `items` are `{subject, notes?: [{id, at_s, text, tags?,
    /// by, links?}], kept?: [{field, value?, band?}]}`, where `kept` holds the
    /// record's facts the view shows (kept values, an actor's name and kind);
    /// `stream_hash` lets the seat check it received this exactly.
    Disclosed {
        audience: String,
        at_s: i64,
        #[ts(type = "unknown[]")]
        items: Vec<serde_json::Value>,
        stream_hash: String,
    },
    /// `rules_step`: `report` is `{at_s, passes: [[{rule, subject, observer?,
    /// cause?, fired, outcome, written: [{chronicle, event, effect}]}]], limit?}`.
    RulesStepped {
        #[ts(type = "unknown")]
        report: serde_json::Value,
        state_hash: String,
        record_hash: String,
    },
    /// `rules_pending`: `[{id, rule, subject, observer?, at_s, effects, explain}]`.
    RulesPending {
        #[ts(type = "unknown[]")]
        proposals: Vec<serde_json::Value>,
    },
    /// `rules_decide`: what the yes wrote (empty for a no).
    RulesDecided {
        #[ts(type = "unknown[]")]
        written: Vec<serde_json::Value>,
        state_hash: String,
        record_hash: String,
    },
    RulesPolicySet {
        event_id: String,
        record_hash: String,
    },
    /// `rules`: each rule `{id, pack, phase, when, effects, approval, explain}`, and the trail.
    RulesListed {
        #[ts(type = "unknown[]")]
        rules: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        trail: Vec<serde_json::Value>,
    },
    /// `intents`: `kinds` `[{kind, meaning, fields, subject?, valid?, never_auto?, pack, applied_by}]`,
    /// `seats` (the plan in force) and `pending` `[{id, seat, intent, at_s}]`.
    IntentsListed {
        #[ts(type = "unknown[]")]
        kinds: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        seats: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        pending: Vec<serde_json::Value>,
    },
    SeatsSet {
        event_id: String,
        record_hash: String,
    },
    /// `send_intent`: `decision` `{action: apply | ask_gm | refuse, reason?}`;
    /// `applied_by` `kernel` (with the request's response in `applied`),
    /// `record` (`intent.<kind>` written) or `session` (the session applies
    /// it); `proposal` when it waits for the GM.
    IntentSent {
        #[ts(type = "unknown")]
        decision: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        applied_by: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        proposal: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        applied: Option<serde_json::Value>,
    },
    /// `intent_decide`: the answer's event, the intent, and what a yes applied.
    IntentDecided {
        event_id: String,
        seat: String,
        #[ts(type = "unknown")]
        intent: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        applied_by: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        applied: Option<serde_json::Value>,
    },
    /// `templates`: `[{id, kind, label, facts?, kept?, tags?, programs?, explain?, pack, version}]`.
    TemplatesListed {
        #[ts(type = "unknown[]")]
        templates: Vec<serde_json::Value>,
    },
    /// `create`: the new address, its universe event and the kept values' record events.
    Created {
        address: String,
        event_id: String,
        kept: Vec<String>,
        state_hash: String,
        record_hash: String,
    },
    Retired {
        event_id: String,
        state_hash: String,
    },
    /// `split_construct`: the new construct's address.
    ConstructSplit {
        address: String,
        event_id: String,
        state_hash: String,
    },
    /// `join_constructs`.
    ConstructsJoined {
        how: String,
        event_id: String,
        state_hash: String,
    },
    /// `programs`: `programs: [{id, label, params, start, states, budget, explain, pack}]`,
    /// `limit`, and `running: [{address, program, seat, state, since_s}]`.
    ProgramsListed {
        #[ts(type = "unknown[]")]
        programs: Vec<serde_json::Value>,
        limit: u32,
        #[ts(type = "unknown[]")]
        running: Vec<serde_json::Value>,
    },
    ProgramAttached {
        event_id: String,
        record_hash: String,
    },
    ProgramDetached {
        event_id: String,
        record_hash: String,
    },
    /// `programs_step`: per program `{address, program, from, to, moves,
    /// sensed, picked?, intent?, decision?, applied_by?, proposal?, skipped?}`,
    /// then the rules step's report.
    ProgramsStepped {
        #[ts(type = "unknown[]")]
        programs: Vec<serde_json::Value>,
        #[ts(type = "unknown")]
        rules: serde_json::Value,
        record_hash: String,
    },
    /// `orbits_at`: per body `{address, placed, parent?, period_s?,
    /// phase_source?, missing?, at: [{at_s, r_m: [x, y, z], phase_deg}]}`.
    OrbitsPlaced {
        system: String,
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
    },
    /// `ship_power`: `{ship, flux_w_m2, accel_g, exhaust_km_s, ledger,
    /// balance, fictional, why}`.
    ShipPowerIs {
        #[ts(type = "unknown")]
        power: serde_json::Value,
    },
    /// `log_flight`: the entry's id, the entries a split or join seeded in
    /// other constructs' logs, and notes (a split on a course seeds nothing).
    FlightLogged {
        event_id: String,
        seeded: Vec<String>,
        notes: Vec<String>,
        record_hash: String,
    },
    /// `flight_log`: the assembled log (sse-nav `FlightLog`), its entries as
    /// written (`{event, entry, written_s, writer, body, also?, reason?}`),
    /// and with `where_at`, where the craft was (`{kind, r_m?, v_m_s?, with?,
    /// mark, words}`).
    FlightLogIs {
        #[ts(type = "unknown")]
        log: serde_json::Value,
        #[ts(type = "unknown[]")]
        entries: Vec<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        at: Option<serde_json::Value>,
    },
    /// `orbit_stability`: `verdict` (`lasts`, `watch` or `unstable`, the
    /// worst check), `checks` (`{check, block, around, bodies, level, says,
    /// inputs, results}`; level 0 lasts, 1 watch, 2 will not last, 3 the
    /// orbits cross) and `unchecked` (`{check, bodies, missing}`).
    OrbitStabilityIs {
        system: String,
        verdict: String,
        #[ts(type = "unknown[]")]
        checks: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        unchecked: Vec<serde_json::Value>,
    },
    /// `set_campaign_now` and `campaign_now`.
    CampaignNowIs {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        at_s: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        event_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        state_hash: Option<String>,
    },
    /// `calendar`: `{name, epoch_s, epoch_year, day_s, year_days, months?, format}`, or absent.
    CalendarIs {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "unknown")]
        calendar: Option<serde_json::Value>,
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
    /// `clouds`: a formation run's leftovers (F5), each `{address, kind,
    /// around, source, a_min_au, a_max_au, largest_km, over_1km, shape,
    /// i_sigma_deg}` (`shape` `disc` with a Rayleigh tilt spread, or
    /// `sphere` for isotropic tilts with `i_sigma_deg` null); `around`
    /// is null for a cloud that orbits the star (belts, scattered disc,
    /// Oort), otherwise the world it orbits, with axes in AU around it.
    /// Absent when there are none.
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(optional, type = "unknown[]")]
        clouds: Vec<serde_json::Value>,
        /// The system's frame in the galaxy: `{pole_l_deg, pole_b_deg,
        /// zero_l_deg, zero_b_deg, node_l_deg, to_galactic (3x3 rows),
        /// evidence ("drawn"), source, model}`.
        #[ts(type = "unknown")]
        frame: serde_json::Value,
    },
    /// `classify_preview`: one row per body in address order, `{address,
    /// base, fit, modifiers}` (`base` null when nothing fits), with the
    /// candidate atlas's hash and the scheme's class examples it gets wrong.
    ClassifyPreviewed {
        scheme: String,
        atlas_hash: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        at_s: Option<i64>,
        #[ts(type = "unknown[]")]
        bodies: Vec<serde_json::Value>,
        unclassified: u32,
        failing_examples: Vec<String>,
    },
    /// `codes`: one entry per body, in address order:
    /// `{address, reading}` with the sse-codes `Reading` unchanged in shape.
    Codes {
        sheet: String,
        #[ts(type = "unknown[]")]
        readings: Vec<serde_json::Value>,
    },
    /// `import_sector`: the sse-codes `Import` (worlds, plans, what was
    /// impossible, guessed or differs, a summary) and, when applied, the
    /// events written.
    SectorImport {
        #[ts(type = "unknown")]
        import: serde_json::Value,
        summary: String,
        events: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        state_hash: Option<String>,
    },
    /// `export_sector`: the file's text, the systems written, and those
    /// skipped (no main world found), by address.
    SectorExport {
        text: String,
        written: u32,
        skipped: Vec<String>,
    },
    /// `sector_map`: `{system, body, hex, name, sector, reading}` per mapped
    /// system, by hex, and the systems skipped (no main world).
    SectorMapped {
        sheet: String,
        #[ts(type = "unknown[]")]
        worlds: Vec<serde_json::Value>,
        skipped: Vec<String>,
    },
    Error {
        code: String,
        message: String,
    },
    /// `halo_cell`: `{address, index, side_mpc, origin_mpc, delta_l, mass_msun,
    /// halo_mass_msun, total, above_min, haloes: [{address, key, mass_msun,
    /// position_mpc, half_mass_gyr, spin, mergers, major_mergers, last_major_gyr,
    /// neighbour}], bins, now_gyr, model}`.
    HaloCelled {
        #[ts(type = "unknown")]
        cell: serde_json::Value,
    },
    /// `matter_budget`: `{cell, cell_mpc, matter_msun, dark {catalogued_haloes,
    /// haloes_below_floor, never_collapsed}, ordinary {stars, cold_gas, hot_gas,
    /// thrown_out, black_holes, between_galaxies}, baryon_share, f_b,
    /// floor_low_msun, catalogue_floor_msun, haloes, environments: [{environment,
    /// volume, matter}], samples_a_side, model}`, solar masses.
    MatterBudgeted {
        #[ts(type = "unknown")]
        budget: serde_json::Value,
    },
    /// `star_cell`: `{galaxy, cell, frame, frame_start_myr, t_myr, cell_pc, expected, pins,
    /// systems: [{address, pinned, drawn_in {cell, frame, slot}, mass_msun,
    /// companions, birth_gyr, feh, xyz_pc, state}], model}`.
    StarCelled {
        #[ts(type = "unknown")]
        cell: serde_json::Value,
    },
    /// `sky_from_here`: `{galaxy, observer {address?, xyz_pc}, t_myr, frame,
    /// limit_mag, band, bands, cone?, stars: [{address, pinned, class,
    /// distance_pc, light_time_yr, l_deg, b_deg, dir, a_v, teff_k,
    /// luminosity_lsun, phase, double, components: [{teff_k, radius_m}],
    /// magnitudes, colours}], counts: [{mag,
    /// n}], walked: {classes: [{class, top_lsun, radius_pc, found}], reads},
    /// truncated, conventions}`.
    Sky {
        #[ts(type = "unknown")]
        sky: serde_json::Value,
    },
    /// `sky_glow`: `{nside, order, frame, limit_mag, limit_band,
    /// observer_pc, t_myr, radii_pc, bands: [{band, intensity, mag_arcsec2,
    /// whole_sky_ab, whole_sky_mag}], a_v, units}`.
    SkyGlowed {
        #[ts(type = "unknown")]
        glow: serde_json::Value,
    },
    /// `plan_voyage`: `{ship, means, searched, cells, t_myr, systems: [{address,
    /// name, xyz_pc, fuel, surveyed, visited}], voyage: {from, to, minimise,
    /// legs: [{from, to, means, distance_pc, counted_pc?, home_days, ship_days,
    /// jump_fuel_t, propellant_t, peak_beta?, coast, fictional, gamble,
    /// refuel?, jump_fuel_left_t, propellant_left_t, words}], distance_pc,
    /// home_days, ship_days, jump_fuel_t, propellant_t, refuels, gambles,
    /// fictional, needs?} | null, refusals: [text], model}`; inside a system
    /// `{scale: "system", system, ship, depart_s, places: [{address, name,
    /// a_au, fuel, belt, visited}], voyage | null, refusals, model}`, its legs
    /// with `means: "course"`, `course` and `wait_days`.
    VoyagePlanned {
        #[ts(type = "unknown")]
        voyage: serde_json::Value,
    },
    /// `campaign_tags`: `{id: "campaign", tags: {groups, tags}}`.
    CampaignTagsListed {
        #[ts(type = "unknown")]
        pack: serde_json::Value,
    },
    /// `dressing`: candidates `[{id, world, rule, pack, template, label,
    /// where, explain, why: [{field, value}], tags?, faction?}]` and orphans
    /// `[{address, pack, rule, touched}]`.
    Dressed {
        scope: String,
        #[ts(type = "unknown[]")]
        candidates: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        orphans: Vec<serde_json::Value>,
    },
    /// `read_matter`: a row (`{table, id, row, sources, packs, worked?,
    /// used_by, skipped_by}`) or a listing of ids by table.
    Matter {
        #[ts(type = "unknown")]
        value: serde_json::Value,
    },
    /// `name_sample`: `{scheme, kind, layer, count, digest, names: [...],
    /// avoid_hits}`, each name with its form, faces and parts.
    /// `read_names`: `{address, kind, names: [{layer, text, source, scheme,
    /// named, original}]}`.
    /// `pack_reports`: `[{pack, version, licence, sources, fudges: [{id, what,
    /// stands_in_for, replace_with?, flows, source?, from, number?}],
    /// reproduction: [{name, source, where?, flow, pass, rows: [{kind, unit?,
    /// published, got, tolerance, off, pass, error?}]}], reproduced}]`.
    PackReports {
        #[ts(type = "unknown")]
        reports: serde_json::Value,
    },
    Names {
        #[ts(type = "unknown")]
        value: serde_json::Value,
    },
    /// `read_reach`: the constant, its real and in-force values, the value
    /// proposed, and the three groups of blocks.
    Reach {
        constant: String,
        real: f64,
        in_force: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        proposed: Option<f64>,
        #[ts(type = "unknown[]")]
        follows: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        estimated: Vec<serde_json::Value>,
        #[ts(type = "unknown[]")]
        cannot_follow: Vec<serde_json::Value>,
    },
    /// `hold_dressing`: the candidate and whether it is now held off.
    DressingHeld {
        candidate: String,
        held: bool,
        event_id: String,
    },
    /// `promote_star`: the stored description, the event that stored it
    /// (absent when the star was already pinned) and the state hash.
    StarPromoted {
        address: String,
        already: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        event_id: Option<String>,
        #[ts(type = "unknown")]
        description: serde_json::Value,
        state_hash: String,
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

/// A point drawn on a place, in that place's frame (OV3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HoldPoint {
    pub frame: String,
    pub xy: [f64; 2],
}
