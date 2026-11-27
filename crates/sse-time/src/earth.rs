//! Which way the Earth faces at a date (RW17; real-catalogues design 6A.2):
//! where on the ground a shadow falls. Read from
//! `packs/base-sol/motion/earth-orientation.json`, every number cited there.
//!
//! The series run on an even clock (TT). The Earth's spin does not: it
//! slows unevenly, and the gap, Delta T = TT - UT1, is measured for the last
//! few centuries and estimated outside them. So far from now an eclipse's
//! time stays good while the place it is seen from drifts; `delta_t` gives
//! the gap and how far it may be off, and `drift_deg` turns that into
//! degrees of longitude.

use crate::series::J2000_JD;
use serde::Deserialize;
use sse_core::math as m;

#[derive(Clone, Debug, Deserialize)]
struct Era {
    turns_at_j2000: f64,
    turns_per_ut1_day: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct Poly {
    poly: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct Value {
    v: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct Precession {
    zeta_a: Vec<f64>,
    z_a: Vec<f64>,
    theta_a: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct Piece {
    from: f64,
    to: f64,
    centre: f64,
    scale: f64,
    poly: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct Uncertainty {
    measured_s: f64,
    past_s_per_century2: f64,
    past_centre: f64,
    future_s_per_century2: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct DeltaT {
    pieces: Vec<Piece>,
    measured_years: [f64; 2],
    uncertainty: Uncertainty,
}

#[derive(Clone, Debug, Deserialize)]
struct Doc {
    format: String,
    /// The model's id, as a body's `spin.model` names it.
    #[serde(default)]
    id: String,
    era: Era,
    gmst_minus_era_arcsec: Poly,
    obliquity_j2000_arcsec: Value,
    precession_arcsec: Precession,
    delta_t: DeltaT,
}

/// The Earth's orientation model.
#[derive(Clone, Debug)]
pub struct EarthOrientation {
    d: Doc,
}

const ARCSEC: f64 = m::PI / 648_000.0;
const JULIAN_YEAR_D: f64 = 365.25; // not-sheet: the Julian year of 365.25 days is a definition of the calendar the series count in
const CENTURY_D: f64 = 36_525.0; // not-sheet: the Julian century, 100 Julian years, by definition

fn horner(c: &[f64], t: f64) -> f64 {
    let mut s = 0.0;
    for &x in c.iter().rev() {
        s = s * t + x;
    }
    s
}

fn rot1(a: f64, v: [f64; 3]) -> [f64; 3] {
    let (c, s) = (m::cos(a), m::sin(a));
    [v[0], c * v[1] + s * v[2], -s * v[1] + c * v[2]]
}
fn rot2(a: f64, v: [f64; 3]) -> [f64; 3] {
    let (c, s) = (m::cos(a), m::sin(a));
    [c * v[0] - s * v[2], v[1], s * v[0] + c * v[2]]
}
fn rot3(a: f64, v: [f64; 3]) -> [f64; 3] {
    let (c, s) = (m::cos(a), m::sin(a));
    [c * v[0] + s * v[1], -s * v[0] + c * v[1], v[2]]
}

/// A place on the ground and how sure it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ground {
    /// Geodetic latitude, degrees north.
    pub lat_deg: f64,
    /// Longitude, degrees east in (-180, 180].
    pub lon_deg: f64,
}

impl EarthOrientation {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let d: Doc = serde_json::from_str(text).map_err(|e| format!("earth orientation: {e}"))?;
        if d.format != "sse.earth-orientation/1" {
            return Err(format!("earth orientation: format {}", d.format));
        }
        Ok(EarthOrientation { d })
    }

    /// The model's id, as a body's `spin.model` names it.
    pub fn id(&self) -> &str {
        &self.d.id
    }

    /// The decimal year of a Julian date.
    pub fn year(jd: f64) -> f64 {
        2000.0 + (jd - J2000_JD) / JULIAN_YEAR_D
    }

    /// Delta T = TT - UT1 at decimal year `y`, seconds, and how far it may be
    /// off (seconds, one sigma).
    pub fn delta_t(&self, y: f64) -> (f64, f64) {
        let dt = &self.d.delta_t;
        let p = dt
            .pieces
            .iter()
            .find(|p| y >= p.from && y < p.to)
            .unwrap_or(&dt.pieces[0]);
        let v = horner(&p.poly, (y - p.centre) / p.scale);
        let u = &dt.uncertainty;
        let [a, b] = dt.measured_years;
        let sigma = if y < a {
            let c = (y - u.past_centre) / 100.0;
            u.measured_s.max(u.past_s_per_century2 * c * c)
        } else if y > b {
            let c = (y - b) / 100.0;
            u.measured_s + u.future_s_per_century2 * c * c
        } else {
            u.measured_s
        };
        (v, sigma)
    }

    /// How far east or west a ground place may be off at `jd` (TT), degrees,
    /// from the uncertainty of Delta T alone.
    pub fn drift_deg(&self, jd: f64) -> f64 {
        let (_, s) = self.delta_t(Self::year(jd));
        s * self.d.era.turns_per_ut1_day * 360.0 / sse_core::sheet::DAY
    }

    /// The Greenwich mean sidereal angle at `jd` (TT), radians, with UT1 from
    /// Delta T.
    pub fn gmst(&self, jd_tt: f64) -> f64 {
        let (dt, _) = self.delta_t(Self::year(jd_tt));
        let du = jd_tt - dt / sse_core::sheet::DAY - J2000_JD;
        let e = &self.d.era;
        let turns = e.turns_at_j2000 + e.turns_per_ut1_day * du;
        let era = m::TAU * (turns - m::floor(turns));
        let t = (jd_tt - J2000_JD) / CENTURY_D;
        era + horner(&self.d.gmst_minus_era_arcsec.poly, t) * ARCSEC
    }

    /// A J2000 ecliptic vector (from Earth's centre) in the Earth's own
    /// frame at `jd` (TT): x to Greenwich on the equator, z to the north pole.
    pub fn to_earth_fixed(&self, v: [f64; 3], jd_tt: f64) -> [f64; 3] {
        let eps = self.d.obliquity_j2000_arcsec.v * ARCSEC;
        let eq = rot1(-eps, v);
        let t = (jd_tt - J2000_JD) / CENTURY_D;
        let p = &self.d.precession_arcsec;
        let (zeta, z, theta) = (
            horner(&p.zeta_a, t) * ARCSEC,
            horner(&p.z_a, t) * ARCSEC,
            horner(&p.theta_a, t) * ARCSEC,
        );
        let of_date = rot3(-z, rot2(theta, rot3(-zeta, eq)));
        rot3(self.gmst(jd_tt), of_date)
    }

    /// The other way: an Earth-fixed vector back to the J2000 ecliptic.
    pub fn from_earth_fixed(&self, v: [f64; 3], jd_tt: f64) -> [f64; 3] {
        let eps = self.d.obliquity_j2000_arcsec.v * ARCSEC;
        let t = (jd_tt - J2000_JD) / CENTURY_D;
        let p = &self.d.precession_arcsec;
        let (zeta, z, theta) = (
            horner(&p.zeta_a, t) * ARCSEC,
            horner(&p.z_a, t) * ARCSEC,
            horner(&p.theta_a, t) * ARCSEC,
        );
        let of_date = rot3(-self.gmst(jd_tt), v);
        rot1(eps, rot3(zeta, rot2(-theta, rot3(z, of_date))))
    }

    /// The geodetic place under an Earth-fixed point on (or near) an
    /// ellipsoid of polar to equatorial radius ratio `axis_ratio`.
    pub fn ground(p: [f64; 3], axis_ratio: f64) -> Ground {
        let rho = m::hypot(p[0], p[1]);
        let lat = m::atan2(p[2], axis_ratio * axis_ratio * rho);
        let mut lon = m::atan2(p[1], p[0]) * 180.0 / m::PI;
        if lon <= -180.0 {
            lon += 360.0;
        }
        Ground {
            lat_deg: lat * 180.0 / m::PI,
            lon_deg: lon,
        }
    }
}
