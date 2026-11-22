//! Where aurorae light up (magnetic fields design 6.5, stage MF9 part 1).
//! An aurora sits where currents run along field lines into the air. Two
//! drivers make them, each computed and kept if it lands inside the bubble:
//! the star's wind, at the boundary between closed and open field lines
//! (the line through the bubble's nose), and the planet's own spin, where
//! plasma loaded by a moon can no longer keep up with the turning field
//! (the Hill distance). A moon's own field line lands as a spot.

use crate::common::*;
use crate::light::TESLA;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const KR95: &str = "Kivelson, M. G. and Russell, C. T. (eds) 1995, Introduction to Space Physics, Cambridge University Press: a dipole field line obeys r = L R cos²λ, so the line crossing the magnetic equator at L R reaches the surface at cos²λ = 1/L. https://www.cambridge.org/core/books/introduction-to-space-physics/BA017948E00BF032AA06D8D2BFD8062F";
const FS67: &str = "Feldstein, Y. I. and Starkov, G. V. 1967, Dynamics of auroral belt and polar geomagnetic disturbances, Planet. Space Sci. 15, 209: the auroral oval near 67° to 75° magnetic latitude, moving equatorward as activity rises. doi:10.1016/0032-0633(67)90190-0";
const HILL79: &str = "Hill, T. W. 1979, Inertial limit on corotation, J. Geophys. Res. 84, 6554: plasma loaded at Ṁ stops corotating beyond ρ_H, where the ionosphere's Pedersen current can no longer supply the torque. doi:10.1029/JA084iA11p06554";
const NICH11: &str = "Nichols, J. D. 2011, Magnetosphere-ionosphere coupling in Jupiter's middle magnetosphere, J. Geophys. Res. 116 (arXiv:1108.3219), eq. 24: ρ_H / R = (2π Σ_P* B² R² / Ṁ)^¼ with Σ_P* the effective Pedersen conductance of one hemisphere; about 30 R_J for Jupiter. https://arxiv.org/abs/1108.3219";

/// Magnetic latitude (radians) at which the dipole field line crossing the
/// magnetic equator at distance `r` reaches a sphere of radius `radius`.
/// Zero if the line never rises above the sphere.
pub fn footprint_latitude(r: f64, radius: f64) -> f64 {
    if r <= radius || radius <= 0.0 {
        return 0.0;
    }
    m::acos(m::sqrt(radius / r))
}

/// Hill distance, m: where plasma loaded at `mdot` (kg/s) stops corotating
/// with a dipole of equatorial surface field `b_eq` (T) on a planet of
/// radius `radius`, for effective Pedersen conductance `sigma` (S).
pub fn hill_distance(sigma: f64, b_eq: f64, radius: f64, mdot: f64) -> f64 {
    if mdot <= 0.0 || sigma <= 0.0 {
        return f64::INFINITY;
    }
    radius
        * m::pow(
            2.0 * m::PI * sigma * b_eq * b_eq * radius * radius / mdot,
            0.25,
        )
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let conductance = D([-2, -1, 3, 2, 0, 0, 0]);
    let mass_rate = D([0, 1, -1, 0, 0, 0, 0]);
    vec![
        Def {
            id: "sse.aurora.footprint",
            version: "1.0.0",
            inputs: vec![
                port("distance", "aurora.field_line_distance", D::LENGTH, None),
                port("radius", "body.radius", D::LENGTH, None),
            ],
            outputs: vec![port("latitude", "aurora.magnetic_latitude", D::NONE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[KR95, FS67],
            curious: "Aurorae glow where field lines from far out in space come down into the air. The further out a line reaches, the nearer the pole it lands, so aurorae make a ring round each magnetic pole.",
            science: "A dipole line that crosses the magnetic equator at L planet radii reaches the surface at cos²λ = 1/L. The wind-driven oval is the footprint of the last closed line, near the bubble's nose; a moon's spot is the footprint of the line through the moon.",
            formula: "λ = arccos √(R / r)",
            f: |i, _| Ok(vec![footprint_latitude(i[0].max(0.0), positive(i[1], "radius")?)]),
        }
        .build(),
        Def {
            id: "sse.aurora.hill_distance",
            version: "1.0.0",
            inputs: vec![
                port("conductance", "ionosphere.pedersen_conductance", conductance, None),
                port("field", "body.equatorial_surface_field", TESLA, None),
                port("radius", "body.radius", D::LENGTH, None),
                port("loading", "magnetosphere.plasma_source_rate", mass_rate, None),
            ],
            outputs: vec![port("distance", "magnetosphere.corotation_limit", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[HILL79, NICH11],
            curious: "A fast-spinning giant drags its trapped gas round with it. When a moon like Io pours in a tonne of gas a second, the far gas lags behind, and the currents that try to pull it along light the giant's brightest aurora.",
            science: "Plasma loaded at Ṁ corotates while the ionosphere's Pedersen current can supply its angular momentum; beyond the Hill distance it lags, and the field-aligned currents of the corotation-enforcement system make the main oval.",
            formula: "ρ_H = R (2π Σ_P* B² R² / Ṁ)^¼",
            f: |i, _| {
                Ok(vec![hill_distance(
                    i[0].max(0.0),
                    i[1].max(0.0),
                    positive(i[2], "radius")?,
                    i[3].max(0.0),
                )])
            },
        }
        .build(),
    ]
}
