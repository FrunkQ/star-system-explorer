//! Orbital elements and state vectors (Murray and Dermott 1999, Solar
//! System Dynamics, sections 2.4 to 2.8). Units: au, years, solar masses,
//! with G = 4 pi^2.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

pub const G: f64 = 4.0 * m::PI * m::PI; // not-sheet: G in AU^3 per solar mass per year^2 is 4 pi^2 by the choice of units (Kepler's third law), not a measured value

/// Heliocentric osculating elements; angles in radians.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Elements {
    pub a: f64,
    pub e: f64,
    pub i: f64,
    /// Longitude of the ascending node.
    pub node: f64,
    /// Argument of pericentre.
    pub peri: f64,
    /// Mean anomaly.
    pub mean: f64,
}

pub type V3 = [f64; 3];

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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
fn wrap(x: f64) -> f64 {
    let t = m::fmod(x, 2.0 * m::PI);
    if t < 0.0 {
        t + 2.0 * m::PI
    } else {
        t
    }
}

/// Eccentric anomaly from the mean anomaly: Newton's method from Danby's
/// starting value, a fixed 20 iterations (converged to rounding for e < 0.97).
pub fn eccentric_anomaly(mean: f64, e: f64) -> f64 {
    let mm = wrap(mean);
    let mut x = mm + 0.85 * e * if m::sin(mm) >= 0.0 { 1.0 } else { -1.0 };
    for _ in 0..20 {
        let f = x - e * m::sin(x) - mm;
        let fp = 1.0 - e * m::cos(x);
        x -= f / fp;
    }
    x
}

/// Position and velocity relative to the central body, `mu` = G (M + m).
pub fn to_state(el: &Elements, mu: f64) -> (V3, V3) {
    let ea = eccentric_anomaly(el.mean, el.e);
    let (se, ce) = (m::sin(ea), m::cos(ea));
    let b = el.a * m::sqrt(1.0 - el.e * el.e);
    let (x, y) = (el.a * (ce - el.e), b * se);
    let n = m::sqrt(mu / (el.a * el.a * el.a));
    let r = el.a * (1.0 - el.e * ce);
    let (vx, vy) = (-el.a * el.a * n * se / r, b * el.a * n * ce / r);
    let (cw, sw) = (m::cos(el.peri), m::sin(el.peri));
    let (cn, sn) = (m::cos(el.node), m::sin(el.node));
    let (ci, si) = (m::cos(el.i), m::sin(el.i));
    let rot = |px: f64, py: f64| -> V3 {
        [
            (cw * cn - sw * sn * ci) * px + (-sw * cn - cw * sn * ci) * py,
            (cw * sn + sw * cn * ci) * px + (-sw * sn + cw * cn * ci) * py,
            (sw * si) * px + (cw * si) * py,
        ]
    };
    (rot(x, y), rot(vx, vy))
}

/// Elements from a relative position and velocity.
pub fn from_state(r: V3, v: V3, mu: f64) -> Elements {
    let h = cross(r, v);
    let rn = norm(r);
    let hn = norm(h);
    let energy = 0.5 * dot(v, v) - mu / rn;
    let a = -mu / (2.0 * energy);
    let ev = {
        let vh = cross(v, h);
        [
            vh[0] / mu - r[0] / rn,
            vh[1] / mu - r[1] / rn,
            vh[2] / mu - r[2] / rn,
        ]
    };
    let e = norm(ev);
    let i = m::acos((h[2] / hn).clamp(-1.0, 1.0));
    let nvec = [-h[1], h[0], 0.0];
    let nn = norm(nvec);
    let node = if nn > 1e-300 {
        wrap(m::atan2(nvec[1], nvec[0]))
    } else {
        0.0
    };
    // Longitude-of-pericentre style angles that stay defined at i = 0.
    let peri = if nn > 1e-300 && e > 1e-300 {
        let c = (dot(nvec, ev) / (nn * e)).clamp(-1.0, 1.0);
        let w = m::acos(c);
        if ev[2] < 0.0 {
            2.0 * m::PI - w
        } else {
            w
        }
    } else if e > 1e-300 {
        wrap(m::atan2(ev[1], ev[0]))
    } else {
        0.0
    };
    let nu = if e > 1e-300 {
        let c = (dot(ev, r) / (e * rn)).clamp(-1.0, 1.0);
        let t = m::acos(c);
        if dot(r, v) < 0.0 {
            2.0 * m::PI - t
        } else {
            t
        }
    } else {
        0.0
    };
    let ea = 2.0
        * m::atan2(
            m::sqrt(1.0 - e) * m::sin(nu / 2.0),
            m::sqrt(1.0 + e) * m::cos(nu / 2.0),
        );
    let mean = wrap(ea - e * m::sin(ea));
    Elements {
        a,
        e,
        i,
        node,
        peri,
        mean,
    }
}
