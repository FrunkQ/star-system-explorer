//! The figure of a spinning, self-gravitating body that has relaxed like a
//! fluid (Chandrasekhar 1969): a Maclaurin spheroid up to the bifurcation,
//! a Jacobi ellipsoid beyond it. The spin parameter is q = Omega^2 / (pi G rho).
//!
//! This is the breakup-side companion of the shared flattening block
//! `sse.shape.spin_flattening` (Darwin-Radau, good to small q): it covers the
//! fast, strongly flattened end that small bodies and Haumea-like dwarfs reach.
//! It moves into a formula block `sse.shape.spin_figure` with SB2.

use sse_core::math as m;

/// Maclaurin q at the Jacobi bifurcation (e = 0.81267), Chandrasekhar 1969.
pub const Q_BIFURCATION: f64 = 0.374_23;
/// The largest q any Maclaurin spheroid reaches (e = 0.92996).
pub const Q_MACLAURIN_MAX: f64 = 0.449_33;
/// Jacobi sequence end (the pear-shaped point), b/a = 0.432, c/a = 0.345.
pub const Q_JACOBI_END: f64 = 0.284_03;

/// q of a Maclaurin spheroid of eccentricity e.
pub fn maclaurin_q(e: f64) -> f64 {
    if e < 1e-4 {
        // Series: q = 8 e^2 / 15 to leading order.
        return 8.0 * e * e / 15.0;
    }
    let s = m::sqrt(1.0 - e * e);
    2.0 * s * (3.0 - 2.0 * e * e) * m::asin(e) / (e * e * e) - 6.0 * (1.0 - e * e) / (e * e)
}

/// c/a of the Maclaurin spheroid spinning at q (the slow branch, e below the
/// bifurcation); q above the bifurcation is clamped to it.
pub fn maclaurin_c_over_a(q: f64) -> f64 {
    let q = q.clamp(0.0, Q_BIFURCATION);
    let (mut lo, mut hi) = (0.0, 0.812_67);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if maclaurin_q(mid) < q {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let e = 0.5 * (lo + hi);
    m::sqrt(1.0 - e * e)
}

/// b/a and c/a of a relaxed body too fast for a stable Maclaurin figure: the
/// Jacobi sequence, taken as a straight line from the bifurcation (1, 0.5827)
/// to its end (0.432, 0.345) as q runs from the bifurcation to the Maclaurin
/// maximum (fudge SB-jacobi-line).
pub fn jacobi_axes(q: f64) -> (f64, f64) {
    let t = ((q - Q_BIFURCATION) / (Q_MACLAURIN_MAX - Q_BIFURCATION)).clamp(0.0, 1.0);
    (1.0 - 0.568 * t, 0.5827 - 0.2377 * t)
}

/// (b/a, c/a, name) of the relaxed figure at q.
pub fn relaxed(q: f64) -> (f64, f64, &'static str) {
    if q <= Q_BIFURCATION {
        (1.0, maclaurin_c_over_a(q), "maclaurin")
    } else {
        let (b, c) = jacobi_axes(q);
        (b, c, "jacobi")
    }
}

/// The fastest spin a relaxed body holds, hours: the Maclaurin maximum.
pub fn relaxed_fastest_hours(rho: f64) -> f64 {
    let omega = m::sqrt(Q_MACLAURIN_MAX * m::PI * sse_core::sheet::G * rho);
    2.0 * m::PI / omega / 3600.0
}

/// The spin barrier of a strengthless sphere, hours: sqrt(3 pi / (G rho)).
pub fn barrier_hours(rho: f64) -> f64 {
    m::sqrt(3.0 * m::PI / (sse_core::sheet::G * rho)) / 3600.0
}

/// q for a spin period in hours.
pub fn q_of(spin_hours: f64, rho: f64) -> f64 {
    let omega = 2.0 * m::PI / (spin_hours * 3600.0);
    omega * omega / (m::PI * sse_core::sheet::G * rho)
}
