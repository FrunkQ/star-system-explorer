//! The life suite's sockets (life design section 14 stage L0; handoff 11A).
//!
//! The interfaces here are fixed from L0 and do not change as the insides
//! grow, so a procedural box can later be swapped for a simulated one:
//! - [`LifeRequirementProfile`]: what the suite is told about one habitat.
//! - [`LifeEnvelope`]: the cloud of possibilities for a habitat, region or
//!   world at an epoch.
//! - [`LifeSuite::resolve`]: an envelope to aggregate guilds and relations.
//! - `advance`: the K10 [`sse_suite::SuiteAdapter`] socket.
//! - [`LifeSuite::query_feasible`] and [`LifeSuite::explain`].
//!
//! Habitat graphs ([`habitat`]) are derived from P4 world stacks by
//! pack rules. The only suite at L0 is [`NoLifeModel`], which answers
//! "no life model installed" everywhere.
#![forbid(unsafe_code)]

pub mod budget;
pub mod habitat;
pub mod history;
pub mod scenario;
mod suite;

pub use habitat::{Footprint, Habitat, HabitatGraph, HabitatRule, Link};
pub use suite::{NoLifeModel, NO_MODEL};

use serde::{Deserialize, Serialize};
use sse_chronicle::UniverseTime;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

/// A closed range of a quantity in the field's stated unit.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Range {
    pub lo: f64,
    pub hi: f64,
}

/// Free-energy supply by channel, W/m² at the habitat. `None` means the
/// physics suite has not supplied it yet, never zero.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct EnergyChannels {
    /// Channels the habitat can in principle draw on (`light`, `chemical`,
    /// `oxidant`, `disequilibrium`), set by the habitat rule and the stack.
    pub available: Vec<String>,
    pub light_w_m2: Option<f64>,
    /// Peak of the light reaching the habitat, nm.
    pub light_peak_nm: Option<f64>,
    pub chemical_w_m2: Option<f64>,
    pub tidal_w_m2: Option<f64>,
    pub radiogenic_w_m2: Option<f64>,
}

/// What the life suite needs per habitat (handoff 11A), typed with units
/// in the field names. Every value the stack did not supply is `None` and
/// listed in `missing`, so nothing downstream mistakes a gap for a zero.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeRequirementProfile {
    /// The habitat's address (`<object>/habitat:@<kind>`).
    pub habitat: Address,
    pub kind: String,
    /// `liquid`, `surface`, `subsurface`, `cloud` or a pack medium.
    pub medium: String,
    /// `liquid`, `solid`, `gas`, or a mixture such as `solid+liquid`.
    pub phase: String,
    pub solvent: String,
    pub temperature_k: Option<Range>,
    /// Typical swing over a day or season, K.
    pub temperature_swing_k: Option<f64>,
    pub pressure_pa: Option<Range>,
    pub energy: EnergyChannels,
    /// Material reservoirs, by element or compound, in the stack's units.
    pub materials: BTreeMap<String, f64>,
    pub area_m2: Option<f64>,
    pub volume_m3: Option<f64>,
    /// How long conditions have held, seconds.
    pub persistence_s: Option<f64>,
    pub radiation_dose_msv_yr: Option<f64>,
    pub shielding_kg_m2: Option<f64>,
    /// Other numbers from the stack object (salinity, acidity...).
    pub properties: BTreeMap<String, f64>,
    /// Physical chronicle events affecting the habitat (impacts, freezes).
    pub events: Vec<ContentHash>,
    /// Profile fields nobody supplied.
    pub missing: Vec<String>,
}

/// How a claim was made (life design section 13): shown with every number.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeEvidence {
    /// No life model is installed: nothing is claimed either way.
    NoModel,
    /// Procedural sketch within physical caps (L1).
    PlausibleReconstruction,
    /// As above, coupled to the planet through fluxes (L2).
    PlausibleReconstructionCoupled,
    /// Simulated mechanisms within stated limits (L3+).
    Simulated,
    /// Written by a person.
    Authored,
}

/// A distribution over time, seconds of universe time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TimeDistribution {
    /// Quantiles as (probability, time) pairs, rising.
    pub quantiles: Vec<(f64, f64)>,
}

/// An additive change a suite proposes to a physical field (K10 rights apply).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EnvelopeFlux {
    pub field: String,
    /// Per second, in the field's unit.
    pub rate: f64,
}

/// The cloud for a habitat, region or world at an epoch (handoff 11A).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeEnvelope {
    pub address: Address,
    pub epoch: UniverseTime,
    /// Probability life is present; `None` when no model can say.
    pub p_present: Option<f64>,
    pub origin_time: Option<TimeDistribution>,
    /// Free-energy throughput budget, W.
    pub energy_budget_w: Option<f64>,
    pub biomass_class: Option<String>,
    /// Probability that each capability (transformation) exists.
    pub capabilities: BTreeMap<String, f64>,
    pub complexity: BTreeMap<String, f64>,
    pub fluxes: Vec<EnvelopeFlux>,
    pub signatures: BTreeMap<String, f64>,
    /// Plain statement of what the numbers do not know.
    pub uncertainty: String,
    pub evidence: LifeEvidence,
    /// Hash of the profiles the envelope was made from.
    pub inputs: Option<ContentHash>,
}

/// An aggregate population: never individuals or genomes.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    /// Habitat addresses it occupies, with its share of each.
    pub habitats: BTreeMap<Address, f64>,
    pub biomass_kg: Option<f64>,
    pub since: Option<UniverseTime>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Ancestry,
    Resource,
    Habitat,
    Capability,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Relation {
    pub kind: RelationKind,
    pub from: String,
    pub to: String,
}

/// What `resolve` returns for one address.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ResolvedBiosphere {
    pub address: Address,
    pub guilds: Vec<Guild>,
    pub relations: Vec<Relation>,
    pub evidence: LifeEvidence,
}

/// A question for `query_feasible`: can life with these capabilities exist
/// here, at this epoch?
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeQuery {
    pub address: Address,
    pub epoch: UniverseTime,
    pub capabilities: Vec<String>,
}

/// `Some(true)` or `Some(false)` with a reason, or `None` when nothing
/// installed can answer (never a guess).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Feasibility {
    pub answer: Option<bool>,
    pub reason: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LifeError(pub String);
impl std::fmt::Display for LifeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The life suite's own calls, beside K10's `SuiteAdapter` (`advance`).
pub trait LifeSuite {
    fn id(&self) -> &str;
    /// Evidence label for everything the suite produces.
    fn evidence(&self) -> LifeEvidence;
    /// The envelope for a set of habitats (a habitat, region or world).
    fn envelope(
        &self,
        address: &Address,
        epoch: UniverseTime,
        profiles: &[LifeRequirementProfile],
    ) -> LifeEnvelope;
    fn resolve(
        &self,
        envelope: &LifeEnvelope,
        seed: &sse_core::Seed,
    ) -> Result<ResolvedBiosphere, LifeError>;
    fn query_feasible(&self, q: &LifeQuery, profiles: &[LifeRequirementProfile]) -> Feasibility;
    fn explain(&self, address: &Address, field: &str) -> String;
}

#[cfg(test)]
mod budget_tests;
#[cfg(test)]
mod history_tests;
