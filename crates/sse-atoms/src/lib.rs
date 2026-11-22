//! SSE4 physics atom library (H6.2, build plan P1). Atoms are compiled,
//! cited operators; flows wire them together as data. Every atom here has at
//! least one citation and golden cases in `tests.rs`. New atoms follow the
//! issue, spec, tests, code rule (docs/atom-spec-template.md).
#![forbid(unsafe_code)]

pub mod activity;
pub mod air;
pub mod chem;
pub mod climate;
pub mod common;
pub mod convection;
pub mod field;
pub mod heat;
pub mod layers;
pub mod light;
pub mod magnetic;
pub mod nebula;
pub mod orbit;
pub mod planet;
pub mod reach;
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
        .chain(magnetic::atoms())
        .chain(light::atoms())
        .chain(nebula::atoms())
        .chain(activity::atoms())
        .chain(convection::atoms())
        .chain(field::atoms())
        .chain(air::atoms())
        .chain(chem::atoms())
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
        "chem.standard_gibbs_298",
        "standard free energy of a reaction at 298.15 K, per mole of reaction as written",
    ),
    (
        "chem.standard_enthalpy",
        "standard enthalpy of a reaction, per mole of reaction as written",
    ),
    (
        "chem.standard_gibbs",
        "standard free energy of a reaction at the local temperature",
    ),
    ("chem.temperature", "temperature at which a reaction runs"),
    (
        "chem.ln_reaction_quotient",
        "natural log of the reaction quotient Q from local activities",
    ),
    (
        "chem.reaction_gibbs",
        "free energy released (negative) or needed (positive) by a reaction under local conditions",
    ),
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
    (
        "interior.dynamo_density",
        "mean density of the convecting, conducting region that runs the dynamo",
    ),
    (
        "interior.dynamo_radius",
        "outer radius of the dynamo region (the core top, or about 0.85 R for a giant)",
    ),
    (
        "interior.dynamo_heat_flux",
        "heat flux carried out of the dynamo region by convection",
    ),
    (
        "magnetic.dynamo_field",
        "rms magnetic field inside the dynamo region",
    ),
    (
        "body.surface_magnetic_field",
        "equatorial dipole magnetic field at the body's surface",
    ),
    ("body.magnetic_moment", "magnetic dipole moment of the body"),
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
    // light (O2)
    ("body.internal_heat_flux", "heat flowing out through a body's surface from inside, per area: a star's or brown dwarf's own luminosity, a giant planet's left-over heat"),
    ("body.effective_temperature", "temperature of the black body that radiates what this body radiates in total"),
    ("light.reflected_area", "effective area for reflected starlight: Bond albedo times cross-section times the source's dilution (R*/d)²"),
    ("wind.density", "mass density of the stellar wind at the body"),
    ("wind.speed", "speed of the stellar wind at the body"),
    ("wind.transverse_field", "magnetic field in the stellar wind across the flow direction, at the body"),
    ("body.magnetic_moment", "magnetic dipole moment of a body"),
    ("body.magnetopause_radius", "stand-off distance of the magnetopause towards the wind, from the body's centre; zero without a field"),
    ("radio.auroral_power", "auroral radio power, all frequencies"),
    ("radio.max_frequency", "highest auroral radio frequency, the electron gyrofrequency at the magnetic pole"),
    // nebulae (O3)
    ("light.ionising_photon_rate", "photons per second able to ionise hydrogen (above 13.6 eV)"),
    ("gas.hydrogen_density", "number density of hydrogen nuclei in the surrounding gas"),
    ("nebula.ionised_radius", "radius out to which a star's light keeps the gas ionised (Strömgren radius)"),
    ("nebula.recombination_rate", "hydrogen recombinations per second in a nebula"),
    ("nebula.h_alpha_power", "power in the hydrogen-alpha line"),
    ("nebula.expansion_speed", "speed at which a nebula's shell expands"),
    ("nebula.age", "time since the nebula's cause (a star's death, an explosion, a cluster's birth)"),
    ("nebula.shell_radius", "radius of a nebula's shell or shock"),
    ("event.explosion_energy", "kinetic energy of an explosion"),
    ("event.ejecta_mass", "mass thrown out by an explosion"),
    // stellar activity (O10)
    ("body.rotation_period", "time for one turn about the spin axis (sidereal)"),
    ("star.convective_turnover_time", "time for the churning outer layer of a cool star to turn over once"),
    ("star.rossby_number", "rotation period over convective turnover time; low means magnetically active"),
    ("star.xray_fraction", "share of a star's power given out as coronal X-rays (L_X / L_bol)"),
    ("star.xray_luminosity", "coronal X-ray power of a star"),
    ("star.activity_cycle_period", "length of a star's spot and activity cycle"),
    ("flare.energy", "total energy released by a flare, all wavelengths"),
    ("flare.rate_above", "flares per unit time with at least the given energy"),
    // what stops a wind (MF6)
    ("body.ionopause_altitude", "height above the ground at which a body's air holds off the stellar wind"),
    ("magnetosphere.flaring", "how quickly a magnetosphere's boundary flares out behind the nose (Shue exponent)"),
    ("host.field_at_orbit", "the host planet's magnetic field strength at a moon's orbit"),
    ("host.plasma_pressure_at_orbit", "pressure of the host's trapped plasma flowing past a moon"),
    ("layer.electrical_conductivity", "electrical conductivity of a layer (salty ocean, magma, iron)"),
    ("layer.thickness", "thickness of a layer"),
    ("layer.outer_radius", "radius of a layer's top"),
    ("field.change_period", "period with which the outside magnetic field seen by a body changes"),
    ("induction.amplitude", "strength of a conducting layer's induced field relative to a perfect conductor's"),
    // surface convection
    ("convection.cell_size", "diameter of a surface convection cell (granule)"),
    ("convection.speed", "speed of the churning gas near the surface"),
    ("convection.cell_lifetime", "how long one surface convection cell lasts"),
    ("convection.supergranule_size", "diameter of the large slow cells that outline the magnetic network"),
    ("convection.supergranule_lifetime", "how long one supergranule lasts"),
    // air optics
    ("light.wavelength", "a wavelength of light"),
    ("gas.refractivity", "refractive index minus one of a gas at its reference density"),
    ("gas.reference_density", "number density at which a gas's refractivity is given"),
    ("gas.king_factor", "King correction factor for scattering by non-spherical molecules (1 for spheres)"),
    ("gas.rayleigh_cross_section", "Rayleigh scattering cross-section per molecule"),
    ("atmosphere.mean_molecular_mass", "mean mass of one air molecule"),
    ("atmosphere.rayleigh_optical_depth", "Rayleigh optical depth of the whole air column, looking straight up"),
    ("atmosphere.number_density", "molecules per cubic metre of air"),
    ("atmosphere.rayleigh_scattering_coefficient", "Rayleigh scattering per metre of path"),
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
