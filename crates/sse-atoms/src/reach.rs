//! How a changed constant reaches each block (matter pack design section
//! 7A.3, MP1c). A block is declared here when it **follows**: it is a
//! formula, and every constant it uses is read from the one handle
//! (`sse_core::constants`), so it gives the right answer for any value.
//! A block not declared here **cannot follow**: it was computed or fitted
//! with the real constants and keeps the real-universe result. That is the
//! honest default, so a block moves up only with a test.
//!
//! **Estimated** (a fitted or tabulated result rescaled by a cited law,
//! stamped ESTIMATED): the main-sequence luminosity and radius, rescaled by
//! stellar homology (Adams 2008). The lifetime and the age brightening are
//! formulas on the luminosity, so they follow it.

/// One block's declared reach.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Reach {
    /// A formula: it reads these constants from the handle.
    Follows(&'static [&'static str]),
    /// A table rescaled by a cited law.
    Estimated {
        reads: &'static [&'static str],
        law: &'static str,
    },
}

/// The declared blocks, by atom id.
pub const DECLARED: &[(&str, Reach)] = &[
    ("sse.orbit.total_mass", Reach::Follows(&[])),
    ("sse.gravity.zero_velocity_radius", Reach::Follows(&["G"])),
    ("sse.orbit.period", Reach::Follows(&["G"])),
    (
        "sse.orbit.semi_major_axis_from_period",
        Reach::Follows(&["G"]),
    ),
    ("sse.orbit.apsides", Reach::Follows(&[])),
    ("sse.orbit.kepler_position", Reach::Follows(&[])),
    ("sse.orbit.hill_radius", Reach::Follows(&[])),
    ("sse.orbit.roche_limit", Reach::Follows(&[])),
    ("sse.planet.bulk_density", Reach::Follows(&[])),
    ("sse.planet.surface_gravity", Reach::Follows(&["G"])),
    ("sse.planet.escape_velocity", Reach::Follows(&["G"])),
    ("sse.tides.eccentricity_heating", Reach::Follows(&["G"])),
    ("sse.tides.locking_time", Reach::Follows(&["G"])),
    ("sse.tides.ocean_range", Reach::Follows(&[])),
    ("sse.tides.period", Reach::Follows(&[])),
    ("sse.ocean.wind_waves", Reach::Follows(&[])),
    ("sse.climate.insolation", Reach::Follows(&[])),
    (
        "sse.climate.equilibrium_temperature",
        Reach::Follows(&["sigma_SB"]),
    ),
    ("sse.climate.grey_surface_temperature", Reach::Follows(&[])),
    ("sse.atmosphere.scale_height", Reach::Follows(&["k_B"])),
    ("sse.atmosphere.surface_pressure", Reach::Follows(&[])),
    (
        "sse.atmosphere.jeans_parameter",
        Reach::Follows(&["G", "k_B"]),
    ),
    (
        "sse.star.effective_temperature",
        Reach::Follows(&["sigma_SB"]),
    ),
    (
        "sse.star.ms_luminosity",
        Reach::Estimated {
            reads: &["G", "sigma_SB", "k_B"],
            law: "stellar homology, L ∝ σ G⁴ M³ / k_B⁴ (Adams 2008, doi:10.1088/1475-7516/2008/08/010)",
        },
    ),
    (
        "sse.star.ms_radius",
        Reach::Estimated {
            reads: &["G"],
            law: "stellar homology, R ∝ G M at a fixed ignition energy (Adams 2008, doi:10.1088/1475-7516/2008/08/010)",
        },
    ),
    // t = t☉ (M/M☉)/(L/L☉): the energy per kilogram of fuel is nuclear, so
    // the lifetime moves only through the luminosity it is given.
    ("sse.star.ms_lifetime", Reach::Follows(&[])),
    ("sse.star.ms_luminosity_at_age", Reach::Follows(&[])),
    // MP17a: the Look up block gives whatever the stacked sheet holds, so a
    // changed constant or material value reaches it directly.
    ("sse.reference.lookup", Reach::Follows(&[])),
    // ND3: the maths blocks and the Equation hold no constant; every
    // constant reaches them on a wire from a Look up block, so they follow.
    ("sse.math.equation", Reach::Follows(&[])),
    ("math.power", Reach::Follows(&[])),
    ("math.sqrt", Reach::Follows(&[])),
    ("math.cbrt", Reach::Follows(&[])),
    ("math.exp", Reach::Follows(&[])),
    ("math.expm1", Reach::Follows(&[])),
    ("math.ln", Reach::Follows(&[])),
    ("math.log10", Reach::Follows(&[])),
    ("math.sin", Reach::Follows(&[])),
    ("math.cos", Reach::Follows(&[])),
    ("math.tan", Reach::Follows(&[])),
    ("math.atan2", Reach::Follows(&[])),
];

/// The cosmos chain's stages (DM1, galaxy histories design section 3.3).
/// They are not atoms: each is a step of `sse-cosmos` that reads the
/// sheet's Cosmos group through the handle (`sse_core::constants::cosmos`)
/// whenever the cosmology settings are read. The reach report lists them
/// beside the atoms, each in exactly one group.
pub const STAGES: &[(&str, Option<Reach>)] = &[
    (
        "sse.cosmos.expansion",
        Some(Reach::Follows(&["H0", "Omega_m", "T_cmb", "N_eff"])),
    ),
    ("sse.cosmos.growth", Some(Reach::Follows(&["Omega_m"]))),
    (
        "sse.cosmos.power_spectrum",
        Some(Reach::Follows(&[
            "H0", "Omega_m", "Omega_b", "n_s", "sigma8", "T_cmb",
        ])),
    ),
    ("sse.cosmos.web", Some(Reach::Follows(&["Omega_m", "sigma8"]))),
    ("sse.cosmos.epochs", Some(Reach::Follows(&["H0", "Omega_m", "Omega_b"]))),
    ("sse.cosmos.halo_counts", Some(Reach::Follows(&["Omega_m", "sigma8"]))),
    ("sse.cosmos.halo_growth", Some(Reach::Follows(&["Omega_m"]))),
    ("sse.cosmos.baryon_share", Some(Reach::Follows(&["Omega_b", "Omega_m"]))),
    (
        "sse.cosmos.major_mergers",
        Some(Reach::Estimated {
            reads: &["Omega_m", "sigma8"],
            law: "merger rate fitted in our universe, used unchanged on the changed halo counts (Fakhouri, Ma and Boylan-Kolchin 2010, MNRAS 406, 2267, arXiv:1001.2304)",
        }),
    ),
    (
        "sse.cosmos.subhaloes",
        Some(Reach::Estimated {
            reads: &["Omega_m", "sigma8"],
            law: "sub-halo counts and sinking times fitted in our universe, used unchanged (Giocoli et al. 2008; Boylan-Kolchin, Ma and Quataert 2008, MNRAS 383, 93, doi:10.1111/j.1365-2966.2007.12530.x)",
        }),
    ),
    // Tuned to the observed galaxy population: winds, depletion times, the
    // hot-halo cut, yields. They keep our universe's numbers.
    ("sse.cosmos.galaxy_regulator", None),
    ("sse.cosmos.ring_yields", None),
];

/// A stage's reach: `None` is "cannot follow".
pub fn of_stage(stage: &str) -> Option<Reach> {
    STAGES
        .iter()
        .find(|(s, _)| *s == stage)
        .and_then(|(_, r)| *r)
}

/// A block's reach: `None` is "cannot follow".
pub fn of(atom: &str) -> Option<Reach> {
    DECLARED.iter().find(|(a, _)| *a == atom).map(|(_, r)| *r)
}

/// What every block not declared says.
pub const CANNOT_FOLLOW: &str =
    "computed or fitted with the real constants: keeps the real-universe result, not following your change";
