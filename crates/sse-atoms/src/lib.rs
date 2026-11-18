//! SSE4 physics atom library (H6.2, build plan P1). Atoms are compiled,
//! cited operators; flows wire them together as data. Every atom here has at
//! least one citation and golden cases in `tests.rs`. New atoms follow the
//! issue, spec, tests, code rule (docs/atom-spec-template.md).
#![forbid(unsafe_code)]

pub mod climate;
pub mod common;
pub mod heat;
pub mod layers;
pub mod orbit;
pub mod planet;
pub mod star;
pub mod tides;

use sse_core::{Dimension, QuantityKind};
use sse_flow::AtomRegistry;
use std::collections::BTreeMap;

/// Every first-wave atom.
/// Every atom in the library, for callers that merge it into another registry.
pub fn atoms() -> Vec<Box<dyn sse_flow::Atom>> {
    star::atoms()
        .into_iter()
        .chain(orbit::atoms())
        .chain(planet::atoms())
        .chain(climate::atoms())
        .chain(tides::atoms())
        .chain(layers::atoms())
        .chain(heat::atoms())
        .collect()
}

pub fn registry() -> AtomRegistry {
    let mut r = AtomRegistry::default();
    for a in atoms() {
        r.insert(a);
    }
    r
}

/// Descriptions of every quantity kind the atoms' ports use. A port kind
/// missing here fails `every_port_kind_is_described`.
pub const KIND_DESCRIPTIONS: &[(&str, &str)] = &[
    ("star.mass", "mass of a star"),
    (
        "star.luminosity",
        "bolometric luminosity, typical for the star's mass on the main sequence",
    ),
    (
        "star.luminosity_at_age",
        "bolometric luminosity at the star's present age",
    ),
    ("star.radius", "photospheric radius of a star"),
    (
        "star.effective_temperature",
        "effective (black-body) surface temperature of a star",
    ),
    (
        "star.main_sequence_lifetime",
        "time a star spends on the main sequence",
    ),
    ("star.age", "time since the star reached the main sequence"),
    ("orbit.semi_major_axis", "semi-major axis of an orbit"),
    ("orbit.eccentricity", "orbital eccentricity, 0 for a circle"),
    ("orbit.period", "sidereal orbital period"),
    (
        "orbit.total_mass",
        "sum of the two masses in a two-body orbit",
    ),
    ("orbit.host_mass", "mass of the body being orbited"),
    ("orbit.periapsis", "closest distance to the host"),
    ("orbit.apoapsis", "farthest distance from the host"),
    (
        "orbit.mean_anomaly",
        "mean anomaly in radians: the share of the period elapsed since periapsis, times 2π",
    ),
    ("orbit.eccentric_anomaly", "eccentric anomaly in radians"),
    (
        "orbit.true_anomaly",
        "true anomaly in radians: angle from periapsis seen from the host",
    ),
    (
        "orbit.radius_fraction",
        "current distance from the host as a fraction of the semi-major axis",
    ),
    ("body.mass", "mass of a planet, moon or minor body"),
    (
        "body.radius",
        "volumetric mean radius of a planet, moon or minor body",
    ),
    ("body.bulk_density", "mean density"),
    (
        "body.surface_gravity",
        "gravitational acceleration at the surface, ignoring rotation",
    ),
    ("body.escape_velocity", "escape velocity from the surface"),
    ("body.hill_radius", "Hill sphere radius at periapsis"),
    (
        "body.roche_limit",
        "distance inside which a satellite held by its own gravity is torn apart",
    ),
    (
        "body.initial_spin_rate",
        "spin angular rate before tidal despinning",
    ),
    ("satellite.bulk_density", "mean density of a satellite"),
    (
        "interior.core_mass_fraction",
        "fraction of the mass in the iron core",
    ),
    ("interior.core_radius", "radius of the iron core"),
    ("body.age", "time since the body formed"),
    (
        "inheritance.k40_rel",
        "potassium-40 at the star's birth relative to the Sun's birth cloud",
    ),
    (
        "inheritance.r_process_rel",
        "r-process elements (uranium, thorium) relative to the Sun's birth cloud",
    ),
    (
        "inheritance.al26_rel",
        "aluminium-26 at the star's birth relative to the Sun's birth cloud",
    ),
    (
        "interior.mantle_mass",
        "mass of the silicate mantle and crust",
    ),
    (
        "interior.radiogenic_heating",
        "heat released by radioactive decay per kg of silicate",
    ),
    (
        "interior.radiogenic_power",
        "total heat released by radioactive decay in the interior",
    ),
    (
        "interior.radiogenic_heat_flow",
        "radiogenic heat per square metre of surface",
    ),
    (
        "interior.active_geology_lifetime",
        "age at which radiogenic heat flow falls too low for volcanism and plate motion",
    ),
    (
        "planetesimal.formation_time",
        "time after the first solids (CAIs) that a planetesimal formed",
    ),
    (
        "planetesimal.al26_energy",
        "aluminium-26 energy per kg released after formation",
    ),
    (
        "planetesimal.water_kept_fraction",
        "share of a planetesimal's water that survives aluminium-26 heating",
    ),
    ("planet.insolation", "stellar flux arriving at the planet"),
    (
        "planet.bond_albedo",
        "fraction of all incoming starlight reflected",
    ),
    (
        "planet.equilibrium_temperature",
        "temperature of a bare black body absorbing the same light",
    ),
    (
        "planet.surface_temperature",
        "global mean surface temperature",
    ),
    (
        "atmosphere.infrared_optical_depth",
        "grey infrared optical depth of the atmosphere",
    ),
    (
        "atmosphere.temperature",
        "temperature of an atmospheric layer",
    ),
    (
        "atmosphere.mean_molecular_weight",
        "mean molecular weight in atomic mass units",
    ),
    (
        "atmosphere.scale_height",
        "height over which pressure falls by a factor e",
    ),
    ("atmosphere.mass", "total mass of the atmosphere"),
    (
        "atmosphere.surface_pressure",
        "atmospheric pressure at the surface",
    ),
    (
        "atmosphere.exobase_radius",
        "distance from the centre to the exobase",
    ),
    (
        "atmosphere.exobase_temperature",
        "temperature at the exobase",
    ),
    (
        "atmosphere.particle_mass_amu",
        "mass of an escaping particle in atomic mass units",
    ),
    (
        "atmosphere.jeans_parameter",
        "Jeans escape parameter at the exobase",
    ),
    ("tides.heating_power", "tidal heating power"),
    (
        "tides.locking_time",
        "time to despin to synchronous rotation",
    ),
    (
        "ocean.salinity_mass_fraction",
        "dissolved salt as a fraction of the brine's mass",
    ),
    (
        "ocean.freezing_point",
        "temperature at which the brine starts to freeze",
    ),
    (
        "layer.thickness",
        "thickness of a layer, or depth within it",
    ),
    ("layer.top_temperature", "temperature at the top of a layer"),
    (
        "layer.base_temperature",
        "temperature at the base of a layer",
    ),
    (
        "layer.conductive_heat_flow",
        "heat flow conducted through a layer",
    ),
    ("layer.density", "mean density of a layer"),
    ("layer.pressure_increase", "pressure added across a layer"),
];

/// Quantity kinds for the base-physics pack, from the atoms' port kinds.
pub fn kinds() -> BTreeMap<String, QuantityKind> {
    let desc: BTreeMap<&str, &str> = KIND_DESCRIPTIONS.iter().copied().collect();
    let mut out: BTreeMap<String, QuantityKind> = BTreeMap::new();
    let reg = registry();
    for d in reg.decls() {
        for p in d.inputs.iter().chain(d.outputs.iter()) {
            out.entry(p.kind.clone()).or_insert_with(|| QuantityKind {
                id: p.kind.clone(),
                dim: p.dim,
                range: range_of(&p.kind, p.dim),
                description: desc.get(p.kind.as_str()).unwrap_or(&"").to_string(),
            });
        }
    }
    out
}

/// Physically valid ranges, where a kind has one.
fn range_of(kind: &str, _dim: Dimension) -> Option<(f64, f64)> {
    match kind {
        "orbit.eccentricity" => Some((0.0, 1.0)),
        "planet.bond_albedo" => Some((0.0, 1.0)),
        "interior.core_mass_fraction" => Some((0.0, 1.0)),
        "planetesimal.water_kept_fraction" => Some((0.0, 1.0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
