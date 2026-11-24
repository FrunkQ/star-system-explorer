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
pub mod layers;
pub mod light;
pub mod magnetic;
pub mod matter;
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
    ("star.mass", "mass of a star"),
    ("orbit.angular_momentum_deficit", "angular momentum an orbit lacks against a flat circle of the same size (kg m2 s-1)"),
    ("orbit.ascending_node", "longitude of an orbit's ascending node, radians"),
    ("orbit.inclination", "tilt of an orbit against its host's reference plane, radians"),
    ("orbit.binary_limit", "widest (round one star) or nearest (round both) lasting planet orbit in a binary"),
    ("orbit.hill_gap", "inner apoapsis to outer periapsis, in mutual Hill radii"),
    ("orbit.hill_spacing", "gap between two neighbouring orbits, in mutual Hill radii"),
    ("orbit.kozai_time", "time for one Kozai-Lidov swing of an inner orbit"),
    ("orbit.mass_share", "a binary's mass ratio, the chosen star's share of the pair's mass"),
    ("orbit.moon_limit", "widest orbit a moon keeps round its planet"),
    ("orbit.mutual_hill_radius", "radius of the gravity bubble two neighbours share"),
    ("orbit.mutual_inclination", "angle between two orbital planes, radians"),
    ("orbit.relative_amd", "a system's angular momentum deficit against an outer planet's circular angular momentum"),
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
    // aurorae (MF9)
    ("aurora.field_line_distance", "distance at which a field line crosses the magnetic equator"),
    ("aurora.magnetic_latitude", "magnetic latitude at which a field line comes down to the surface"),
    ("ionosphere.pedersen_conductance", "effective height-integrated Pedersen conductance of one hemisphere's ionosphere"),
    ("body.equatorial_surface_field", "magnetic field strength at the surface on the magnetic equator"),
    ("magnetosphere.plasma_source_rate", "mass of plasma added to a magnetosphere each second (by a moon's volcanoes or geysers)"),
    ("magnetosphere.corotation_limit", "distance beyond which trapped plasma stops turning with the planet (Hill distance)"),
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
    ("range.beam_power", "power in a beam as it leaves its mirror or lens"),
    ("range.wavelength", "wavelength of a beam"),
    ("range.aperture", "width of the mirror or lens that sends a beam"),
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
    ("light.minnaert_k", "Minnaert's k: how a lit surface's brightness falls toward its edge (1 is matt)"),
    ("light.incidence_cosine", "cosine of the angle sunlight arrives at on a surface"),
    ("light.brightness_share", "brightness of a lit surface against a matt one seen square on in full sun"),
    ("eye.linear_level", "light level of one screen colour, 0 to 1, in proportion to light"),
    ("eye.screen_level", "level sent to the screen, 0 to 1"),
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
