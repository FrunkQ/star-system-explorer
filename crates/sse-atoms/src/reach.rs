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
];

/// A block's reach: `None` is "cannot follow".
pub fn of(atom: &str) -> Option<Reach> {
    DECLARED.iter().find(|(a, _)| *a == atom).map(|(_, r)| *r)
}

/// What every block not declared says.
pub const CANNOT_FOLLOW: &str =
    "computed or fitted with the real constants: keeps the real-universe result, not following your change";
