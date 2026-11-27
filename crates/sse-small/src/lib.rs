//! Small bodies up close (track SB).
//!
//! A belt, ring or cloud is a probability field (formation's
//! [`sse_formation::leftovers::Cloud`]) until one member is looked at.
//! [`resolve`] turns that member, from the seed and its address alone, into a
//! definite body: its makeup, the type read off that makeup, rubble pile or
//! monolith, single or contact binary or separated pair, its spin, its figure
//! and a short history that explains it. The same body comes back at every
//! detail level and on every device, in any order of asking.
//!
//! It is the one resolving method for belts and rings: it starts from
//! `Cloud::resolve` (mass, size and orbit of the k-th largest object) and adds
//! what the member is, never a second way to make one. Notable bodies that
//! the system's formation made, such as the bared iron core of a world the
//! violent phase shattered (the Psyche case), come out of the same call: the
//! largest member of that impact's debris is the core ([`notables`] lists them).
//!
//! No per-asteroid simulation: each step is a cited, editable relation or
//! share read from `packs/base-small-bodies` (citations SB-*, fudges SB-*).

#![forbid(unsafe_code)]

pub mod comet;
pub mod figure;
mod member;
pub mod ring;
pub mod settings;
pub mod takes;

pub use comet::{activity, ice_shares, keeps_surface_ice, onset_au, Activity, Volatile};
pub use figure::{jacobi_axes, maclaurin_c_over_a, Q_BIFURCATION, Q_JACOBI_END, Q_MACLAURIN_MAX};
pub use member::{
    designed_field, made_field, makeup_for_density, mass_for_diameter, mass_for_makeup,
    nearest_cloud, notables, rank_of, relaxed_strength_of, resolve, resolve_pinned, round_km,
    Notable, Pins,
};
pub use settings::Settings;
pub use takes::{takes, Take};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
mod comet_tests;
#[cfg(test)]
mod design_tests;
#[cfg(test)]
mod ring_tests;
#[cfg(test)]
mod tests;

/// The star and system a cloud belongs to.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Host {
    pub luminosity_lsun: f64,
    pub mass_msun: f64,
    pub age_myr: f64,
}

/// Mass shares of a member's four material groups (they sum to 1).
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Makeup {
    pub metal: f64,
    pub rock: f64,
    pub carbon: f64,
    pub ice: f64,
}

impl Makeup {
    pub fn total(&self) -> f64 {
        self.metal + self.rock + self.carbon + self.ice
    }
    pub fn as_array(&self) -> [f64; 4] {
        [self.metal, self.rock, self.carbon, self.ice]
    }
    pub fn from_array(a: [f64; 4]) -> Self {
        let t: f64 = a.iter().sum();
        let t = if t > 0.0 { t } else { 1.0 };
        Makeup {
            metal: a[0] / t,
            rock: a[1] / t,
            carbon: a[2] / t,
            ice: a[3] / t,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Orbit {
    /// AU around the star, or around `around` for ring particles and irregular moons.
    pub a_au: f64,
    pub e: f64,
    pub i_deg: f64,
    pub node_deg: f64,
    pub peri_deg: f64,
    pub mean_anomaly_deg: f64,
    pub swarm: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Structure {
    /// "monolith", "rubble-pile", "relaxed" (round under its own gravity),
    /// "stripped-core" or "ring-aggregate".
    pub kind: String,
    /// Strength (0 to 1) of the process that makes a rubble pile here:
    /// disrupted within the system's age times held by gravity, not strength.
    pub rubble_strength: f64,
    /// Strength (0 to 1) of relaxing to a fluid figure.
    pub relaxed_strength: f64,
    /// How long ago it was last shattered and fell back together, Myr.
    pub disrupted_myr_ago: Option<f64>,
    /// Bulk porosity used for the bulk density.
    pub porosity: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pair {
    /// "single", "contact" (two lobes touching) or "separated" (a moon).
    pub kind: String,
    /// "yorp-fission", "gentle-merger" or "impact-moon".
    pub origin: Option<String>,
    /// Secondary's radius over the primary's.
    pub size_ratio: f64,
    /// Centre-to-centre distance of a separated pair, metres.
    pub separation_m: f64,
    /// The secondary's mass, kg (part of the member's mass).
    pub secondary_mass_kg: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Lobe {
    pub axes_m: [f64; 3],
    /// Centre along the long axis from the barycentre, metres.
    pub offset_m: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Figure {
    /// "maclaurin", "jacobi", "irregular", "top-shape" or "two-lobed", the
    /// name of the dominant figure (the axes are a weighted blend).
    pub kind: String,
    /// Semi-axes a >= b >= c of the whole body (its envelope for two lobes), metres.
    pub axes_m: [f64; 3],
    /// Two lobes for a contact binary, none otherwise.
    pub lobes: Vec<Lobe>,
    /// Surface relief as a share of the mean radius (craters, boulders, ridges).
    pub relief: f64,
    /// Spin parameter Omega^2 / (pi G rho).
    pub q: f64,
}

/// One line of a member's story.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Myr before the system's present, when known.
    pub myr_ago: Option<f64>,
    pub says: String,
}

/// A resolved member: a definite body.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Member {
    pub address: String,
    pub cloud: String,
    /// The cloud's kind ("asteroid-belt", "oort", "ring" and so on).
    pub kind: String,
    pub rank: u64,
    pub orbit: Orbit,
    pub mass_kg: f64,
    /// Volume-equivalent diameter at the member's own bulk density, km.
    pub diameter_km: f64,
    pub bulk_density_kg_m3: f64,
    pub makeup: Makeup,
    /// Blackbody temperature at its distance from the star, K.
    pub temperature_k: f64,
    /// Type label read off the result: S, Q, C, M, V, P, D, comet or ring.
    pub type_label: String,
    pub type_why: String,
    pub structure: Structure,
    pub pair: Pair,
    pub spin_hours: f64,
    /// The spin barrier for a strengthless body at its bulk density, hours.
    pub barrier_hours: f64,
    /// Strength (0 to 1) of sunlight's spin torque (YORP) over the system's age.
    pub yorp_strength: f64,
    pub figure: Figure,
    pub albedo: f64,
    /// Space weathering, 0 (fresh) to 1 (fully darkened and reddened).
    pub weathering: f64,
    pub history: Vec<Event>,
    /// Facts in the shape every other reader expects (mass_kg, radius_m,
    /// makeup.*): Traveller's resource books read these unchanged.
    pub facts: BTreeMap<String, f64>,
}
