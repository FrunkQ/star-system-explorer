//! The suite socket (H11 `SuiteAdapter`, L0 interfaces).

use serde::{Deserialize, Serialize};
use sse_chronicle::UniverseTime;
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash, Dimension, Seed};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DeterminismClass {
    /// Bit-exact everywhere: any device may re-run it.
    A,
    /// Not proven bit-exact: runs once on the GM machine, results travel as data.
    B,
}

/// One typed input the suite needs from the levels above (CTR-PROVIDER-REQUIREMENT).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Requirement {
    pub field: String,
    pub dim: Dimension,
    #[serde(default = "yes")]
    pub required: bool,
    #[serde(default)]
    pub description: String,
}
fn yes() -> bool {
    true
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SuiteManifest {
    /// Lowercase id; event kinds are `suite.<id>.<kind>`.
    pub id: String,
    pub version: String,
    pub determinism_class: DeterminismClass,
    pub requirement_profile: Vec<Requirement>,
    /// Fields the suite owns and sets outright (its living state).
    #[serde(default)]
    pub owns: Vec<String>,
    /// Physical fields its fluxes may change (an atmosphere's oxygen).
    #[serde(default)]
    pub may_flux: Vec<String>,
}

/// What the kernel hands a suite for one address and one interval: pinned
/// and content-hashed (CTR-ENV shape).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct InputDataset {
    pub address: Address,
    pub from: UniverseTime,
    pub to: UniverseTime,
    pub values: BTreeMap<String, Value>,
    /// The suite's own state fields as last committed.
    pub own: BTreeMap<String, Value>,
}
impl InputDataset {
    pub fn hash(&self) -> ContentHash {
        ContentHash::of(self).expect("inputs hold no NaN")
    }
}

/// An additive change to a numeric physical field.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Flux {
    pub address: Address,
    pub field: String,
    pub delta: f64,
}

/// A suite-defined event (kind is prefixed `suite.<id>.` on commit).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SuiteEvent {
    pub kind: String,
    pub subject: Address,
    #[serde(default)]
    pub set: BTreeMap<String, Value>,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AdvanceOutput {
    /// New values of fields the suite owns, per address.
    pub own: BTreeMap<Address, BTreeMap<String, Value>>,
    pub fluxes: Vec<Flux>,
    pub events: Vec<SuiteEvent>,
    /// What an observer could detect (feeds data views: a biosignature at Scanned).
    pub signatures: BTreeMap<Address, BTreeMap<String, Value>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SuiteError(pub String);
impl fmt::Display for SuiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The socket. `resolve`, `query_feasible`, `explain` and `sift` have the
/// kernel's shapes; suites that do not offer them keep the defaults.
pub trait SuiteAdapter {
    fn manifest(&self) -> &SuiteManifest;
    /// Inputs for one address up to the next boundary. Called for every
    /// address in scope, in address order, before `advance`.
    fn provide(&mut self, seed: &Seed, input: &InputDataset) -> Result<(), SuiteError>;
    /// Advance everything provided to `to`.
    fn advance(&mut self, to: UniverseTime) -> Result<AdvanceOutput, SuiteError>;
    /// The suite's internal state, for checkpoints (canonical data).
    fn checkpoint(&self) -> Value {
        Value::Null
    }
    fn restore(&mut self, _state: &Value) -> Result<(), SuiteError> {
        Ok(())
    }
    fn explain(&self, _address: &Address, _field: &str) -> Option<String> {
        None
    }
}

/// The authoritative physics provider at boundaries: sets physical fields
/// for the interval (stellar brightening, cooling). Owns physical truth.
pub trait Physics {
    fn step(
        &mut self,
        state: &sse_chronicle::State,
        scope: &Address,
        from: UniverseTime,
        to: UniverseTime,
    ) -> BTreeMap<Address, BTreeMap<String, Value>>;
}

/// The empty suite of stage L0: installed, takes inputs, returns nothing,
/// and says so when asked why.
pub struct NullSuite {
    pub manifest: SuiteManifest,
}
impl NullSuite {
    pub fn life() -> Self {
        NullSuite {
            manifest: SuiteManifest {
                id: "life".into(),
                version: "0.0.0".into(),
                determinism_class: DeterminismClass::A,
                requirement_profile: vec![],
                owns: vec![],
                may_flux: vec![],
            },
        }
    }
}
impl SuiteAdapter for NullSuite {
    fn manifest(&self) -> &SuiteManifest {
        &self.manifest
    }
    fn provide(&mut self, _: &Seed, _: &InputDataset) -> Result<(), SuiteError> {
        Ok(())
    }
    fn advance(&mut self, _: UniverseTime) -> Result<AdvanceOutput, SuiteError> {
        Ok(AdvanceOutput::default())
    }
    fn explain(&self, _: &Address, _: &str) -> Option<String> {
        Some("no life model installed".into())
    }
}
