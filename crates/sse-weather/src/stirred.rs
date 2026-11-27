//! Carrying a giant's cloud backwards along its own winds (track WS, package WS4; design
//! `handover/design/SSE4-stirred-clouds-design.md` section 3.6).
//!
//! For a place and a date, the cloud there was made some time ago somewhere else. Follow the wind backwards
//! (bands, the polar jet and its polygon, the storms and every eddy, all from their stream functions) to where it
//! was made, and read today's face there. Cloud is made in overlapping generations on a fixed lattice of dates a
//! renewal time τ apart (WS3), anchored at the campaign epoch and offset band by band by a keyed draw. Two
//! generations are alive at a place; one fades in from age 0 to its strongest at τ and out by 2τ, so their weights
//! sum to one and the mean age on show is τ. The picture at a date depends only on the date.
//!
//! - **Storms keep their shape.** The face is read for today, with the storms where they stand now: a place that
//!   came from within a storm's reach (three of its radii) is read where that storm stands now (moved by its drift
//!   over the age), a place outside by its own row's drift. (The design blended the two by the storm's Gaussian;
//!   over a renewal the two can be a band's length apart, so a blend reads neither.) Cloud a pole's polygon holds is
//!   not moved: the polygon stands still.
//! - **Threads thinner than a pixel.** Each generation carries the stretch of the map from here to where it is read
//!   (its larger singular value), so the renderer can read through the face's coarser levels.
//!
//! This is the reference answer for a patch (`want: "stirred"`); the renderer's GPU pass is held to it by test
//! (WS5). Eddies are round here: their steady ellipse is what the band's shear does to a round vortex, so giving
//! them their shape and the shear as well would count the shear twice.

use crate::conditions::Conditions;
use crate::contract::Date;
use crate::contract::WeatherSystem;
use crate::eddies::{bands, eddies_between, renewal_s, Eddy};
use crate::giants::{
    band_rho_km, band_wind_at, draws, giant_systems, polygon_hold, spin_sense, spin_sign,
    storm_r_at, wrap, zonal_vorticity, zonal_wind, GiantLayout, GiantSettings, Vortex, DEG,
};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const DAY_S: f64 = sse_core::sheet::DAY;
/// How far a round eddy's winds reach, in its radii (as a storm's).
const REACH_R: f64 = 3.0;
/// The time bins the eddies are filed in, days.
const BIN_D: f64 = 0.5;
/// The step of the bands' stream-function table, degrees of ρ.
const PSI_STEP_DEG: f64 = 0.01;
/// Eddies are filed by size, one tier per doubling, so a sum looks only as far as each tier reaches.
const TIERS: usize = 12;
/// How often the storm list is read over a cloud's age, days (the shortest storm family lives years).
const STORM_STEP_D: f64 = 30.0;
/// The offset, m, of the neighbours whose one-step paths give each step's Jacobian.
const NUDGE_M: f64 = 1.0;
/// A step is halved while it changes the local map by more than this (each entry of its Jacobian from the identity).
const STEP_STRAIN: f64 = 0.05;
/// At most this many halvings of a step.
const MAX_HALVINGS: u32 = 8;
/// The Jacobian is scaled down by 2^RESCALE whenever an entry passes 2^RESCALE.
const RESCALE: i32 = 64;

/// One generation of the cloud at a place.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Generation {
    /// How long ago this cloud was made, days.
    pub age_days: f64,
    /// Its share of the cloud on show (the two sum to one).
    pub weight: f64,
    /// Where to read today's face for it, degrees.
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// How far the map from here to that place stretches a line (its larger singular value), as a power of two:
    /// cloud drawn finer than a pixel is read through the face's coarser levels by this. Stirring is chaotic, so it
    /// can pass anything a float holds; its power of two cannot.
    pub stretch_log2: f64,
    /// The area the carrying keeps: the backward map's determinant (1 for the true flow).
    pub area: f64,
}

/// The cloud at each cell of a patch, carried (WS4, `want: "stirred"`). Cells run as `patch`'s: row by row from
/// the south-west, `n` across.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StirredPatch {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub half_km: f64,
    pub n: u32,
    pub day: f64,
    /// The renewal time τ, days (WS3); 0 where the deck has none, and then nothing is carried.
    pub renewal_days: f64,
    /// Each cell's two generations, the younger first.
    pub cells: Vec<[Generation; 2]>,
}

struct Storm {
    v: Vortex,
    sense: f64,
    born: f64,
    ends: f64,
    /// The day `v.lon_deg` is for.
    day_ref: f64,
    /// Its drift, degrees of longitude a day (0 for one that stays put).
    drift_deg_d: f64,
}

impl Storm {
    fn at(&self, day: f64) -> Option<Vortex> {
        (day >= self.born && day < self.ends).then(|| Vortex {
            lon_deg: wrap(self.v.lon_deg + self.drift_deg_d * (day - self.day_ref)),
            ..self.v
        })
    }
}

/// A place being carried back, and the Jacobian of the map from where it started (east and north there, to east and
/// north here), as 2^`log2` × `j`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Path {
    pub p: [f64; 3],
    pub j: [[f64; 2]; 2],
    pub log2: i32,
    /// The map's determinant as a power of two, summed step by step (the product's own determinant cancels away
    /// once the stretch is large).
    pub area_log2: f64,
}

impl Path {
    pub fn at(lat: f64, lon: f64) -> Self {
        Path {
            p: unit(lat, lon),
            j: [[1.0, 0.0], [0.0, 1.0]],
            log2: 0,
            area_log2: 0.0,
        }
    }
}

/// Everything that moves the cloud over a span of dates, filed for quick sums.
pub struct Carrier<'a> {
    gs: &'a GiantSettings,
    l: &'a GiantLayout,
    a: f64,
    turn: f64,
    storms: Vec<Storm>,
    eddies: Vec<Eddy>,
    day0: f64,
    nbins: usize,
    /// Per band, size tier and bin: the eddies alive in it, (longitude at the bin's middle, index), by longitude.
    lists: Vec<Vec<(f64, u32)>>,
    /// Per band, size tier and bin: how far an eddy in it moves from its filed longitude within the bin, degrees.
    slack: Vec<f64>,
    /// Per band: its latitudes.
    band_lat: Vec<[f64; 2]>,
    /// Per band and size tier: the reach of its widest eddy, km.
    group_reach_km: Vec<f64>,
    /// The eddies' stream function at a storm's centre, by (day, storm), the last few asked (the barrier reads it
    /// at every stage of every step near a storm).
    psi_cache: std::cell::RefCell<Vec<(f64, usize, f64)>>,
    /// The bands' Ψ(ρ), m²/s, north and south, every [`PSI_STEP_DEG`] of ρ from the pole; and the south's offset
    /// that joins the two at the equator.
    psi_band: [Vec<f64>; 2],
    psi_south: f64,
    /// The step, s: a quarter of the fastest shear time, an eighth of the fastest eddy's turn, a sixteenth of the
    /// fastest storm's, and a day at most.
    dt_s: f64,
    /// The step near a polygon, s: a quarter of the time the polar jet takes to cross two jet deformation radii.
    dt_poly_s: f64,
    /// Poleward of this |latitude| a pole's polygon is near, degrees (90 where there is none).
    poly_lat: f64,
}

fn unit(lat: f64, lon: f64) -> [f64; 3] {
    let (p, l) = (lat / DEG, lon / DEG);
    [m::cos(p) * m::cos(l), m::cos(p) * m::sin(l), m::sin(p)]
}

fn lat_lon(p: [f64; 3]) -> (f64, f64) {
    (
        m::asin(p[2].clamp(-1.0, 1.0)) * DEG,
        m::atan2(p[1], p[0]) * DEG,
    )
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(p: [f64; 3]) -> [f64; 3] {
    let r = m::sqrt(dot(p, p));
    [p[0] / r, p[1] / r, p[2] / r]
}

fn axpy(p: [f64; 3], h: f64, v: [f64; 3]) -> [f64; 3] {
    [p[0] + h * v[0], p[1] + h * v[1], p[2] + h * v[2]]
}

/// East and north at a place.
fn frame(lat: f64, lon: f64) -> ([f64; 3], [f64; 3]) {
    let (p, l) = (lat / DEG, lon / DEG);
    (
        [-m::sin(l), m::cos(l), 0.0],
        [-m::sin(p) * m::cos(l), -m::sin(p) * m::sin(l), m::cos(p)],
    )
}

/// Which band a latitude is in, as [`bands`] lists them.
fn band_of(l: &GiantLayout, lat: f64) -> usize {
    let j = l.jets.max(1) as usize;
    let k = ((m::sin(lat.abs() / DEG) * j as f64) as usize).min(j - 1);
    if lat >= 0.0 {
        k
    } else {
        j + k
    }
}

impl<'a> Carrier<'a> {
    /// What moves the cloud from `day0` to `day1`.
    pub fn new(
        gs: &'a GiantSettings,
        c: &Conditions,
        l: &'a GiantLayout,
        day0: f64,
        day1: f64,
    ) -> Self {
        let a = c.radius_m;
        let turn = spin_sign(l);
        // Storms: the list read every STORM_STEP_D days over the span, each storm once.
        let mut storms: Vec<Storm> = vec![];
        let mut seen: Vec<String> = vec![];
        let steps = (m::ceil((day1 - day0) / STORM_STEP_D) as i64).max(0);
        for k in 0..=steps {
            let day = (day0 + k as f64 * STORM_STEP_D).min(day1);
            for s in giant_systems(gs, c, Date { day }) {
                let Some(sense) = spin_sense(&s.kind) else {
                    continue;
                };
                if seen.contains(&s.id) {
                    continue;
                }
                seen.push(s.id.clone());
                storms.push(storm_of(gs, l, a, &s, sense, day));
            }
        }
        let eddies = eddies_between(gs, c, l, day0, day1);
        let bs = bands(l);
        let nb = bs.len();
        let nbins = (m::ceil((day1 - day0) / BIN_D) as usize).max(1);
        let ng = nb * TIERS;
        let mut lists: Vec<Vec<(f64, u32)>> = vec![vec![]; ng * nbins];
        let mut slack = vec![0.0f64; ng * nbins];
        let mut group_reach_km = vec![0.0f64; ng];
        let smallest = eddies
            .iter()
            .fold(f64::INFINITY, |x, e| x.min(e.size_km))
            .max(1e-3);
        for (i, e) in eddies.iter().enumerate() {
            let tier = (m::floor(m::log2(e.size_km / smallest)).max(0.0) as usize).min(TIERS - 1);
            let b = band_of(l, e.lat_deg) * TIERS + tier;
            group_reach_km[b] = group_reach_km[b].max(REACH_R * 0.5 * e.size_km);
            let rate = e.drift_m_s * DAY_S / (a * m::cos(e.lat_deg / DEG).max(1e-6)) * DEG;
            let lo = (m::floor((e.born - day0) / BIN_D).max(0.0)) as usize;
            let hi = (m::floor((e.ends - day0) / BIN_D).max(0.0) as usize).min(nbins - 1);
            for bin in lo..=hi {
                let mid = day0 + (bin as f64 + 0.5) * BIN_D;
                let k = b * nbins + bin;
                lists[k].push((e.lon_at(a, mid), i as u32));
                slack[k] = slack[k].max(rate.abs() * 0.5 * BIN_D);
            }
        }
        for v in lists.iter_mut() {
            v.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        }
        let band_lat = bs
            .iter()
            .map(|b| {
                [
                    b.lat_lo_deg.min(b.lat_hi_deg),
                    b.lat_lo_deg.max(b.lat_hi_deg),
                ]
            })
            .collect();
        // The step.
        let mut zeta: f64 = 0.0;
        let mut lat = -89.75;
        while lat < 90.0 {
            zeta = zeta.max(zonal_vorticity(gs, l, a, lat).abs());
            lat += 0.25;
        }
        let mut dt = DAY_S;
        if zeta > 0.0 {
            dt = dt.min(0.25 / zeta);
        }
        for e in &eddies {
            if e.strength > 0.0 {
                dt = dt.min(m::PI * e.size_km * 1000.0 / e.strength / 8.0);
            }
        }
        for s in &storms {
            if s.v.strength > 0.0 {
                dt = dt.min(m::PI * s.v.size_km * 1000.0 / s.v.strength / 16.0);
            }
        }
        let has_poly = l.poles.iter().any(|q| q.polygon_sides > 0);
        let ln_km = 2.0 * l.jet_deformation_km.max(1.0);
        let dt_poly = if has_poly && l.polar_jet_m_s.abs() > 0.0 {
            dt.min(0.25 * ln_km * 1000.0 / l.polar_jet_m_s.abs())
        } else {
            dt
        };
        let poly_lat = if has_poly {
            (l.polar_jet_lat_deg - 4.0 * ln_km * 1000.0 / a * DEG).max(0.0)
        } else {
            90.0
        };
        // The bands' Ψ(ρ) = ∫ u dρ along each pole's distance (banded_wind's Ψ' is the zonal wind at the latitude
        // ρ stands for), by the trapezoid rule.
        let psi_band = [1.0, -1.0].map(|h: f64| {
            let n = (100.0 / PSI_STEP_DEG) as usize;
            let dr = PSI_STEP_DEG / DEG * a;
            let u =
                |k: usize| zonal_wind(gs, l, a, h * (90.0 - (k as f64 * PSI_STEP_DEG).min(180.0)));
            let mut out = vec![0.0; n + 1];
            for k in 1..=n {
                out[k] = out[k - 1] + 0.5 * (u(k - 1) + u(k)) * dr;
            }
            out
        });
        let eq = (90.0 / PSI_STEP_DEG) as usize;
        let psi_south = psi_band[0][eq] + psi_band[1][eq];
        Carrier {
            gs,
            l,
            a,
            turn,
            storms,
            eddies,
            day0,
            nbins,
            lists,
            slack,
            band_lat,
            group_reach_km,
            psi_cache: std::cell::RefCell::new(vec![]),
            psi_band,
            psi_south,
            dt_s: dt,
            dt_poly_s: dt_poly,
            poly_lat,
        }
    }

    /// The wind at a place on a day, m/s, as a vector along the surface.
    pub fn wind(&self, p: [f64; 3], day: f64) -> [f64; 3] {
        let a = self.a;
        let (lat, lon) = lat_lon(p);
        let (e, n) = frame(lat, lon);
        let to3 = |w: [f64; 2]| [0, 1, 2].map(|i| w[0] * e[i] + w[1] * n[i]);
        // The storms' own winds.
        let mut own = [0.0f64; 2];
        for s in &self.storms {
            if let Some(v) = s.at(day) {
                if let Some(sw) = tapered_storm_wind(a, &v, self.turn, s.sense, lat, lon) {
                    own[0] += sw[0];
                    own[1] += sw[1];
                }
            }
        }
        let mut out = to3(own);
        // The background: the bands and the eddies.
        let (ve, psi_e) = self.eddy_sum(p, day);
        let vb = to3(band_wind_at(self.gs, self.l, a, lat, lon));
        // A storm turns as one body. Its own winds are the steady shape the band's shear holds it in (Moore and
        // Saffman, through `torn_by_shear`), so within it the shear and the eddies do not act again: there the
        // background is only the storm's drift, a turning of the sky at its rate. Written on the background's
        // stream function Ψ (bands and eddies), so the flow stays free of divergence: Ψ' = Ψ G + Ψ_ref (1 − G), G
        // the product of each near storm's shield (0 within its radius, rising smoothly to 1 at 1.5 radii), Ψ_ref
        // each storm's drift (Ψ at its centre, plus −ω a² sin φ about it) weighted by 1 − shield.
        let near: Vec<(Vortex, f64, f64)> = self
            .storms
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.at(day).map(|v| (i, s, v)))
            .filter(|(_, _, v)| storm_r_at(a, v, lat, lon) < 1.6)
            .map(|(i, s, v)| {
                let psi_c = self.band_psi(v.lat_deg, v.lon_deg) + self.psi_at_storm(i, &v, day);
                (v, psi_c, s.drift_deg_d / DEG / DAY_S)
            })
            .collect();
        if near.is_empty() {
            for i in 0..3 {
                out[i] += vb[i] + ve[i];
            }
            return out;
        }
        let psi = self.band_psi(lat, lon) + psi_e;
        // (G, Ψ_ref (1 − G)) at a place.
        let gh = |la: f64, lo: f64| {
            let (mut g, mut wsum, mut wpsi) = (1.0, 0.0, 0.0);
            let sl = m::sin(la / DEG);
            for (v, psi_c, om) in &near {
                let x = 1.0 - shield(storm_r_at(a, v, la, lo));
                g *= 1.0 - x;
                wsum += x;
                wpsi += x * (psi_c - om * a * a * (sl - m::sin(v.lat_deg / DEG)));
            }
            let reference = if wsum > 0.0 { wpsi / wsum } else { 0.0 };
            (g, reference * (1.0 - g))
        };
        let dd = 1e-4;
        let (g0, _) = gh(lat, lon);
        let (gn, hn) = gh(lat + dd, lon);
        let (gs_, hs) = gh(lat - dd, lon);
        let (ge, he) = gh(lat, lon + dd);
        let (gw, hw) = gh(lat, lon - dd);
        let step = 2.0 * dd / DEG;
        let d_phi = (psi * (gn - gs_) + (hn - hs)) / step;
        let d_lam = (psi * (ge - gw) + (he - hw)) / step;
        let cosl = m::cos(lat / DEG).max(1e-6);
        let ux = -d_phi / a;
        let vx = d_lam / (a * cosl);
        for i in 0..3 {
            out[i] += g0 * (vb[i] + ve[i]) + ux * e[i] + vx * n[i];
        }
        out
    }

    /// The bands' stream function at a place, m²/s (continuous across the equator).
    fn band_psi(&self, lat: f64, lon: f64) -> f64 {
        let (h, rho_km) = band_rho_km(self.l, self.a, lat, lon);
        let t = &self.psi_band[if h > 0.0 { 0 } else { 1 }];
        let x = (rho_km * 1000.0 / self.a * DEG / PSI_STEP_DEG).clamp(0.0, (t.len() - 1) as f64);
        let k = (m::floor(x) as usize).min(t.len() - 2);
        let f = x - k as f64;
        let psi = t[k] + (t[k + 1] - t[k]) * f;
        if h > 0.0 {
            psi
        } else {
            self.psi_south - psi
        }
    }

    /// The eddies' stream function at storm `i`'s centre on `day`.
    fn psi_at_storm(&self, i: usize, v: &Vortex, day: f64) -> f64 {
        if let Some(x) = self
            .psi_cache
            .borrow()
            .iter()
            .find(|x| x.0 == day && x.1 == i)
        {
            return x.2;
        }
        let psi = self.eddy_sum(unit(v.lat_deg, v.lon_deg), day).1;
        let mut c = self.psi_cache.borrow_mut();
        if c.len() >= 64 {
            c.remove(0);
        }
        c.push((day, i, psi));
        psi
    }

    /// The eddies' winds at a place on a day, m/s along the surface, and their stream function there, m²/s. Round
    /// eddies: Ψ = −q exp((1 − x²) / 2), x = d / R, q = sense h turn s R; with σ the angle to the centre c,
    /// u = Ψ'(d) (n·c) / sin σ and v = −Ψ'(d) (e·c) / sin σ (the storm's formula, by hand).
    fn eddy_sum(&self, p: [f64; 3], day: f64) -> ([f64; 3], f64) {
        let a = self.a;
        let (lat, lon) = lat_lon(p);
        let (e, n) = frame(lat, lon);
        let mut ve = [0.0f64; 3];
        let mut psi = 0.0f64;
        let bin = m::floor((day - self.day0) / BIN_D);
        if bin < 0.0 || bin as usize >= self.nbins {
            return (ve, psi);
        }
        let bin = bin as usize;
        let km = a / 1000.0;
        for (b, reach) in self.group_reach_km.iter().enumerate() {
            let reach = *reach;
            if reach <= 0.0 {
                continue;
            }
            let [lo, hi] = &self.band_lat[b / TIERS];
            let off = if lat < *lo {
                lo - lat
            } else if lat > *hi {
                lat - hi
            } else {
                0.0
            };
            if off / DEG * km > reach {
                continue;
            }
            let k = b * self.nbins + bin;
            let list = &self.lists[k];
            if list.is_empty() {
                continue;
            }
            // An eddy nearer the pole than here reaches further in longitude: size the window by the cosine at
            // the poleward edge of where an eddy in reach can be, or its winds would stop at the window's edge.
            let far = lo
                .abs()
                .max(hi.abs())
                .max(lat.abs() + reach / km * DEG)
                .min(90.0);
            let cos_far = m::cos(far / DEG);
            let half = if cos_far < 1e-6 {
                180.0
            } else {
                (m::asin((m::sin(reach / km).min(1.0) / cos_far).min(1.0)) * DEG + self.slack[k])
                    .min(180.0)
            };
            let mut visit = |x: &(f64, u32)| {
                let ed = &self.eddies[x.1 as usize];
                let s = ed.strength_at(day);
                if s <= 0.0 {
                    return;
                }
                let r = 0.5 * ed.size_km * 1000.0;
                let c = unit(ed.lat_deg, ed.lon_at(a, day));
                let sigma = m::acos(dot(p, c).clamp(-1.0, 1.0));
                let d = sigma * a;
                let x2 = (d / r) * (d / r);
                if x2 > REACH_R * REACH_R {
                    return;
                }
                let h = if ed.lat_deg >= 0.0 { 1.0 } else { -1.0 };
                let q = ed.sense * h * self.turn * s * r;
                let ex = m::exp(0.5 * (1.0 - x2));
                let x = d / r;
                let (w, dw) = taper(x);
                psi += -q * ex * w;
                let sn = m::sin(sigma);
                if sn < 1e-12 {
                    return;
                }
                // d(Ψ w)/dd = (q e / R)(x w − w').
                let dphi = q * ex / r * (x * w - dw);
                let u = dphi * dot(n, c) / sn;
                let v = -dphi * dot(e, c) / sn;
                for i in 0..3 {
                    ve[i] += u * e[i] + v * n[i];
                }
            };
            if half >= 180.0 {
                list.iter().for_each(&mut visit);
                continue;
            }
            // The longitude window, wrapped.
            let (w0, w1) = (lon - half, lon + half);
            let span = |from: f64, to: f64| {
                let i0 = list.partition_point(|x| x.0 < from);
                let i1 = list.partition_point(|x| x.0 <= to);
                i0..i1
            };
            for i in span(w0.max(-180.0), w1.min(180.0)) {
                visit(&list[i]);
            }
            if w0 < -180.0 {
                for i in span(w0 + 360.0, 180.0) {
                    visit(&list[i]);
                }
            }
            if w1 > 180.0 {
                for i in span(-180.0, w1 - 360.0) {
                    visit(&list[i]);
                }
            }
        }
        (ve, psi)
    }

    /// One Runge-Kutta step of `h` seconds (backwards when negative) from `p` on `day`.
    fn step(&self, p: [f64; 3], day: f64, h: f64) -> [f64; 3] {
        let a = self.a;
        let hd = h / DAY_S;
        let k1 = self.wind(p, day);
        let k2 = self.wind(norm(axpy(p, 0.5 * h / a, k1)), day + 0.5 * hd);
        let k3 = self.wind(norm(axpy(p, 0.5 * h / a, k2)), day + 0.5 * hd);
        let k4 = self.wind(norm(axpy(p, h / a, k3)), day + hd);
        let v = [0, 1, 2].map(|i| (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0);
        norm(axpy(p, h / a, v))
    }

    /// Follow the wind backwards from `p` on `day` for `days`.
    pub fn back(&self, p: [f64; 3], day: f64, days: f64) -> [f64; 3] {
        self.back_path(
            Path {
                p,
                j: [[1.0, 0.0], [0.0, 1.0]],
                log2: 0,
                area_log2: 0.0,
            },
            day,
            days,
        )
        .p
    }

    /// One step of `h` s back from `p` on day `t`, with its Jacobian (east and north at `p` to east and north at
    /// the end) from the one-step paths of two neighbours a metre off.
    fn step_with_jacobian(&self, p: [f64; 3], t: f64, h: f64) -> ([f64; 3], [f64; 2], [f64; 2]) {
        let a = self.a;
        let d = NUDGE_M / a;
        let (la, lo) = lat_lon(p);
        let (e, nn) = frame(la, lo);
        let p1 = self.step(p, t, -h);
        let pe = self.step(norm(axpy(p, d, e)), t, -h);
        let pn = self.step(norm(axpy(p, d, nn)), t, -h);
        (
            p1,
            tangent(p1, pe, a).map(|v| v / NUDGE_M),
            tangent(p1, pn, a).map(|v| v / NUDGE_M),
        )
    }

    /// Carry `path` back `h` s from day `t`: in one step if that step changes the local map by under
    /// [`STEP_STRAIN`], else as two halves (down to 2^-[`MAX_HALVINGS`] of the step), so the strongest strain is
    /// followed as finely as it needs.
    fn advance(&self, path: &mut Path, t: f64, h: f64, depth: u32) {
        let (p1, ce, cn) = self.step_with_jacobian(path.p, t, h);
        let strain = (ce[0] - 1.0)
            .abs()
            .max(ce[1].abs())
            .max(cn[0].abs())
            .max((cn[1] - 1.0).abs());
        if strain > STEP_STRAIN && depth < MAX_HALVINGS {
            self.advance(path, t, 0.5 * h, depth + 1);
            self.advance(path, t - 0.5 * h / DAY_S, 0.5 * h, depth + 1);
            return;
        }
        path.area_log2 += m::log2((ce[0] * cn[1] - cn[0] * ce[1]).abs());
        let j = path.j;
        path.j = [
            [
                ce[0] * j[0][0] + cn[0] * j[1][0],
                ce[0] * j[0][1] + cn[0] * j[1][1],
            ],
            [
                ce[1] * j[0][0] + cn[1] * j[1][0],
                ce[1] * j[0][1] + cn[1] * j[1][1],
            ],
        ];
        path.p = p1;
        let big = path.j.iter().flatten().fold(0.0f64, |x, v| x.max(v.abs()));
        if big > m::pow(2.0, RESCALE as f64) {
            let f = m::pow(2.0, -(RESCALE as f64));
            path.j = path.j.map(|r| r.map(|v| v * f));
            path.log2 += RESCALE;
        }
    }

    /// Follow the wind backwards from `path` on `day` for `days`, carrying the map's Jacobian step by step, so it
    /// holds however chaotic the stirring.
    pub fn back_path(&self, mut path: Path, day: f64, days: f64) -> Path {
        if days <= 0.0 {
            return path;
        }
        let (lat, _) = lat_lon(path.p);
        let dt = if lat.abs() >= self.poly_lat {
            self.dt_poly_s
        } else {
            self.dt_s
        };
        let n = (m::ceil(days * DAY_S / dt) as u64).max(1);
        let h = days * DAY_S / n as f64;
        for k in 0..n {
            self.advance(&mut path, day - k as f64 * h / DAY_S, h, 0);
        }
        path
    }

    /// Where today's face is read for cloud that was at `x` `age` days before `day`: moved on by the drift of what
    /// carries it, so the storms are where they stand today and a band's edge is where it is.
    pub fn read_place(&self, x: [f64; 3], day: f64, age: f64) -> [f64; 3] {
        let a = self.a;
        let (lat, lon) = lat_lon(x);
        let then = day - age;
        let row = zonal_wind(self.gs, self.l, a, lat) * age * DAY_S
            / (a * m::cos(lat / DEG).max(1e-6))
            * DEG;
        let band = (1.0 - polygon_hold(self.l, a, lat, lon)) * row;
        let mut g = 0.0;
        let mut moved = 0.0;
        for s in &self.storms {
            if let Some(v) = s.at(then) {
                let r = storm_r_at(a, &v, lat, lon);
                if r > REACH_R {
                    continue;
                }
                let gs = m::exp(-r * r);
                if gs > g {
                    g = gs;
                    moved = s.drift_deg_d * age;
                }
            }
        }
        // Within a storm's reach the cloud is read where the storm stands now. Blending its drift with the row's
        // would land the read between the two, which over a renewal can be a whole band's length apart; the bands
        // under a storm do not change along the row, so the storm's drift is the right one for all of it.
        let dl = if g > 0.0 { moved } else { band };
        unit(lat, wrap(lon + dl))
    }
}

fn storm_of(
    gs: &GiantSettings,
    l: &GiantLayout,
    a: f64,
    s: &WeatherSystem,
    sense: f64,
    day: f64,
) -> Storm {
    // Storms with a track drift with the zonal wind at their centre (`giants::track`); polar ones stay put.
    let drift = if s.track.is_empty() {
        0.0
    } else {
        zonal_wind(gs, l, a, s.lat_deg) * DAY_S / (a * m::cos(s.lat_deg / DEG).max(1e-6)) * DEG
    };
    Storm {
        v: Vortex::of(s),
        sense,
        born: s.born,
        ends: s.ends,
        day_ref: day,
        drift_deg_d: drift,
    }
}

/// A vortex's stream function is brought smoothly to nothing between 2.5 and 3 of its radii (the quintic
/// smoothstep), so its winds end with no seam: neighbours either side of a sharp cut would see different flows.
/// Returns the factor and its slope in radii.
fn taper(x: f64) -> (f64, f64) {
    let t = ((REACH_R - x) / 0.5).clamp(0.0, 1.0);
    let w = t * t * t * (10.0 + t * (6.0 * t - 15.0));
    let dwdt = if t > 0.0 && t < 1.0 {
        30.0 * t * t * (1.0 - t) * (1.0 - t)
    } else {
        0.0
    };
    (w, -dwdt / 0.5)
}

/// A storm's own winds as [`crate::giants::storm_wind`] has them, with the stream function tapered to nothing by
/// three radii ([`taper`]), m/s [east, north]. (`giant_cell` keeps the plain cut; only the carrying needs the
/// winds to end without a seam.)
fn tapered_storm_wind(
    a: f64,
    s: &Vortex,
    turn: f64,
    sense: f64,
    lat: f64,
    lon: f64,
) -> Option<[f64; 2]> {
    let rr = 0.5 * s.size_km.max(1.0);
    if storm_r_at(a, s, lat, lon) > REACH_R + 0.01 {
        return None;
    }
    let h = if s.lat_deg >= 0.0 { 1.0 } else { -1.0 };
    let q = sense * h * turn * s.strength * rr * 1000.0;
    let phi = |la: f64, lo: f64| {
        let x = storm_r_at(a, s, la, lo);
        -q * m::exp(0.5 * (1.0 - x * x)) * taper(x).0
    };
    let dd = 1e-5 * rr * 1000.0 / a * DEG;
    let cp = m::cos(lat / DEG);
    if cp < 1e-6 {
        return None;
    }
    let dphi = (phi(lat + dd, lon) - phi(lat - dd, lon)) / (2.0 * dd / DEG);
    let dlam = (phi(lat, lon + dd / cp) - phi(lat, lon - dd / cp)) / (2.0 * dd / cp / DEG);
    Some([-dphi / a, dlam / (a * cp)])
}

/// How much of the eddies' flow reaches a place `r` storm radii from a storm's centre: none within its radius,
/// all beyond 1.5, rising between as the quintic smoothstep (smooth to its second slope, as the steps need).
fn shield(r: f64) -> f64 {
    let t = ((r - 1.0) / 0.5).clamp(0.0, 1.0);
    t * t * t * (10.0 + t * (6.0 * t - 15.0))
}

/// The two generations alive at a place in band `band` on `day`, (age, weight), the younger first.
fn generations(c: &Conditions, band: usize, day: f64, tau: f64) -> [(f64, f64); 2] {
    let off = tau * draws(c, "weather.giant_generations.v1").unit(band as u64);
    let a1 = (day - off).rem_euclid(tau);
    [(a1, a1 / tau), (a1 + tau, 1.0 - a1 / tau)]
}

/// The tangent components, m, of `q − p` at `p`.
fn tangent(p: [f64; 3], q: [f64; 3], a: f64) -> [f64; 2] {
    let (lat, lon) = lat_lon(p);
    let (e, n) = frame(lat, lon);
    let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
    [dot(d, e) * a, dot(d, n) * a]
}

/// A 2×2 map's larger singular value and determinant, from its columns.
fn stretch_and_area(ce: [f64; 2], cn: [f64; 2]) -> (f64, f64) {
    let det = ce[0] * cn[1] - cn[0] * ce[1];
    let s = ce[0] * ce[0] + ce[1] * ce[1] + cn[0] * cn[0] + cn[1] * cn[1];
    let disc = (s * s - 4.0 * det * det).max(0.0);
    (m::sqrt(0.5 * (s + m::sqrt(disc))), det)
}

/// The stirred cloud over a patch round a place (WS4, design 3.6): each cell's two generations, where to read
/// today's face for each and how stretched.
#[allow(clippy::too_many_arguments)]
pub fn stirred_patch(
    gs: &GiantSettings,
    cs: &sse_climate::settings::ClimateSettings,
    c: &Conditions,
    l: &GiantLayout,
    day: f64,
    lat_deg: f64,
    lon_deg: f64,
    half_km: f64,
    n: u32,
) -> StirredPatch {
    let a = c.radius_m;
    let tau = renewal_s(gs, cs, c, l)
        .map(|s| s / DAY_S)
        .filter(|t| *t > 0.0 && t.is_finite());
    let places: Vec<(f64, f64)> = (0..n)
        .flat_map(|j| (0..n).map(move |i| (i, j)))
        .map(|(i, j)| {
            let at = |k: u32| half_km * (-1.0 + (2.0 * k as f64 + 1.0) / n as f64);
            lat_lon(crate::rules::patch_dir(a, lat_deg, lon_deg, at(i), at(j)))
        })
        .collect();
    let Some(tau) = tau else {
        let cells = places
            .into_iter()
            .map(|(la, lo)| {
                let g = |w: f64| Generation {
                    age_days: 0.0,
                    weight: w,
                    lat_deg: la,
                    lon_deg: lo,
                    stretch_log2: 0.0,
                    area: 1.0,
                };
                [g(1.0), g(0.0)]
            })
            .collect();
        return StirredPatch {
            lat_deg,
            lon_deg,
            half_km,
            n,
            day,
            renewal_days: 0.0,
            cells,
        };
    };
    let carrier = Carrier::new(gs, c, l, day - 2.0 * tau - 1.0, day);
    let cells = places
        .into_iter()
        .map(|(la, lo)| {
            let gens = generations(c, band_of(l, la), day, tau);
            // One path back to the younger generation, then on to the older.
            let p1 = carrier.back_path(Path::at(la, lo), day, gens[0].0);
            let p2 = carrier.back_path(p1, day - gens[0].0, gens[1].0 - gens[0].0);
            [(p1, gens[0]), (p2, gens[1])].map(|(path, (age, weight))| {
                let r = carrier.read_place(path.p, day, age);
                // The read's own Jacobian, from neighbours a metre off.
                let (bl, bo) = lat_lon(path.p);
                let (e, nn) = frame(bl, bo);
                let d = NUDGE_M / a;
                let re = carrier.read_place(norm(axpy(path.p, d, e)), day, age);
                let rn = carrier.read_place(norm(axpy(path.p, d, nn)), day, age);
                let de = tangent(r, re, a).map(|v| v / NUDGE_M);
                let dn = tangent(r, rn, a).map(|v| v / NUDGE_M);
                let j = path.j;
                let comp = [
                    [
                        de[0] * j[0][0] + dn[0] * j[1][0],
                        de[0] * j[0][1] + dn[0] * j[1][1],
                    ],
                    [
                        de[1] * j[0][0] + dn[1] * j[1][0],
                        de[1] * j[0][1] + dn[1] * j[1][1],
                    ],
                ];
                let (s_comp, _) =
                    stretch_and_area([comp[0][0], comp[1][0]], [comp[0][1], comp[1][1]]);
                let area = m::pow(2.0, path.area_log2);
                let (rl, rn2) = lat_lon(r);
                Generation {
                    age_days: age,
                    weight,
                    lat_deg: rl,
                    lon_deg: rn2,
                    stretch_log2: m::log2(s_comp) + path.log2 as f64,
                    area,
                }
            })
        })
        .collect();
    StirredPatch {
        lat_deg,
        lon_deg,
        half_km,
        n,
        day,
        renewal_days: tau,
        cells,
    }
}

#[cfg(test)]
#[path = "stirred_tests.rs"]
mod tests;
