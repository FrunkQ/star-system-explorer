//! Orbital forcing and ice ages (W9 part 2, weather review section 4): how a
//! world's tilt, eccentricity and perihelion wander over thousands of years,
//! the summer sunshine that follows, and how often its glaciers advance.
//!
//! - **The orbit's history** comes in as an [`OrbitTrack`]: eccentricity,
//!   perihelion, inclination and node through time, from T1's
//!   Laplace-Lagrange secular theory (`sse-time::secular`) for the world's
//!   planetary siblings, or fixed when the world has none (a lone world has
//!   no orbital cycles).
//! - **The spin axis** precesses about the orbit's pole (Zeebe 2022 eqs. 1 to 5):
//!   ṡ = α (n · s)(s × n), cos ε = n · s, α = K (κ + β),
//!   K = (3/2) E_d GM / (ω a³), κ = (1 − e²)^(−3/2), β = Σ g_L (a / a_L)³ (m_L / M)
//!   over the world's moons. E_d, the dynamical ellipticity, is Earth's
//!   0.00327381 scaled by the square of the spin (a hydrostatic bulge grows
//!   as ω²; a registered fudge) unless given. Integrated by RK4 in the fixed
//!   frame, so the tilt's beat with the orbit's wobble (Earth's 41 kyr) and
//!   the drift of perihelion against the equinox (its climatic precession)
//!   come out of the equation, not a table.
//! - **Summer energy** at a latitude (Huybers 2006): the sunshine summed over
//!   the days whose mean insolation is at least `summer_threshold_w_m2`
//!   (275 W/m², near where temperature crosses 0 °C).
//! - **Ice ages**: the ice balance at a place (`water.rs`) responds linearly
//!   to its summer energy, measured from the climate and one copy of it at a
//!   tilt `probe_obliquity_deg` lower (a registered fudge: no ice-albedo or
//!   ice-sheet feedback). Each upward crossing of zero in the window is one
//!   glacial advance; the share of the window above zero is how long the
//!   place lies under ice.

use crate::climate::Climate;
use crate::conditions::Conditions;
use crate::ebm;
use crate::settings::ClimateSettings;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const G: f64 = 6.6743e-11;
const AU_M: f64 = 1.495_978_707e11;
const M_SUN_KG: f64 = 1.988_47e30;
const YEAR_S: f64 = 3.15576e7;
const EARTH_OMEGA: f64 = 7.292_115e-5;

type V3 = [f64; 3];

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn scale(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn unit(a: V3) -> V3 {
    let l = m::sqrt(dot(a, a)).max(1e-300);
    scale(a, 1.0 / l)
}

/// The orbit at one time (fixed frame; angles in radians).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct TrackSample {
    pub t_yr: f64,
    pub e: f64,
    /// Longitude of perihelion, node and inclination in the fixed frame.
    pub varpi: f64,
    pub incl: f64,
    pub node: f64,
}

/// The orbit through time, evenly sampled, and where it came from.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OrbitTrack {
    pub samples: Vec<TrackSample>,
    pub source: String,
}

impl OrbitTrack {
    /// A lone world's orbit: the same at every time.
    pub fn fixed(e: f64, span_yr: f64) -> Self {
        let s = |t| TrackSample {
            t_yr: t,
            e,
            varpi: 0.0,
            incl: 0.0,
            node: 0.0,
        };
        OrbitTrack {
            samples: vec![s(0.0), s(span_yr)],
            source: "fixed (no siblings to perturb the orbit)".into(),
        }
    }

    /// The orbit at time `t`, linear between samples ((k, h) and (q, p)
    /// interpolated, so angles never jump).
    pub fn at(&self, t: f64) -> TrackSample {
        let v = &self.samples;
        let n = v.len();
        if n == 1 || t <= v[0].t_yr {
            return v[0];
        }
        if t >= v[n - 1].t_yr {
            return v[n - 1];
        }
        let dt = (v[n - 1].t_yr - v[0].t_yr) / (n - 1) as f64;
        let i = (m::floor((t - v[0].t_yr) / dt) as usize).min(n - 2);
        let f = (t - v[i].t_yr) / dt;
        let (a, b) = (v[i], v[i + 1]);
        let mix = |x: f64, y: f64| x + (y - x) * f;
        let (k, h) = (
            mix(a.e * m::cos(a.varpi), b.e * m::cos(b.varpi)),
            mix(a.e * m::sin(a.varpi), b.e * m::sin(b.varpi)),
        );
        let (q, p) = (
            mix(a.incl * m::cos(a.node), b.incl * m::cos(b.node)),
            mix(a.incl * m::sin(a.node), b.incl * m::sin(b.node)),
        );
        TrackSample {
            t_yr: t,
            e: m::hypot(k, h),
            varpi: m::atan2(h, k),
            incl: m::hypot(q, p),
            node: m::atan2(p, q),
        }
    }

    pub fn span_yr(&self) -> f64 {
        self.samples.last().map(|s| s.t_yr).unwrap_or(0.0) - self.samples[0].t_yr
    }
}

/// The orbit's pole and perihelion direction for a sample.
fn frame(s: &TrackSample) -> (V3, V3) {
    let (ci, si) = (m::cos(s.incl), m::sin(s.incl));
    let (cn, sn) = (m::cos(s.node), m::sin(s.node));
    let n = [si * sn, -si * cn, ci];
    // Argument of perihelion from the node, in the orbit plane.
    let w = s.varpi - s.node;
    let (cw, sw) = (m::cos(w), m::sin(w));
    let p = [cn * cw - sn * sw * ci, sn * cw + cn * sw * ci, sw * si];
    (n, unit(p))
}

/// A moon's pull on the bulge: its mass and distance.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct MoonPull {
    pub mass_kg: f64,
    pub a_m: f64,
}

/// K and β of the precession constant α = K (κ + β), rad/yr.
pub fn precession_parts(
    cs: &ClimateSettings,
    c: &Conditions,
    star_kg: f64,
    a_au: f64,
    moons: &[MoonPull],
) -> (f64, f64) {
    let f = &cs.forcing;
    let w = c.spin_rate().max(1e-12);
    let ed = f.earth_ellipticity * m::pow(w / EARTH_OMEGA, f.ellipticity_spin_power);
    let a = a_au * AU_M;
    let k = 1.5 * ed * G * star_kg / (w * a * a * a) * YEAR_S;
    let beta: f64 = moons
        .iter()
        .map(|mo| {
            let r = a / mo.a_m.max(1.0);
            f.moon_geometry * r * r * r * mo.mass_kg / star_kg
        })
        .sum();
    (k, beta)
}

/// The tilt and perihelion at one time.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct SpinSample {
    pub t_yr: f64,
    pub obliquity_deg: f64,
    pub e: f64,
    /// Solar longitude of perihelion, from the northern spring equinox.
    pub perihelion_lon_deg: f64,
}

/// The spin axis through the track, from tilt `eps0_deg` and perihelion
/// `peri0_deg` (solar longitude) at its start, with precession parts (K, β)
/// in rad/yr. Samples every `sample_yr`, RK4 steps of `step_yr`.
pub fn spin_history(
    track: &OrbitTrack,
    eps0_deg: f64,
    peri0_deg: f64,
    k_beta: (f64, f64),
    step_yr: f64,
    sample_yr: f64,
) -> Vec<SpinSample> {
    let t0 = track.samples[0].t_yr;
    let span = track.span_yr();
    let (k, beta) = k_beta;
    let alpha = |t: f64| {
        let e = track.at(t).e.min(0.99);
        k * (m::pow(1.0 - e * e, -1.5) + beta)
    };
    let pole = |t: f64| frame(&track.at(t)).0;
    // Start: the equinox direction γ sits θ = peri − 180° behind perihelion.
    let first = track.at(t0);
    let (n0, p0) = frame(&first);
    let th = (peri0_deg - 180.0) * m::PI / 180.0;
    let gamma = unit(add(
        scale(p0, m::cos(th)),
        scale(cross(n0, p0), -m::sin(th)),
    ));
    let e0 = eps0_deg * m::PI / 180.0;
    let mut s = unit(add(
        scale(n0, m::cos(e0)),
        scale(cross(n0, gamma), m::sin(e0)),
    ));
    let ds = |t: f64, s: V3| {
        let n = pole(t);
        scale(cross(s, n), alpha(t) * dot(n, s))
    };
    let read = |t: f64, s: V3| {
        let tr = track.at(t);
        let (n, p) = frame(&tr);
        let eps = m::acos(dot(n, s).clamp(-1.0, 1.0));
        let g = unit(cross(s, n));
        let th = m::atan2(dot(p, cross(n, g)), dot(p, g));
        let mut peri = th * 180.0 / m::PI + 180.0;
        peri -= 360.0 * m::floor(peri / 360.0);
        SpinSample {
            t_yr: t,
            obliquity_deg: eps * 180.0 / m::PI,
            e: tr.e,
            perihelion_lon_deg: peri,
        }
    };
    let h = step_yr.max(1.0);
    let per = (m::round(sample_yr / h) as usize).max(1);
    let steps = m::ceil(span / h) as usize;
    let mut out = vec![read(t0, s)];
    let mut t = t0;
    for i in 1..=steps {
        let k1 = ds(t, s);
        let k2 = ds(t + h / 2.0, add(s, scale(k1, h / 2.0)));
        let k3 = ds(t + h / 2.0, add(s, scale(k2, h / 2.0)));
        let k4 = ds(t + h, add(s, scale(k3, h)));
        let d = add(add(k1, scale(k2, 2.0)), add(scale(k3, 2.0), k4));
        s = unit(add(s, scale(d, h / 6.0)));
        t += h;
        if i % per == 0 {
            out.push(read(t, s));
        }
    }
    out
}

/// Huybers's summer energy at `lat_deg`: sunlight summed over the days whose
/// mean insolation is at least `threshold`, J/m².
pub fn summer_energy(c: &Conditions, lat_deg: f64, threshold: f64, steps: u32) -> f64 {
    let n = steps.max(4);
    let dt = c.year_days.max(1e-6) * 86_400.0 / n as f64;
    let eps = c.obliquity_deg * m::PI / 180.0;
    let mut j = 0.0;
    for k in 0..n {
        let (lon, f) = ebm::orbit_at(c, (k as f64 + 0.5) / n as f64);
        let decl = m::asin(m::sin(eps) * m::sin(lon * m::PI / 180.0)) * 180.0 / m::PI;
        let q = ebm::insolation(c.flux_w_m2 * f, lat_deg, decl);
        if q >= threshold {
            j += q * dt;
        }
    }
    j
}

/// The conditions with a sample's tilt, eccentricity and perihelion.
pub fn with_spin(c: &Conditions, s: &SpinSample) -> Conditions {
    let mut x = c.clone();
    x.obliquity_deg = s.obliquity_deg;
    x.eccentricity = s.e;
    x.perihelion_lon_deg = s.perihelion_lon_deg;
    x
}

/// The ice-age response of a world: its spin history, the climate at a
/// lower tilt for the sensitivity, and each band's summer energy through time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct IceAgeModel {
    pub source: String,
    pub spin: Vec<SpinSample>,
    pub probe: Climate,
    /// Summer energy now and at the probe, by band, J/m².
    pub energy_now: Vec<f64>,
    pub energy_probe: Vec<f64>,
    /// Summer energy by sample, then band, and each band's range.
    pub energy: Vec<Vec<f64>>,
    pub energy_min: Vec<f64>,
    pub energy_max: Vec<f64>,
}

/// The ice-age response of `cl` under `spin` (from [`spin_history`]).
pub fn ice_age_model(
    cs: &ClimateSettings,
    cl: &Climate,
    epoch_myr: f64,
    spin: Vec<SpinSample>,
    source: &str,
) -> IceAgeModel {
    let f = &cs.forcing;
    let c = &cl.conditions;
    let mut pc = c.clone();
    pc.obliquity_deg = (c.obliquity_deg - f.probe_obliquity_deg).max(0.0);
    let probe = crate::climate::orbit(cs, &pc, epoch_myr);
    let band = |cc: &Conditions| -> Vec<f64> {
        cl.lat_deg
            .iter()
            .map(|l| summer_energy(cc, *l, f.summer_threshold_w_m2, f.year_steps))
            .collect()
    };
    let energy_now = band(c);
    let energy_probe = band(&pc);
    let energy: Vec<Vec<f64>> = spin.iter().map(|s| band(&with_spin(c, s))).collect();
    let nb = cl.lat_deg.len();
    let energy_min = (0..nb)
        .map(|j| energy.iter().map(|e| e[j]).fold(f64::INFINITY, f64::min))
        .collect();
    let energy_max = (0..nb)
        .map(|j| {
            energy
                .iter()
                .map(|e| e[j])
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect();
    IceAgeModel {
        source: source.to_string(),
        spin,
        probe,
        energy_now,
        energy_probe,
        energy,
        energy_min,
        energy_max,
    }
}

/// The ice ages at one place.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct IceAges {
    /// Glacial advances in the window, and per million years.
    pub advances: u32,
    pub advances_per_myr: f64,
    /// Share of the window the place lies under growing ice.
    pub glaciated_share: f64,
    /// The ice balance's range through the window, m a year.
    pub balance_min_m_yr: f64,
    pub balance_max_m_yr: f64,
}

impl IceAgeModel {
    /// The tilt's range through the window, degrees.
    pub fn obliquity_range(&self) -> (f64, f64) {
        self.spin
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), s| {
                (a.min(s.obliquity_deg), b.max(s.obliquity_deg))
            })
    }

    /// The ice ages at `lat_deg` and `height_m` under the climate `cl` this
    /// model was built from, with lapse rate `lapse_k_km`.
    pub fn at(
        &self,
        cs: &ClimateSettings,
        cl: &Climate,
        lapse_k_km: f64,
        lat_deg: f64,
        height_m: f64,
    ) -> IceAges {
        let b0 = crate::water::water_at(cs, cl, lapse_k_km, lat_deg, height_m).ice_balance_m_yr;
        let b1 =
            crate::water::water_at(cs, &self.probe, lapse_k_km, lat_deg, height_m).ice_balance_m_yr;
        let j0 = cl.lerp(&self.energy_now, lat_deg);
        let j1 = cl.lerp(&self.energy_probe, lat_deg);
        let slope = if (j1 - j0).abs() > 1e-6 * j0.abs().max(1.0) {
            (b1 - b0) / (j1 - j0)
        } else {
            0.0
        };
        // A place the swing never takes across zero needs no series.
        let lo = b0 + slope * (cl.lerp(&self.energy_min, lat_deg) - j0);
        let hi = b0 + slope * (cl.lerp(&self.energy_max, lat_deg) - j0);
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        if lo > 0.0 || hi <= 0.0 {
            return IceAges {
                advances: 0,
                advances_per_myr: 0.0,
                glaciated_share: if lo > 0.0 { 1.0 } else { 0.0 },
                balance_min_m_yr: lo,
                balance_max_m_yr: hi,
            };
        }
        let series: Vec<f64> = self
            .energy
            .iter()
            .map(|e| b0 + slope * (cl.lerp(e, lat_deg) - j0))
            .collect();
        let n = series.len().max(1);
        let advances = series
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count() as u32;
        let span = self.spin.last().map(|s| s.t_yr).unwrap_or(0.0)
            - self.spin.first().map(|s| s.t_yr).unwrap_or(0.0);
        IceAges {
            advances,
            advances_per_myr: if span > 0.0 {
                advances as f64 * 1.0e6 / span
            } else {
                0.0
            },
            glaciated_share: series.iter().filter(|b| **b > 0.0).count() as f64 / n as f64,
            balance_min_m_yr: series.iter().cloned().fold(f64::INFINITY, f64::min),
            balance_max_m_yr: series.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }
}

impl IceAgeModel {
    /// The dated onsets of the ice advances at `lat_deg` and `height_m`:
    /// the sample times, years from the window's start as [`SpinSample`]
    /// counts them, at which the ice balance turns from losing to gaining.
    /// The same series [`IceAgeModel::at`] counts, so `len()` equals its
    /// `advances`.
    pub fn onsets_yr(
        &self,
        cs: &ClimateSettings,
        cl: &Climate,
        lapse_k_km: f64,
        lat_deg: f64,
        height_m: f64,
    ) -> Vec<f64> {
        let b0 = crate::water::water_at(cs, cl, lapse_k_km, lat_deg, height_m).ice_balance_m_yr;
        let b1 =
            crate::water::water_at(cs, &self.probe, lapse_k_km, lat_deg, height_m).ice_balance_m_yr;
        let j0 = cl.lerp(&self.energy_now, lat_deg);
        let j1 = cl.lerp(&self.energy_probe, lat_deg);
        if (j1 - j0).abs() <= 1e-6 * j0.abs().max(1.0) {
            return vec![];
        }
        let slope = (b1 - b0) / (j1 - j0);
        let series: Vec<f64> = self
            .energy
            .iter()
            .map(|e| b0 + slope * (cl.lerp(e, lat_deg) - j0))
            .collect();
        series
            .windows(2)
            .zip(self.spin.windows(2))
            .filter(|(w, _)| w[0] <= 0.0 && w[1] > 0.0)
            .map(|(_, s)| s[1].t_yr)
            .collect()
    }
}

/// The star's mass in kg from solar masses (for [`precession_parts`]).
pub fn star_kg(msun: f64) -> f64 {
    msun * M_SUN_KG
}
