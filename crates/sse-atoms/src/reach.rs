//! How a changed constant reaches each block (matter pack design section
//! 7A.3, MP1c). A block is declared here when it **follows**: it is a
//! formula, and every constant it uses is read from the one handle
//! (`sse_core::constants`), so it gives the right answer for any value.
//! A block not declared here **cannot follow**: it was computed or fitted
//! with the real constants and keeps the real-universe result. That is the
//! honest default, so a block moves up only with a test.
//!
//! **Estimated** (a tabulated result rescaled by a cited law, stamped
//! ESTIMATED) has no member yet: the main-sequence relations move there
//! when their homology scalings (Adams 2008) are built.

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
];

/// A block's reach: `None` is "cannot follow".
pub fn of(atom: &str) -> Option<Reach> {
    DECLARED.iter().find(|(a, _)| *a == atom).map(|(_, r)| *r)
}

/// What every block not declared says.
pub const CANNOT_FOLLOW: &str =
    "computed or fitted with the real constants: keeps the real-universe result, not following your change";
