//! Two-body orbits around the star: elements to state, universal-variable
//! propagation (any conic) and Lambert's problem. Deterministic: fixed
//! iteration counts, pinned maths, no fused multiply-add.
//!
//! References: Curtis 2014, Orbital Mechanics for Engineering Students, 3rd
//! ed., algorithms 3.4 (universal Kepler) and 5.2 (Lambert), Elsevier,
//! doi:10.1016/C2011-0-69685-1; Vallado 2013, Fundamentals of Astrodynamics
//! and Applications, 4th ed., Microcosm (algorithm 8 starting guesses).

use sse_core::math as m;

pub type V3 = [f64; 3];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn scale(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn norm(a: V3) -> f64 {
    m::sqrt(dot(a, a))
}

/// Newton steps for Kepler's equation and the universal anomaly.
pub const NEWTON_STEPS: u32 = 60;
/// Bisection steps for Lambert's z.
pub const LAMBERT_STEPS: u32 = 200;

/// Orbital elements around the star (angles in degrees, ecliptic frame).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Elements {
    pub a_m: f64,
    pub e: f64,
    pub i_deg: f64,
    pub node_deg: f64,
    /// Longitude of periapsis ϖ = Ω + ω.
    pub varpi_deg: f64,
    /// Mean longitude at the reference time.
    pub mean_longitude_deg: f64,
}

/// Solve M = E − e sin E (elliptic), M in radians.
pub fn eccentric_anomaly(mean: f64, e: f64) -> f64 {
    let mut mm = m::fmod(mean, m::TAU);
    if mm > m::PI {
        mm -= m::TAU;
    } else if mm <= -m::PI {
        mm += m::TAU;
    }
    let mut big_e = if e < 0.8 {
        mm
    } else if mm >= 0.0 {
        m::PI
    } else {
        -m::PI
    };
    for _ in 0..NEWTON_STEPS {
        let f = big_e - e * m::sin(big_e) - mm;
        let fp = 1.0 - e * m::cos(big_e);
        big_e -= f / fp;
    }
    big_e
}

/// Rotate a perifocal vector into the ecliptic frame.
fn to_ecliptic(p: V3, node: f64, argp: f64, inc: f64) -> V3 {
    let (co, so) = (m::cos(node), m::sin(node));
    let (cw, sw) = (m::cos(argp), m::sin(argp));
    let (ci, si) = (m::cos(inc), m::sin(inc));
    let r11 = co * cw - so * sw * ci;
    let r12 = -co * sw - so * cw * ci;
    let r21 = so * cw + co * sw * ci;
    let r22 = -so * sw + co * cw * ci;
    let r31 = sw * si;
    let r32 = cw * si;
    [
        r11 * p[0] + r12 * p[1],
        r21 * p[0] + r22 * p[1],
        r31 * p[0] + r32 * p[1],
    ]
}

/// Position and velocity at `dt_s` after the elements' reference time.
pub fn state(el: &Elements, mu: f64, dt_s: f64) -> (V3, V3) {
    let a = el.a_m;
    let e = el.e.clamp(0.0, 0.99);
    let n = m::sqrt(mu / (a * a * a));
    let varpi = el.varpi_deg.to_radians();
    let node = el.node_deg.to_radians();
    let mean = el.mean_longitude_deg.to_radians() - varpi + n * dt_s;
    let big_e = eccentric_anomaly(mean, e);
    let (ce, se) = (m::cos(big_e), m::sin(big_e));
    let b = m::sqrt(1.0 - e * e);
    let denom = 1.0 - e * ce;
    let p = [a * (ce - e), a * b * se, 0.0];
    let v = [-n * a * se / denom, n * a * b * ce / denom, 0.0];
    let inc = el.i_deg.to_radians();
    let argp = varpi - node;
    (
        to_ecliptic(p, node, argp, inc),
        to_ecliptic(v, node, argp, inc),
    )
}

/// Stumpff functions C(z), S(z).
pub fn stumpff(z: f64) -> (f64, f64) {
    if z > 1e-6 {
        let s = m::sqrt(z);
        // 1 − cos s = 2 sin²(s/2), exact near s = 2π where Lambert's bound lies.
        let h = m::sin(0.5 * s);
        (2.0 * h * h / z, (s - m::sin(s)) / (s * s * s))
    } else if z < -1e-6 {
        let s = m::sqrt(-z);
        ((m::cosh(s) - 1.0) / (-z), (m::sinh(s) - s) / (s * s * s))
    } else {
        (
            0.5 - z / 24.0 + z * z / 720.0,
            1.0 / 6.0 - z / 120.0 + z * z / 5040.0,
        )
    }
}

/// Conic summary of a state.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Conic {
    /// 1/a (negative when unbound), 1/m.
    pub alpha: f64,
    pub e: f64,
    pub periapsis_m: f64,
    /// Infinite when unbound.
    pub apoapsis_m: f64,
    pub period_s: Option<f64>,
    /// Moving inwards now.
    pub inbound: bool,
}

pub fn conic(r: V3, v: V3, mu: f64) -> Conic {
    let rn = norm(r);
    let v2 = dot(v, v);
    let alpha = 2.0 / rn - v2 / mu;
    let h = cross(r, v);
    let h2 = dot(h, h);
    let rv = dot(r, v);
    let ev = sub(scale(r, v2 / mu - 1.0 / rn), scale(v, rv / mu));
    let e = norm(ev);
    let p = h2 / mu;
    let periapsis = p / (1.0 + e);
    let (apo, period) = if alpha > 0.0 && e < 1.0 {
        let a = 1.0 / alpha;
        (a * (1.0 + e), Some(m::TAU * m::sqrt(a * a * a / mu)))
    } else {
        (f64::INFINITY, None)
    };
    Conic {
        alpha,
        e,
        periapsis_m: periapsis,
        apoapsis_m: apo,
        period_s: period,
        inbound: rv < 0.0,
    }
}

/// Propagate a state by `dt_s` on its conic (universal variables).
pub fn propagate(r0: V3, v0: V3, mu: f64, dt_s: f64) -> (V3, V3) {
    let rn = norm(r0);
    let v2 = dot(v0, v0);
    let alpha = 2.0 / rn - v2 / mu;
    let smu = m::sqrt(mu);
    let mut dt = dt_s;
    if alpha > 1e-20 {
        // Reduce by whole periods on an ellipse.
        let a = 1.0 / alpha;
        let period = m::TAU * m::sqrt(a * a * a / mu);
        dt = m::fmod(dt, period);
    }
    if dt == 0.0 {
        return (r0, v0);
    }
    let rv = dot(r0, v0) / smu;
    let mut chi = if alpha > 1e-20 {
        smu * dt * alpha
    } else if alpha < -1e-20 {
        let a = 1.0 / alpha;
        let sgn = if dt >= 0.0 { 1.0 } else { -1.0 };
        let num = -2.0 * mu * alpha * dt;
        let den = dot(r0, v0) + sgn * m::sqrt(-mu * a) * (1.0 - rn * alpha);
        let g = (num / den).max(1e-12);
        sgn * m::sqrt(-a) * m::ln(g)
    } else {
        smu * dt / rn
    };
    for _ in 0..NEWTON_STEPS {
        let z = alpha * chi * chi;
        let (c, s) = stumpff(z);
        let f = rv * chi * chi * c + (1.0 - alpha * rn) * chi * chi * chi * s + rn * chi - smu * dt;
        let fp = rv * chi * (1.0 - z * s) + (1.0 - alpha * rn) * chi * chi * c + rn;
        if fp == 0.0 {
            break;
        }
        chi -= f / fp;
    }
    let z = alpha * chi * chi;
    let (c, s) = stumpff(z);
    let f = 1.0 - chi * chi / rn * c;
    let g = dt - chi * chi * chi / smu * s;
    let r = add(scale(r0, f), scale(v0, g));
    let r1 = norm(r);
    let fd = smu / (r1 * rn) * (alpha * chi * chi * chi * s - chi);
    let gd = 1.0 - chi * chi / r1 * c;
    (r, add(scale(r0, fd), scale(v0, gd)))
}

/// Lambert's problem, single revolution, prograde: velocities at r1 and r2
/// for a flight of `tof_s`. None when the geometry is degenerate (the two
/// points opposite or coincident) or the time cannot be met.
pub fn lambert(r1: V3, r2: V3, tof_s: f64, mu: f64) -> Option<(V3, V3)> {
    let n1 = norm(r1);
    let n2 = norm(r2);
    let c = cross(r1, r2);
    let cos_dth = (dot(r1, r2) / (n1 * n2)).clamp(-1.0, 1.0);
    let mut dth = m::acos(cos_dth);
    if c[2] < 0.0 {
        dth = m::TAU - dth;
    }
    let sin_dth = m::sin(dth);
    if (1.0 - cos_dth) < 1e-10 || sin_dth.abs() < 1e-6 || tof_s <= 0.0 {
        return None;
    }
    let a = sin_dth * m::sqrt(n1 * n2 / (1.0 - cos_dth));
    let smu = m::sqrt(mu);
    let y = |z: f64| {
        let (cz, sz) = stumpff(z);
        n1 + n2 + a * (z * sz - 1.0) / m::sqrt(cz)
    };
    let ff = |z: f64| {
        let yz = y(z);
        if yz < 0.0 {
            return -smu * tof_s;
        }
        let (cz, sz) = stumpff(z);
        // (y/C)^1.5 as q √q: one square root instead of libm's pow (PF2).
        let q = yz / cz;
        q * m::sqrt(q) * sz + a * m::sqrt(yz) - smu * tof_s
    };
    let mut hi = 4.0 * m::PI * m::PI * (1.0 - 1e-9);
    if ff(hi) < 0.0 {
        return None;
    }
    let mut lo = -4.0 * m::PI * m::PI;
    let mut grow = 0;
    while ff(lo) > 0.0 && grow < 60 {
        lo *= 2.0;
        grow += 1;
    }
    if ff(lo) > 0.0 {
        return None;
    }
    for _ in 0..LAMBERT_STEPS {
        let mid = 0.5 * (lo + hi);
        if ff(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let z = 0.5 * (lo + hi);
    let yz = y(z);
    if yz <= 0.0 {
        return None;
    }
    let f = 1.0 - yz / n1;
    let g = a * m::sqrt(yz / mu);
    let gd = 1.0 - yz / n2;
    let v1 = scale(sub(r2, scale(r1, f)), 1.0 / g);
    let v2 = scale(sub(scale(r2, gd), r1), 1.0 / g);
    Some((v1, v2))
}
