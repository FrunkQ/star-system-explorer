//! SSE4 physics atom library (H6.2, build plan P1). Atoms are compiled,
//! cited operators; flows wire them together as data. Every atom here has at
//! least one citation and golden cases in `tests.rs`. New atoms follow the
//! issue, spec, tests, code rule (docs/atom-spec-template.md).
#![forbid(unsafe_code)]

pub mod activity;
pub mod air;
pub mod aurora;
pub mod belts;
pub mod chem;
pub mod common;
pub mod convection;
pub mod extinction;
pub mod field;
pub mod formulas;
pub mod heat;
pub mod ice_ground;
pub mod inertia;
pub mod layers;
pub mod ledger;
pub mod light;
pub mod magnetic;
pub mod matter;
pub mod mie;
pub mod nebula;
pub mod orbit;
pub mod planet;
pub mod reach;
pub mod redox;
pub mod shield;
pub mod star;
#[cfg(test)]
#[path = "test/mod.rs"]
mod was;
pub mod weathering;

use sse_core::{Dimension, QuantityKind};
use sse_flow::AtomRegistry;
use std::collections::BTreeMap;

/// Every first-wave atom.
/// Every atom in the library, for callers that merge it into another registry.
pub fn atoms() -> Vec<Box<dyn sse_flow::Atom>> {
    star::atoms()
        .into_iter()
        .chain(formulas::atoms())
        .chain(orbit::atoms())
        .chain(planet::atoms())
        .chain(heat::atoms())
        .chain(magnetic::atoms())
        .chain(nebula::atoms())
        .chain(extinction::atoms())
        .chain(mie::atoms())
        .chain(aurora::atoms())
        .chain(belts::atoms())
        .chain(chem::atoms())
        .collect()
}

/// One block's face (ports, settings, references, words) by its id, whether
/// its maths is Rust or a formula file.
pub fn decl_of(id: &str) -> sse_flow::AtomDecl {
    atoms()
        .into_iter()
        .find(|a| a.decl().id == id)
        .map(|a| a.decl().clone())
        .unwrap_or_else(|| panic!("no block {id}"))
}

/// The faces of several blocks, in the order asked.
pub fn decls_of(ids: &[&str]) -> Vec<sse_flow::AtomDecl> {
    ids.iter().map(|id| decl_of(id)).collect()
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
    ("gravity.gravitational_radius", "GM/c², the length every strong-gravity distance scales with (m)"),
    ("gravity.spin", "a black hole's spin a = Jc/GM², 0 to 1; signed where the direction of an orbit or ray matters"),
    ("gravity.horizon_radius", "radius of a black hole's surface of no return (m)"),
    ("gravity.photon_orbit_radius", "radius at which light circles a compact mass (m)"),
    ("gravity.last_stable_orbit_radius", "innermost stable circular orbit round a compact mass, a disc's inner edge (m)"),
    ("gravity.shadow_angle", "angular radius of a black hole's shadow as a viewer sees it, radians"),
    ("gravity.impact_parameter", "distance at which a light ray would pass a mass if space were flat (m)"),
    ("gravity.bend_angle", "angle a light ray is turned by a mass, radians"),
    ("gravity.einstein_angle", "angular radius of the Einstein ring of a lens, radians"),
    ("gravity.distance", "distance from a mass's centre (m)"),
    ("gravity.clock_rate_hovering", "rate of a clock held still near a mass, against a far clock"),
    ("gravity.clock_rate_orbiting", "rate of a clock on a circular orbit near a mass, against a far clock"),
    ("group.age", "age of the universe when a group's edge is found (s)"),
    ("group.zero_velocity_radius", "radius round a group inside which galaxies have stopped moving apart and fall together (m)"),
    ("gravity.tide_per_metre", "difference in pull per metre along the line to a mass (s-2)"),
    ("feed.efficiency", "share of the rest energy of matter falling through a disc that comes out as light"),
    ("feed.eddington_luminosity", "light at which the push of radiation on infalling gas balances gravity (W)"),
    ("feed.eddington_rate", "feeding rate that gives the Eddington luminosity (kg/s)"),
    ("feed.rate", "mass falling onto a body each second through its feed (kg/s)"),
    ("feed.eddington_share", "feeding rate as a share of the Eddington rate"),
    ("feed.inner_edge", "where a disc ends on the inside: surface, last stable orbit or magnetic field (m)"),
    ("feed.magnetospheric_radius", "where a body's magnetic field takes over from its disc (m)"),
    ("feed.outer_edge", "where a disc ends on the outside (m)"),
    ("feed.mass_ratio", "a companion's mass over the fed body's"),
    ("feed.flow_thin", "strength of the thin bright disc, 0 to 1"),
    ("feed.flow_hot", "strength of the hot, puffed-up thin flow, 0 to 1"),
    ("feed.flow_thick", "strength of the thick, swollen disc, 0 to 1"),
    ("feed.hot_lit_share", "how well a hot flow shines against a thin disc fed as fast, 0 to 1"),
    ("feed.gas_density", "density of the gas round a mass that sweeps it up (kg/m³)"),
    ("feed.sound_speed", "sound speed in the gas round a mass (m/s)"),
    ("feed.relative_speed", "speed of a mass through the gas round it (m/s)"),
    ("feed.capture_radius", "distance inside which a mass captures the gas round it (m)"),
    ("light.view_tilt", "angle between the line of sight and a disc's axis, radians (0 face on)"),
    ("light.image_angle", "angle round a picture from a disc's axis as seen, towards the viewer's left, radians"),
    ("light.shift_factor", "seen frequency over emitted frequency of light from a moving or deep source"),
    ("observer.distance", "distance of a viewer from a body's centre (m)"),
    ("observer.lens_distance", "distance from a viewer to a bending mass (m)"),
    ("observer.source_distance", "distance from a viewer to the light behind a bending mass (m)"),
    ("star.mass", "mass of a star"),
    ("orbit.angular_momentum_deficit", "angular momentum an orbit lacks against a flat circle of the same size (kg m2 s-1)"),
    ("orbit.ascending_node", "longitude of an orbit's ascending node, radians"),
    ("orbit.inclination", "tilt of an orbit against its host's reference plane, radians"),
    ("orbit.chaotic_zone", "orbit gap, as a fraction, inside which neighbouring resonances overlap and the motion is chaotic"),
    ("orbit.binary_limit", "widest (round one star) or nearest (round both) lasting planet orbit in a binary"),
    ("orbit.hill_gap", "inner apoapsis to outer periapsis, in mutual Hill radii"),
    ("orbit.hill_spacing", "gap between two neighbouring orbits, in mutual Hill radii"),
    ("orbit.kozai_time", "time for one Kozai-Lidov swing of an inner orbit"),
    ("orbit.mass_share", "a binary's mass ratio, the chosen star's share of the pair's mass"),
    ("orbit.moon_limit", "widest orbit a moon keeps round its planet"),
    ("orbit.mutual_hill_radius", "radius of the gravity bubble two neighbours share"),
    ("orbit.mutual_inclination", "angle between two orbital planes, radians"),
    ("orbit.resonance_offset", "how far two orbits' period ratio is from a whole-number ratio, as a fraction"),
    ("orbit.resonance_turns", "turns one body makes in one cycle of a mean motion resonance"),
    ("orbit.resonance_width", "how far a period ratio may be from a resonance and still be held, as a fraction"),
    ("orbit.relative_amd", "a system's angular momentum deficit against an outer planet's circular angular momentum"),
    ("orbit.mass_ratio", "a world's share of its own and its host's mass together, mu"),
    ("orbit.lagrange_stable", "1 if the points 60 degrees ahead of and behind a world (L4, L5) hold small bodies, 0 if not"),
    ("orbit.tadpole_width", "how far a Trojan's orbit size swings round the world's, as a fraction"),
    ("orbit.giants_between", "how many giant worlds sit between a world and a belt"),
    ("orbit.coorbital_disturbed", "1 if a neighbour shakes a world's Trojans loose, 0 if not"),
    ("orbit.period_ratio", "the outer of two orbits' period over the inner's"),
    ("orbit.resonance_p", "a mean motion resonance's outer number (p in p:q)"),
    ("orbit.resonance_q", "a mean motion resonance's inner number (q in p:q)"),
    ("formation.captured_share", "share of a stirred belt a world catches as Trojans"),
    ("formation.long_lived_share", "share of a world's caught Trojans still there after billions of years"),
    ("orbit.stability_verdict", "a stability check's verdict as a number: 0 lasts, higher is worse (each block says what its numbers mean)"),
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
        "power stirring the dynamo region, as a heat flux: convected heat plus compositional buoyancy",
    ),
    ("interior.core_heat_flux", "heat flux leaving the top of the core"),
    ("element.ci_mass_fraction", "an element's mass share in CI chondrites"),
    ("star.element_ratio", "an element's abundance in a star over the Sun's, by mass per hydrogen"),
    ("element.oxide_oxygen_ratio", "mass of oxygen an element binds in its rock oxide, per mass of the element"),
    ("material.ci_weight", "a formation material's summed CI weight over its riders, bound oxygen included"),
    ("element.material_share", "an element's mass share of the formation material it rides in"),
    ("element.bound_oxygen_share", "mass share of a material that is oxygen bound to one element's oxide"),
    ("interior.separation_strength", "how far a world separated into metal and silicate, 0 to 1"),
    ("element.metal_silicate_partition", "an element's concentration in metal over its concentration in silicate"),
    ("interior.metal_fraction", "metal's mass share of a separated body's metal and silicate"),
    ("element.core_share", "share of a body's element held in its core"),
    ("element.crust_enrichment", "an element's concentration in crust over its concentration in mantle"),
    ("interior.crust_fraction", "the crust's mass share of a body's silicate"),
    ("element.crust_share", "share of the silicate's element held in the crust"),
    ("interior.silicate_mass", "mass of a body's silicate (mantle and crust)"),
    ("star.metallicity_z", "a star's share of heavy elements by mass"),
    ("interior.heavy_element_share", "a giant's share of heavy elements by mass"),
    ("deposit.footprint_area", "area of an ore deposit or ore province"),
    ("element.crust_mass_fraction", "an element's share of a world's crust by mass"),
    ("element.ore_enrichment", "how many times an ore process raised an element over the crust"),
    ("deposit.element_mass", "mass of an element held in an ore deposit"),
    ("deposit.grade", "an element's share of the ore by mass"),
    ("interior.reducing_carbon_share", "carbon free to smelt iron oxide back to metal, share of the rock and metal by mass"),
    ("matter.molar_mass_fe", "molar mass of iron"),
    ("matter.molar_mass_feo", "molar mass of iron oxide, FeO"),
    ("matter.molar_mass_c", "molar mass of carbon"),
    ("interior.redox_iw", "the mantle's oxygen fugacity against the iron-wüstite buffer at its magma ocean's surface, log10 units"),
    ("interior.redox_iw_core", "the mantle's oxygen fugacity against iron-wüstite at core formation, log10 units"),
    ("early.outgassed_hydrogen", "hydrogen a magma ocean gave off, counted as H2"),
    ("early.outgassed_carbon", "carbon a magma ocean gave off"),
    ("early.outgassed_pressure", "pressure of the air a magma ocean gave off"),
    ("early.outgas_h2", "H2's share by moles of the gas a magma ocean gave off"),
    ("early.outgas_h2o", "water vapour's share by moles of the gas a magma ocean gave off"),
    ("early.outgas_co", "carbon monoxide's share by moles of the gas a magma ocean gave off"),
    ("early.outgas_co2", "carbon dioxide's share by moles of the gas a magma ocean gave off"),
    ("early.outgas_ch4", "methane's share by moles of the gas a magma ocean gave off"),
    ("interior.core_top_temperature", "temperature at the top of the core"),
    ("interior.core_top_gravity", "gravity at the top of the core"),
    (
        "interior.convected_flux",
        "heat flux at the core's top beyond what conduction along the adiabat carries",
    ),
    (
        "interior.light_element_flux",
        "mass flux of buoyant light elements let out into the liquid core, per area of its top",
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
    ("tides.frequency", "how fast a tide flexes a world, as an angular rate: the orbit's mean motion for a locked moon"),
    ("tides.im_k2", "imaginary part of a world's tidal Love number k2: the share of a tide's work turned to heat, the k2/Q of the heating law"),
    ("tides.re_k2", "real part of a world's tidal Love number k2: how far its shape follows a tide"),
    ("interior.rigidity", "shear modulus of a world's rock or ice: how stiffly it resists being bent"),
    ("interior.viscosity", "dynamic viscosity of a world's insides over long times: how slowly the solid rock or ice creeps"),
    (
        "tides.locking_time",
        "time to despin to synchronous rotation",
    ),
    ("tides.raiser_mass", "mass of a body raising a tide on the world"),
    ("tides.raiser_distance", "distance from the world's centre to a body raising a tide on it"),
    ("tides.raiser_period", "time for a body raising a tide to go once round the world's sky: its orbit about the world, or the world's year for the star"),
    ("tides.equilibrium_range", "height from low to high water of the open-sea equilibrium tide one body raises"),
    ("tides.period", "time between one high tide and the next"),
    ("weather.wind_speed_10m", "wind speed ten metres above the ground or water"),
    ("ocean.fetch", "length of open water a wind blows over"),
    ("ocean.significant_wave_height", "significant wave height: the mean of the highest third of the waves"),
    ("ocean.wave_peak_period", "period of the waves carrying the most energy"),
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
    ("nebula.electron_temperature", "temperature of the free electrons in a nebula's glowing gas"),
    ("nebula.photons_above_threshold", "photons a second from the lighting stars hard enough to make or strip an ion"),
    ("nebula.photons_taken", "ionising photons a second a nebula takes up, balancing its recombinations"),
    ("nebula.taker_share", "atoms per hydrogen atom of the gas that takes up a range of ionising photons"),
    ("light.photon_energy", "energy of one photon"),
    ("light.fit_energy", "energy scale of a fit to an ion's photoionisation cross-section"),
    ("light.fit_cross_section", "cross-section scale of a fit to an ion's photoionisation cross-section"),
    ("light.photoionisation_cross_section", "target area an ion offers a photon that can strip it"),
    ("nebula.photoionisation_rate", "how often each ion is stripped by the starlight where it is"),
    ("nebula.recombination_coefficient", "rate coefficient for an ion catching an electron back"),
    ("nebula.electron_density", "free electrons per cubic metre in a nebula's glowing gas"),
    ("nebula.lower_ion_share", "share of an element left as the lower of two ions"),
    ("light.fit_ya", "shape number y_a of a fit to an ion's photoionisation cross-section (Verner et al. 1996)"),
    ("light.fit_power", "power P of a fit to an ion's photoionisation cross-section (Verner et al. 1996)"),
    ("light.fit_yw", "shape number y_w of a fit to an ion's photoionisation cross-section (Verner et al. 1996)"),
    ("light.fit_y0", "shape number y_0 of a fit to an ion's photoionisation cross-section (Verner et al. 1996)"),
    ("light.fit_y1", "shape number y_1 of a fit to an ion's photoionisation cross-section (Verner et al. 1996)"),
    ("atmosphere.grazing_column", "molecules per square metre along a ray grazing a world's limb"),
    ("light.bend_angle", "angle through which an air turns a ray of light"),
    ("light.refraction_dilution", "share of light's brightness left after refraction spreads the rays apart"),
    ("shadow.distance", "distance behind a world, along its shadow"),
    ("nebula.zone_share", "share of a nebula's glowing gas, nearest the star first, that a kind of ion fills"),
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
    // prominences (SG6)
    ("star.surface_field", "mean magnetic field strength over a star's surface"),
    ("star.corona_temperature", "mean temperature of a star's corona"),
    ("star.corona_base_density", "particles per cubic metre at the foot of a star's corona"),
    ("star.closed_corona_extent", "reach of a star's closed magnetic field, in stellar radii"),
    // what stops a wind (MF6)
    ("body.ionopause_altitude", "height above the ground at which a body's air holds off the stellar wind"),
    ("magnetosphere.flaring", "how quickly a magnetosphere's boundary flares out behind the nose (Shue exponent)"),
    ("host.field_at_orbit", "the host planet's magnetic field strength at a moon's orbit"),
    ("host.plasma_pressure_at_orbit", "pressure of the host's trapped plasma flowing past a moon"),
    ("layer.electrical_conductivity", "electrical conductivity of a layer (salty ocean, magma, iron)"),
    ("layer.gravity", "gravity inside a layer"),
    ("layer.thermal_expansivity", "thermal expansivity of a layer's matter"),
    ("layer.specific_heat", "specific heat of a layer's matter"),
    ("interior.dynamo_shell_thickness", "depth of the stirred, conducting layer a dynamo runs in"),
    ("interior.dynamo_flow_speed", "typical speed of the flow stirring a conducting layer"),
    ("interior.dynamo_local_rossby", "a stirring flow's speed over the spin speed across its layer"),
    ("interior.magnetic_reynolds", "magnetic Reynolds number of a stirred layer: stretching over leaking"),
    ("interior.magnetic_decay_time", "time an unstirred field takes to leak out of a conducting layer"),
    ("surface.exposure_age", "time bare ground has lain open to space since it was last renewed, by lava, ice or a crater's ejecta"),
    ("surface.wind_flux_ratio", "the star's wind flux at the world against its flux at 1 AU (the inverse square of distance)"),
    ("surface.wind_reaches_ground", "1 when the star's wind reaches the ground, 0 when a magnetic field holds it off"),
    ("surface.maturity", "how weathered bare ground is, 0 fresh to 1 fully mature"),
    ("surface.weathering_factor_650", "weathered over fresh reflectance of the same soil at 650 nm (red)"),
    ("surface.weathering_factor_550", "weathered over fresh reflectance of the same soil at 550 nm (green)"),
    ("surface.weathering_factor_450", "weathered over fresh reflectance of the same soil at 450 nm (blue)"),
    ("body.ice_share", "share of a world's mass that is ice"),
    ("material.rock_density", "density of rock at low pressure"),
    ("material.ice_density", "density of ice at low pressure"),
    ("material.vapour_pressure", "pressure of a substance's own vapour over its solid or liquid at a temperature"),
    ("material.molar_mass", "mass of one mole of a substance"),
    ("material.photo_lifetime_1au", "how long a molecule of a gas lasts in a quiet Sun's light at 1 AU before it is broken up or ionised"),
    ("orbit.star_distance", "distance from the star (a moon's is its planet's)"),
    ("surface.ice_lost", "depth of open ice an airless world loses over its age"),
    ("material.reflectance", "share of light a material's ground sends back, against a white Lambertian surface"),
    ("surface.lag_share", "share of icy ground under a dark lag of dust and rock"),
    ("surface.ground_reflectance", "share of light the ground sends back"),
    ("body.moment_of_inertia_factor", "moment of inertia over mass times radius squared: 0.4 for a uniform ball, less as mass gathers in the middle"),
    ("interior.dynamo_onset", "how far a dynamo runs, 0 to 1, from its magnetic Reynolds number"),
    ("interior.dynamo_rossby", "spin period over a dynamo's stirring time"),
    ("interior.dynamo_rossby_factor", "factor on a dynamo's field from its spin, 1 when saturated"),
    ("layer.thickness", "thickness of a layer"),
    ("layer.outer_radius", "radius of a layer's top"),
    ("field.change_period", "period with which the outside magnetic field seen by a body changes"),
    ("induction.amplitude", "strength of a conducting layer's induced field relative to a perfect conductor's"),
    // aurorae (MF9)
    ("aurora.field_line_distance", "distance at which a field line crosses the magnetic equator"),
    ("aurora.magnetic_latitude", "magnetic latitude at which a field line comes down to the surface"),
    ("ionosphere.pedersen_conductance", "effective height-integrated Pedersen conductance of one hemisphere's ionosphere"),
    ("body.equatorial_surface_field", "magnetic field strength at the surface on the magnetic equator"),
    ("magnetosphere.plasma_source_rate", "mass of plasma added to a magnetosphere each second (by a moon's volcanoes or geysers)"),
    ("magnetosphere.corotation_limit", "distance beyond which trapped plasma stops turning with the planet (Hill distance)"),
    ("aurora.electron_energy", "energy of the electrons raining into the air along the field lines"),
    ("aurora.electron_range", "mass of air above each square metre that a falling electron gets through before it stops"),
    ("aurora.power", "power coming down into the air as aurora, both hemispheres together"),
    ("light.radiative_lifetime", "mean time an excited atom or molecule waits before giving off its light"),
    ("light.quenching_rate_coefficient", "rate coefficient for a collision taking an excited atom's energy away before it shines"),
    ("light.surviving_share", "share of excited atoms that give off their light rather than lose it in a collision"),
    // dust extinction (Y2)
    ("dust.extinction_ratio", "how much dust dims light at a wavelength, relative to the V band (A_λ / A_V)"),
    // cosmic-ray shielding and dose (MF7)
    ("shield.distance", "distance from a body's centre at which shielding is asked for"),
    ("shield.magnetic_latitude", "latitude measured from a body's magnetic equator"),
    ("shield.cutoff_rigidity", "least momentum per charge a cosmic ray needs to arrive (Størmer cut-off)"),
    ("belts.equatorial_field", "magnetic field strength where a field line crosses the magnetic equator"),
    ("belts.wave_path", "length along the field line over which whistler waves grow (D R_p)"),
    ("belts.limit_normalisation", "the Kennel-Petschek cap on a belt: normalisation C of the spectrum C E^-1 per cm² s sr keV"),
    ("belts.shield_column", "mass per area of a shield in front of a dose point"),
    ("belts.penetration_energy_mev", "least energy, MeV, an electron needs to cross a shield"),
    ("belts.electron_energy_mev", "kinetic energy of an electron, MeV"),
    ("belts.radiation_yield", "share of a stopping electron's energy given off as X-rays"),
    ("belts.latitude_share", "share of a belt's equatorial dose left at a magnetic latitude along its field line"),
    ("constant.m_e", "electron mass, read from the master data sheet"),
    ("constant.e", "elementary charge, read from the master data sheet"),
    ("constant.eps0", "vacuum permittivity, read from the master data sheet"),
    ("shield.space_share", "share of the free-space cosmic-ray dose carried by particles above the cut-off"),
    ("shield.ground_share", "share of the no-cut-off ground dose left under a thick air at the cut-off"),
    ("shield.free_space_dose_rate", "cosmic-ray dose rate in open space at the body's orbit, unshielded"),
    ("atmosphere.column_mass", "mass of air above a unit area of ground"),
    ("shield.open_sky", "share of the sky not blocked by the body itself"),
    ("shield.dose_rate", "cosmic-ray dose rate after field and air"),
    // surface convection
    ("convection.cell_size", "diameter of a surface convection cell (granule)"),
    ("convection.speed", "speed of the churning gas near the surface"),
    ("convection.cell_lifetime", "how long one surface convection cell lasts"),
    ("convection.supergranule_size", "diameter of the large slow cells that outline the magnetic network"),
    ("convection.supergranule_lifetime", "how long one supergranule lasts"),
    // air optics
    ("light.wavelength", "a wavelength of light"),
    ("light.collision_strength", "how readily an electron's knock lifts an ion to a level, averaged over the electrons' speeds"),
    ("light.lower_level_weight", "statistical weight of the lower level or term of a line"),
    ("light.upper_level_weight", "statistical weight of the upper level of a line"),
    ("light.excitation_rate_coefficient", "rate at which electron knocks lift ions to a level, per ion per electron density"),
    ("light.deexcitation_rate_coefficient", "rate at which electron knocks bring ions down without light, per ion per electron density"),
    ("gas.refractivity", "refractive index minus one of a gas at its reference density"),
    ("gas.reference_density", "number density at which a gas's refractivity is given"),
    ("gas.king_factor", "King correction factor for scattering by non-spherical molecules (1 for spheres)"),
    ("gas.rayleigh_cross_section", "Rayleigh scattering cross-section per molecule"),
    ("atmosphere.mean_molecular_mass", "mean mass of one air molecule"),
    ("atmosphere.rayleigh_optical_depth", "Rayleigh optical depth of the whole air column, looking straight up"),
    ("atmosphere.number_density", "molecules per cubic metre of air"),
    ("atmosphere.rayleigh_scattering_coefficient", "Rayleigh scattering per metre of path"),
    // light through gas (ND2) and the eye's response
    ("gas.oscillator_strength", "oscillator strength of the line (how many electrons' worth it absorbs)"),
    ("gas.lower_level_energy", "energy of the level the line absorbs from, above the ground state"),
    ("gas.line_wavelength", "wavelength at the line's centre"),
    ("gas.temperature", "temperature of the gas"),
    ("gas.ionisation_energy", "energy to tear the outer electron off an absorbing atom"),
    ("gas.perturber_mass", "mass of one particle crowding an absorbing atom"),
    ("gas.perturber_polarisability", "polarisability volume of a particle crowding an absorbing atom"),
    ("gas.interaction_constant", "van der Waals C6: the pull between an atom and a neighbour, falling as the sixth power of distance"),
    ("gas.line_strength", "cross-section of one atom summed over the whole line (per hertz of width)"),
    ("gas.particle_mass", "mass of one absorbing atom or molecule"),
    ("gas.natural_damping", "natural damping rate of the line (from the levels' lifetimes)"),
    ("gas.widening_per_density", "damping added per perturbing particle per cubic metre, at the reference temperature"),
    ("gas.perturber_density", "number of perturbing particles per cubic metre"),
    ("gas.widening_temperature_exponent", "how the widening per particle grows with temperature (0.3 for van der Waals)"),
    ("gas.widening_reference_temperature", "temperature the widening per particle was measured at"),
    ("gas.doppler_width", "spread of the line in frequency from the atoms' thermal motion (one standard deviation)"),
    ("gas.damping_width", "half width of the line's damping wings in frequency"),
    ("gas.profile_width", "full width at half depth of the combined line shape"),
    ("gas.damping_share", "share of the line shape that is damping wings rather than the thermal core"),
    ("gas.detuning", "distance in frequency from the line's centre"),
    ("gas.line_shape", "share of the line's strength per hertz at this distance from its centre"),
    ("gas.absorption_cross_section", "absorbing cross-section of one particle at this wavelength"),
    ("gas.measured_cross_section", "cross-section per molecule as measured at this wavelength"),
    ("gas.measured_temperature", "temperature the band was measured at"),
    ("gas.band_temperature_exponent", "how the band strength follows temperature (0: not at all)"),
    ("gas.column", "particles in the way, per square metre of the beam"),
    ("gas.optical_depth", "optical depth: each unit dims the light to about a third"),
    ("gas.pair_coefficient", "absorption by a colliding pair, per density of each partner"),
    ("gas.partner_density", "number of partner particles per cubic metre"),
    ("light.spectral_flux", "light arriving at this wavelength"),
    ("gas.path_factor", "how many straight paths the light takes through the gas (2 for down and back up at the vertical, more when slanted)"),
    ("eye.wavelength_nm", "wavelength in nanometres"),
    ("eye.lobe_centre_nm", "wavelength the lobe peaks at, in nanometres"),
    ("eye.lobe_width_blue_nm", "width of the lobe on its blue side, in nanometres"),
    ("eye.lobe_width_red_nm", "width of the lobe on its red side, in nanometres"),
    ("eye.lobe", "height of the lobe at this wavelength, 1 at its peak"),
    ("body.luminosity", "power a body radiates in total"),
    ("outflow.mass_loss_rate", "mass a star sheds into its wind each second"),
    ("outflow.wind_speed", "speed of a star's dusty wind"),
    ("outflow.condensation_radius", "distance from a star where its wind turns to dust"),
    ("outflow.dust_depth", "optical depth of a star's dusty wind, looking out"),
    ("outflow.dust_luminosity", "starlight the dust takes in and glows out again"),
    ("outflow.dust_temperature", "temperature of a giant's new-made dust"),
    ("outflow.accretion_rate", "mass falling onto a young star each second"),
    ("outflow.launch_radius", "distance from a young star where its jet leaves the disc"),
    ("outflow.jet_mass_rate", "mass a young star's jets carry away each second"),
    ("outflow.jet_speed", "speed of a young star's jets"),
    ("outflow.jet_power", "power carried by a young star's jets"),
    ("outflow.shock_light", "light a jet's shocks give off"),
    ("fog.heat_power", "heat a craft must shed at one temperature"),
    ("fog.radiator_temperature", "temperature a craft's waste heat leaves at"),
    ("fog.radiator_area", "glowing radiator area that sheds a craft's heat"),
    ("fog.exhaust_speed", "speed a drive's exhaust leaves at"),
    ("fog.particle_mass", "mass of one exhaust particle"),
    ("fog.plume_temperature", "temperature of a drive's hot exhaust"),
    ("fog.jet_power", "power carried away in a drive's exhaust"),
    ("fog.plume_luminosity", "light a drive plume gives off"),
    ("fog.speed", "speed a contact was last seen moving at"),
    ("fog.acceleration", "most a contact's drive can accelerate it"),
    ("fog.since", "time since a contact was last seen"),
    ("fog.ring_radius", "radius of the ring a lost contact could be anywhere inside"),
    ("range.beam_power", "power in a beam as it leaves its mirror or lens"),
    ("range.wavelength", "wavelength of a beam"),
    ("range.aperture", "width of the mirror or lens that sends a beam"),
    // Ship sizing (RK3).
    ("ship.thrust", "push a drive gives"),
    ("ship.exhaust_speed", "speed of a drive's exhaust"),
    ("ship.jet_power", "power carried by a drive's exhaust"),
    ("ship.stowed_mass", "mass of propellant or cargo carried"),
    ("ship.packed_density", "how densely a load packs: a liquid's density or cargo's stowage"),
    ("ship.stowed_volume", "room a load takes in tanks or holds"),
    ("ship.crew", "people aboard"),
    ("ship.habitable_per_person", "net habitable room each person needs"),
    ("ship.net_to_gross", "share of a pressurised module left free once equipment is in"),
    ("ship.crew_volume", "pressurised room a crew needs"),
    ("ship.waste_per_jet", "heat left aboard for each watt in the jet"),
    ("ship.waste_heat", "heat a ship must shed"),
    ("ship.radiator_temperature", "temperature radiators shed heat at"),
    ("ship.emissivity", "how well a radiator panel glows, from 0 to 1"),
    ("ship.radiating_faces", "faces of a panel that see space"),
    ("ship.radiator_area", "area of radiator panel, one face"),
    // Tanks and dishes (CM13 part 2).
    ("ship.tank_pressure", "pressure a tank holds its load at"),
    ("ship.wall_density", "density of a tank wall's material"),
    ("ship.wall_stress", "stress a tank wall may carry: its strength over the safety factor"),
    ("ship.vessel_shape", "pressure vessel shape factor: 1.5 for a sphere, 2 for a long cylinder"),
    ("ship.wall_mass", "mass of a pressure tank's wall"),
    ("ship.one_tank_surface", "surface of one tank holding the whole load"),
    ("ship.tank_count", "number of like tanks a load is split into"),
    ("ship.split_surface", "surface of all the tanks a load is split into"),
    ("ship.dish_aperture", "width of a dish antenna"),
    ("ship.link_wavelength", "wavelength a radio or laser link works at"),
    ("ship.aperture_efficiency", "share of a dish's area that works"),
    ("ship.dish_gain", "gain of a dish over a bare antenna, as a ratio"),
    ("ship.transmit_power", "power a link sends"),
    ("ship.receive_gain", "gain of the receiving dish, as a ratio"),
    ("ship.link_distance", "distance a link spans"),
    ("ship.noise_temperature", "system noise temperature of a receiver"),
    ("ship.bit_energy_ratio", "energy per bit over noise density a link's coding needs, as a ratio"),
    ("ship.data_rate", "bits a second a link supports"),
    ("range.spread", "beam quality M²: how much faster than a perfect beam it spreads"),
    ("range.jitter", "pointing wobble of a beam, as an angle"),
    ("range.distance", "distance from the weapon to its target"),
    ("range.dwell", "time a beam is held on one spot"),
    ("range.absorbed_share", "share of arriving energy the target's skin takes in"),
    ("range.spot_radius", "radius of a beam's spot on the target"),
    ("range.irradiance", "beam power arriving on each square metre of the spot"),
    ("range.fluence", "energy taken in by each square metre of the target"),
    ("range.burst_energy", "energy a burst sets free (its yield)"),
    ("range.xray_share", "share of a burst's energy that leaves as X-rays"),
    ("range.projectile_mass", "mass of a projectile"),
    ("range.closing_speed", "speed at which a projectile meets its target"),
    ("range.projectile_face", "area of a projectile's face"),
    ("range.impact_energy", "energy a projectile brings to its target"),
    ("range.skin_density", "density of a target's skin"),
    ("range.skin_thickness", "thickness of a target's skin"),
    ("range.heat_capacity", "heat to warm a kilogram of the skin by one kelvin"),
    ("range.start_temperature", "temperature of the skin before the hit"),
    ("range.heat_to_melt", "heat to bring a kilogram of the skin from 298 K to its melting point"),
    ("range.latent_fusion", "heat to melt a kilogram of the skin at its melting point"),
    ("range.heat_to_boil", "heat to bring a kilogram of the melted skin to its boiling point"),
    ("range.latent_vaporisation", "heat to boil a kilogram of the skin at its boiling point"),
    ("range.to_melt", "energy per square metre to melt through the skin"),
    ("range.to_boil", "energy per square metre to boil through the skin"),
    ("range.shot_speed", "speed of a shot (light speed for a beam)"),
    ("range.target_acceleration", "how hard the target can accelerate to dodge"),
    ("range.target_radius", "half the target's width as the shooter sees it"),
    ("range.delay", "time from the light that showed the target to the shot arriving"),
    ("range.wander", "how far the target could move while the shot was on its way"),
    ("range.odds", "chance of a hit"),
    ("range.arrives", "energy per square metre that arrives and is taken in"),
    ("range.takes", "energy per square metre the target can take before the effect"),
    ("range.ratio", "what arrives as a share of what is needed"),
    ("range.reading", "effect at range: 2 in range, 1 marginal, 0 out of range"),
    ("range.margin", "factor either way inside which an effect at range is marginal"),
    ("light.limb_cosine", "cosine between the line of sight and straight out of a surface: 1 at a disc's centre, 0 at its edge"),
    ("light.seen_temperature", "temperature of the layer a disc's light comes from at one point"),
    ("light.refractive_index", "how strongly a liquid or crystal bends light"),
    ("light.absorption_index", "the imaginary part of the refractive index: how strongly the material soaks up light inside it"),
    ("particle.scattering_efficiency", "light a particle scatters, over its cross-section"),
    ("light.bow_radius", "radius of a rainbow round the point opposite the sun, as an angle"),
    ("ocean.slope_variance", "mean square slope of the waves on a sea, which sets how far the sun's glint spreads"),
    ("light.normal_reflectance", "share of light a surface reflects straight back at normal incidence"),
    ("light.prism_angle", "angle between two faces of a crystal light passes through"),
    ("light.halo_radius", "radius of a halo round the sun or moon, as an angle"),
    ("light.minnaert_k", "Minnaert's k: how a lit surface's brightness falls toward its edge (1 is matt)"),
    ("light.incidence_cosine", "cosine of the angle sunlight arrives at on a surface"),
    ("light.brightness_share", "brightness of a lit surface against a matt one seen square on in full sun"),
    ("eye.linear_level", "light level of one screen colour, 0 to 1, in proportion to light"),
    ("eye.screen_level", "level sent to the screen, 0 to 1"),
    ("particle.radius", "radius of one particle"),
    ("particle.wavelength", "wavelength of the light"),
    ("particle.index_real", "refractive index of the particle's material (real part)"),
    ("particle.index_imaginary", "how strongly the material absorbs (imaginary part of the refractive index)"),
    ("particle.extinction_efficiency", "light taken out of the beam, per particle's cross-section"),
    ("particle.absorption_efficiency", "light absorbed, per particle's cross-section"),
    ("particle.single_scattering_albedo", "share of the light taken out that is scattered, not absorbed"),
    ("particle.optical_depth", "optical depth of a layer of particles straight down"),
    ("particle.asymmetry", "how far forward the particles scatter (0 alike every way, 1 straight on)"),
    ("slab.reflectance", "share of the light a layer sends back"),
    ("slab.transmission", "share of the light a layer lets through"),
    ("model.moment_of_inertia_factor", "moment of inertia over M R² (0.4 uniform, lower when mass sinks to the centre)"),
    ("shape.flattening", "how much shorter the pole-to-pole radius is, as a share of the equator's"),
    ("shape.equatorial_radius", "radius at the equator"),
    ("shape.polar_radius", "radius at the poles"),
    ("shape.spin_parameter", "spin's pull outward at the equator over gravity's pull in"),    // HF4b.0: the melt and lid blocks (formulas/melt.json).
    ("interior.parcel_temperature", "temperature of the rock"),
    ("melt.solidus_temperature", "where the rock starts to melt at this depth (solidus)"),
    ("melt.liquidus_temperature", "where the rock is wholly molten at this depth (liquidus)"),
    ("melt.fraction", "share of the rock that is molten"),
    ("interior.mantle_temperature", "temperature of the convecting mantle"),
    ("interior.creep_activation_energy", "how strongly the rock's stiffness depends on temperature (activation energy)"),
    ("heat.lid_base_flux", "heat conducted away from the lid's base"),
    ("heat.mantle_flux", "heat the convecting mantle delivers to the lid's base"),
    ("heat.intruded_flux", "heat of melt freezing inside the lid"),
    ("heat.lid_growth_rate", "how fast the lid's base moves down (negative: the lid thins)"),
    ("interior.thermal_conductivity", "how well the lid's rock conducts heat"),
    ("heat.lid_heating", "heat made inside the lid, per square metre of surface (its crust's radioactivity)"),
    ("heat.burial_speed", "how fast erupted lava buries the surface"),
    ("interior.thermal_diffusivity", "how fast heat spreads through the lid's rock"),
    ("heat.lid_surface_flux", "heat conducted out of the surface"),
    ("interior.mantle_depth", "depth of the convecting mantle"),
    ("heat.boundary_layer_temperature_drop", "temperature drop across the mantle's top boundary layer"),
    ("interior.convection_speed", "typical speed of the mantle's flow"),
    ("melt.zone_mean_fraction", "mean molten share across the melting zone"),
    ("melt.zone_volume", "volume of the melting zone under the lid"),
    ("melt.fertile_share", "share of the rock that can still melt (falls as basalt is taken)"),
    ("melt.production_rate", "melt made, volume per second"),
    ("heat.surplus_power", "heat the mantle gains beyond what it loses (only a gain melts rock)"),
    ("melt.heat_share", "share of that heat landing in the melting zone (its volume share; all of a tide's heat)"),
    ("melt.density", "density of the melt"),
    ("melt.zone_top_area", "area of the melting zone's top"),
    ("melt.viscosity", "how runny the melt is (its viscosity)"),
    ("melt.grain_size", "size of the rock's grains"),
    ("melt.density_contrast", "how much denser the rock is than its melt"),
    ("melt.permeability_exponent", "how fast the rock's permeability grows with its melt share"),
    ("melt.permeability_constant", "the permeability law's constant: permeability is b² φⁿ / C"),
    ("melt.lockup_share", "melt share at which the grains stop touching"),
    ("melt.retained_share", "share of the rock that is melt waiting to leave"),
    ("melt.weakening_exponent", "how strongly melt weakens the rock's flow"),
    ("time.step", "length of the step"),
    ("melt.processed_share", "share of the mantle that passed through the melting zone this step"),
    ("melt.kept_factor", "share of the mantle's fertile rock, water and heat-making elements left behind"),
    ("body.surface_area", "area of the world's surface"),
    ("melt.erupted_rate", "melt erupted at the surface, volume per second"),
    ("melt.intruded_rate", "melt frozen inside the lid, volume per second"),
    ("heat.melt_mantle_loss", "heat the mantle loses to the melt"),
    ("heat.melt_erupted", "heat erupted lava gives the surface"),
    ("heat.melt_intruded", "heat frozen melt gives the lid"),
    ("interior.andrade_exponent", "how the rock's slow creep spreads over time scales (Andrade exponent)"),
    ("tides.host_k2_over_q", "how strongly the host flexes and loses energy under the moon's tide (its k2/Q)"),
    ("tides.host_radius", "radius of the host"),
    ("tides.host_power_ceiling", "the most heat the host's tide can keep feeding the moon"),
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
        "particle.single_scattering_albedo" | "slab.reflectance" | "slab.transmission" => {
            Some((0.0, 1.0))
        }
        "particle.asymmetry" => Some((-1.0, 1.0)),
        "interior.core_mass_fraction" => Some((0.0, 1.0)),
        "planetesimal.water_kept_fraction" => Some((0.0, 1.0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
