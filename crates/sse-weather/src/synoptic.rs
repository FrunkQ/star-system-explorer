//! W-synoptic: today's clouds out of the climate's average cover.
//!
//! The zonal cell carries each layer's average cover at its latitude (how much
//! of the sky is cloudy over a season). A photograph at one instant does not look
//! like that: the sky over any one spot is mostly clear or mostly overcast, and
//! the cloud gathers into weather systems, with clear gaps between them. This
//! file turns the average into one day's cloud the way statistical cloud schemes
//! do. A cell is cloudy where its water exceeds saturation (Sommeria and Deardorff
//! 1977). The water varies about its mean as a random field (Tompkins 2002), and
//! the threshold is set so the field's expected cover equals the average cover.
//! The climate's numbers stay as they are, and only their pattern changes.
//!
//! The field has the atmosphere's own scales. Its largest eddies are the
//! baroclinic waves' half wavelength, 3.9 N H / f (Eady 1949), the same length
//! the storm-track cyclones use. Smaller eddies follow the measured spectrum:
//! k⁻³ down to the break near 500 km, then k⁻⁵/³ (Nastrom and Gage 1985). The
//! field drifts with each layer's wind, and it renews itself over a cyclone's
//! lifetime. Each storm-track cyclone adds its fronts on top: a cold front
//! trailing toward the equator, a warm front ahead of it, the comma head wrapped
//! around the low, and the dry slot behind the cold front (Carlson 1980; Browning
//! 1997). A spin twice as fast halves f's length scale, so a fast-spinning
//! world's weather comes out finer-grained, as it should.

use crate::conditions::Conditions;
use crate::contract::*;
use crate::formations::{position_at, FormationSettings};
use crate::rules::Prepared;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;

const KM: f64 = 1000.0;

/// The pack's numbers for the day's cloud pattern (base-weather `weather_formations.synoptic`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Synoptic {
    /// Spread of the water within one cell, as a share of the field's spread:
    /// how soft a cloud's edge is (Tompkins 2002).
    pub edge_spread: f64,
    /// Spectral slope of the large eddies (k⁻³) and of the mesoscale (k⁻⁵/³), and
    /// the wavelength where one gives way to the other, km (Nastrom and Gage 1985).
    pub slope_synoptic: f64,
    pub slope_mesoscale: f64,
    pub break_km: f64,
    /// The finest eddy drawn, km: anything finer is smaller than a cell.
    pub finest_km: f64,
    /// How much each layer's field shares with the deep one (fronts and
    /// cyclones lift the whole column; low cloud also has its own small cells).
    pub shared: [f64; 3],
    /// Front structure, in units of the field's spread at the front's line.
    pub front_width_km: f64,
    pub cold_front: f64,
    pub warm_front: f64,
    pub comma_head: f64,
    pub dry_slot: f64,
    /// Share of a storm track's cloud that is mid-level frontal deck (altostratus
    /// and nimbostratus) rather than low cloud (ISCCP, Rossow and Schiffer 1999).
    pub frontal_share: f64,
    /// Cover of the trade cumulus that lives under the subtropical inversion over
    /// any sea, sinking air or not (Warren et al. 1988; Norris 1998).
    pub trade_cumulus: f64,
    /// Optical depth inside a cloud of each kind, the middle of its ISCCP class
    /// (Rossow and Schiffer 1999): once the cover is the day's, the cloud that is
    /// there is a whole cloud, not the season's thin average.
    pub in_cloud_tau: std::collections::BTreeMap<String, f64>,
    pub source: String,
}

/// One cell's cover redrawn as the day's pattern.
pub fn scatter(
    fs: &FormationSettings,
    s: &crate::WeatherSettings,
    c: &Conditions,
    p: &Prepared,
    date: Date,
    systems: &[WeatherSystem],
    cell: &mut Cell,
) {
    let sy = &fs.synoptic;
    let regime = p.circ.regime.as_str();
    if p.cond.is_none() || matches!(regime, "airless" | "banded-giant") {
        return;
    }
    let omega = c.spin_rate();
    if omega.is_nan() || omega <= 0.0 {
        return;
    }
    let (lat, lon) = (cell.lat_deg, cell.lon_deg);
    let substance = p
        .cond
        .as_ref()
        .map(|d| d.substance.clone())
        .unwrap_or_default();
    let edge = p.circ.hadley_edge_deg;
    // Storm tracks: part of the average cloud is the fronts' mid-level deck, the total kept.
    let mid = ((lat.abs() - edge) / 10.0).clamp(0.0, 1.0);
    if mid > 0.0
        && cell.layers.len() > 1
        && cell.layers[1].cloud_cover <= 1e-6
        && cell.layers[0].cloud_cover > 1e-6
    {
        let low = cell.layers[0].cloud_cover;
        let m_cover = sy.frontal_share * mid * low;
        cell.layers[0].cloud_cover = 1.0 - (1.0 - low) / (1.0 - m_cover);
        let tau = 4.0 + 2.0 * cell.layers[0].optical_depth;
        let l = &mut cell.layers[1];
        l.cloud_cover = m_cover;
        l.cloud_type = "altostratus".into();
        l.condensate = substance.clone();
        l.optical_depth = tau;
    }
    // Trade cumulus over a sea under the sinking branch.
    let trades = ((edge + 5.0 - lat.abs()) / 10.0).clamp(0.0, 1.0);
    if c.liquid.is_some() && c.ocean_fraction > 0.0 && trades > 0.0 {
        let floor = sy.trade_cumulus * c.ocean_fraction * trades;
        let l = &mut cell.layers[0];
        if l.cloud_cover < floor {
            l.cloud_cover = floor;
            if l.cloud_type == "none" {
                l.cloud_type = "cumulus".into();
            }
            l.condensate = substance.clone();
            l.optical_depth = l.optical_depth.max(3.0);
        }
    }
    let before = total_cover(cell);
    // The eddies' size: half an Eady wavelength at the storm track (the same as the cyclones').
    let track = p
        .circ
        .storm_tracks_deg
        .first()
        .copied()
        .unwrap_or(45.0)
        .abs()
        .max(15.0);
    let f = 2.0 * omega * m::sin(track * m::PI / 180.0);
    let h = s.climate.circulation.tropopause_scale_heights * c.scale_height_m();
    let lambda_m = fs.cyclones.eady_wavelengths * fs.cyclones.n_s * h / f;
    let largest = (0.5 * lambda_m).min(m::PI * c.radius_m);
    let life_s = fs.cyclones.life_days * sse_core::sheet::DAY;
    let seed = seed_of(c);
    let fronts = front_field(sy, c, date, systems, lat, lon);
    let shared = noise(
        seed,
        0,
        sy,
        c,
        largest,
        life_s,
        date,
        lat,
        lon,
        cell.layers[1].wind_m_s[0],
    );
    let n = cell.layers.len().min(3);
    #[allow(clippy::needless_range_loop)]
    for k in 0..n {
        let mean = cell.layers[k].cloud_cover;
        if mean <= 1e-6 || mean >= 1.0 - 1e-6 {
            continue;
        }
        let own = noise(
            seed,
            1 + k as u64,
            sy,
            c,
            largest,
            life_s,
            date,
            lat,
            lon,
            cell.layers[k].wind_m_s[0],
        );
        let a = sy.shared[k].clamp(0.0, 1.0);
        let eta = a * shared + m::sqrt(1.0 - a * a) * own + fronts[k];
        // Cloudy where the water passes saturation: the threshold that keeps the expected cover (Tompkins 2002).
        let spread = sy.edge_spread;
        let t = m::sqrt(1.0 + spread * spread) * inv_phi(1.0 - mean);
        let cover = phi((eta - t) / spread.max(1e-3));
        let l = &mut cell.layers[k];
        if let Some(t) = sy.in_cloud_tau.get(&l.cloud_type) {
            l.optical_depth = l.optical_depth.max(*t);
        }
        // Optical depth goes with the cloud's thickness: thicker where the field is far past saturation.
        if cover > 0.0 {
            l.optical_depth *= (0.5 + 0.5 * (eta - t).clamp(-1.0, 2.0)).max(0.25);
        }
        l.cloud_cover = cover;
    }
    // Rain falls where today's cloud is, the season's total kept.
    let after = total_cover(cell);
    if before > 1e-6 {
        cell.precip_mm_day *= (after / before).min(3.0);
    }
}

fn total_cover(cell: &Cell) -> f64 {
    let mut clear = 1.0;
    for l in &cell.layers {
        clear *= 1.0 - l.cloud_cover.clamp(0.0, 1.0);
    }
    1.0 - clear
}

fn seed_of(c: &Conditions) -> u64 {
    let d = Draws::new(
        Seed::from_text("sse4-weather-rules"),
        c.address.clone(),
        Stream::new("weather.synoptic.v1").expect("stream"),
    );
    let a = (d.unit(0) * 4294967296.0) as u64;
    let b = (d.unit(1) * 4294967296.0) as u64;
    (a << 32) ^ b
}

fn mix(mut z: u64) -> u64 {
    // SplitMix64's finaliser (Steele, Lea and Flood 2014): integer-only, the same on every machine.
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn lattice(seed: u64, field: u64, octave: u64, slice: i64, i: i64, j: i64, k: i64) -> f64 {
    let mut z = seed ^ field.wrapping_mul(0x9e3779b97f4a7c15);
    z = mix(z ^ octave.wrapping_mul(0xd1b54a32d192ed03));
    z = mix(z ^ (slice as u64).wrapping_mul(0x8cb92ba72f3d8dd7));
    z = mix(z ^ (i as u64).wrapping_mul(0xa0761d6478bd642f));
    z = mix(z ^ (j as u64).wrapping_mul(0xe7037ed1a0b428db));
    z = mix(z ^ (k as u64).wrapping_mul(0x8ebc6af09c88c6e3));
    ((z >> 11) as f64 / 9007199254740992.0) * 2.0 - 1.0
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Value noise at a point, lattice spacing 1, values uniform in [-1, 1).
fn value3(seed: u64, field: u64, octave: u64, slice: i64, x: [f64; 3]) -> f64 {
    let fl = [m::floor(x[0]), m::floor(x[1]), m::floor(x[2])];
    let (i0, j0, k0) = (fl[0] as i64, fl[1] as i64, fl[2] as i64);
    let (u, v, w) = (fade(x[0] - fl[0]), fade(x[1] - fl[1]), fade(x[2] - fl[2]));
    let g =
        |di: i64, dj: i64, dk: i64| lattice(seed, field, octave, slice, i0 + di, j0 + dj, k0 + dk);
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    let x00 = lerp(g(0, 0, 0), g(1, 0, 0), u);
    let x10 = lerp(g(0, 1, 0), g(1, 1, 0), u);
    let x01 = lerp(g(0, 0, 1), g(1, 0, 1), u);
    let x11 = lerp(g(0, 1, 1), g(1, 1, 1), u);
    lerp(lerp(x00, x10, v), lerp(x01, x11, v), w)
}

/// Spread of `value3` over space: measured over 20 000 points by the test below (W-synoptic package log).
const VALUE_SD: f64 = 0.4041;

/// The water's departure from its mean, in units of its spread: eddies from the
/// largest down to the finest, drifting with the wind and renewing over a life.
#[allow(clippy::too_many_arguments)]
fn noise(
    seed: u64,
    field: u64,
    sy: &Synoptic,
    c: &Conditions,
    largest_m: f64,
    life_s: f64,
    date: Date,
    lat: f64,
    lon: f64,
    u_m_s: f64,
) -> f64 {
    let t_s = date.day * sse_core::sheet::DAY;
    let slots = t_s / life_s.max(1.0);
    let k0 = m::floor(slots);
    let w = fade(slots - k0);
    let coslat = m::cos(lat * m::PI / 180.0).max(0.05);
    let mut total = 0.0;
    for (n, weight) in [(k0, 1.0 - w), (k0 + 1.0, w)] {
        // Each slice drifts with the wind from its own start, so neighbouring slices meet where they blend.
        let dt = t_s - n * life_s;
        let shift = u_m_s * dt / (c.radius_m * coslat) * 180.0 / m::PI;
        let d = crate::formations::dir(lat, lon - shift);
        total += weight * octaves(seed, field, n as i64, sy, c, largest_m, d);
    }
    // Blending two independent fields narrows the spread; put it back.
    total / m::sqrt((1.0 - w) * (1.0 - w) + w * w)
}

fn octaves(
    seed: u64,
    field: u64,
    slice: i64,
    sy: &Synoptic,
    c: &Conditions,
    largest_m: f64,
    d: [f64; 3],
) -> f64 {
    let mut spacing = largest_m;
    let finest = sy.finest_km * KM;
    let mut amp = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    let mut o = 0u64;
    while o < 12 && (o == 0 || spacing >= finest) {
        let r = c.radius_m / spacing;
        let v = value3(seed, field, o, slice, [d[0] * r, d[1] * r, d[2] * r]) / VALUE_SD;
        sum += amp * v;
        norm += amp * amp;
        // A spectrum E(k) ~ k^-p gives each halving of scale 2^((1-p)/2) of the amplitude.
        let p = if spacing * 2.0 > sy.break_km * KM {
            sy.slope_synoptic
        } else {
            sy.slope_mesoscale
        };
        amp *= m::pow(2.0, (1.0 - p) / 2.0);
        spacing *= 0.5;
        o += 1;
    }
    sum / m::sqrt(norm)
}

/// Each cyclone's fronts, per layer, in units of the field's spread.
fn front_field(
    sy: &Synoptic,
    c: &Conditions,
    date: Date,
    systems: &[WeatherSystem],
    lat: f64,
    lon: f64,
) -> [f64; 3] {
    let mut out = [0.0; 3];
    for sys in systems {
        if sys.kind != "cyclone" {
            continue;
        }
        let (sl, so) = position_at(sys, date.day);
        let r_km = sys.size_km.max(1.0);
        if crate::formations::distance_km(c, (lat, lon), (sl, so)) > 2.0 * r_km {
            continue;
        }
        // Local plane about the low (azimuthal equidistant, so it holds near the poles):
        // x east, y poleward, in units of the storm's size.
        let hemi = if sl >= 0.0 { 1.0 } else { -1.0 };
        let (la, lo) = (sl * m::PI / 180.0, so * m::PI / 180.0);
        let q = crate::formations::dir(lat, lon);
        let east = [-m::sin(lo), m::cos(lo), 0.0];
        let north = [
            -m::sin(la) * m::cos(lo),
            -m::sin(la) * m::sin(lo),
            m::cos(la),
        ];
        let centre = crate::formations::dir(sl, so);
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let (pe, pn) = (dot(q, east), dot(q, north));
        let ang = m::acos(dot(q, centre).clamp(-1.0, 1.0));
        let across = m::hypot(pe, pn).max(1e-12);
        let dist = ang * c.radius_m / KM / r_km;
        let x = dist * pe / across;
        let y = hemi * dist * pn / across;
        // A young wave has an open warm sector; an old one has wrapped up (occluded).
        let span = (sys.ends - sys.born).max(1e-6);
        let age = ((date.day - sys.born) / span).clamp(0.0, 1.0);
        let life = m::sqrt(m::sin(m::PI * age).max(0.0));
        let k = life * (sys.strength / 25.0).clamp(0.4, 1.6);
        let w = sy.front_width_km / r_km;
        // Cold front: from the low toward the equator, bending west at its tail (Bjerknes and Solberg 1922).
        let cold = |s: f64| [0.12 - 0.15 * s - 0.35 * s * s, -0.1 - 0.95 * s];
        // Warm front: east of the low, sloping a little toward the equator.
        let warm = |s: f64| [0.12 + 0.55 * s, -0.1 - 0.12 * s];
        let (sc, past_c) = to_curve(cold, x, y);
        let (sw, past_w) = to_curve(warm, x, y);
        let g = |d: f64, w: f64| m::exp(-0.5 * (d / w) * (d / w));
        let end_c = g(past_c, 2.0 * w);
        let end_w = g(past_w, 2.0 * w);
        // The warm conveyor belt rises just ahead of the cold front and up over the warm front.
        let cold_band = sy.cold_front * g(sc - w, 1.3 * w) * end_c;
        let warm_band = sy.warm_front * g(sw - 1.5 * w, 2.0 * w) * end_w;
        // Comma head: an arc wrapping round the low on its poleward and western side.
        let rr = m::sqrt(x * x + y * y);
        let bearing = m::atan2(y, x); // 0 east, pi/2 poleward
        let arc = g(rr - 0.28, 1.2 * w) * (0.5 + 0.5 * m::cos(bearing - 2.0));
        let comma = sy.comma_head * arc;
        // Dry slot: sinking air just behind the cold front, reaching in toward the low.
        let dry = sy.dry_slot * g(sc + 2.5 * w, 1.5 * w) * end_c * (1.0 - g(rr, 0.15));
        out[0] += k * (0.5 * cold_band - 0.6 * dry);
        out[1] += k * (cold_band + warm_band + comma - dry);
        out[2] += k * (0.8 * cold_band + 1.2 * warm_band + comma - 0.8 * dry);
    }
    out
}

/// Where (x, y) stands from a curve over s in [0, 1]: its signed distance across
/// the curve (positive to the left of the curve's direction of travel, which is
/// east for the cold front and poleward for the warm front), and how far past
/// either end it lies, so a front fades out beyond its ends with no seam.
fn to_curve(f: impl Fn(f64) -> [f64; 2], x: f64, y: f64) -> (f64, f64) {
    const N: usize = 16;
    let mut best = (f64::INFINITY, 0.0, 0.0);
    for i in 0..N {
        let (s0, s1) = (i as f64 / N as f64, (i + 1) as f64 / N as f64);
        let (a, b) = (f(s0), f(s1));
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = m::hypot(dx, dy).max(1e-12);
        let t = ((x - a[0]) * dx + (y - a[1]) * dy) / (len * len);
        let tc = t.clamp(0.0, 1.0);
        let d = m::hypot(x - a[0] - tc * dx, y - a[1] - tc * dy);
        if d < best.0 {
            let across = (dx * (y - a[1]) - dy * (x - a[0])) / len;
            let past = if i == 0 && t < 0.0 {
                -t * len
            } else if i == N - 1 && t > 1.0 {
                (t - 1.0) * len
            } else {
                0.0
            };
            best = (d, across, past);
        }
    }
    (best.1, best.2)
}

/// The standard normal's cumulative distribution.
pub fn phi(x: f64) -> f64 {
    0.5 * m::erfc(-x / core::f64::consts::SQRT_2)
}

/// Its inverse (Acklam's rational approximation, then one Newton step on `phi`).
pub fn inv_phi(p: f64) -> f64 {
    let p = p.clamp(1e-12, 1.0 - 1e-12);
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383_577_518_672_69e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    let lo = 0.02425;
    let x = if p < lo {
        let q = m::sqrt(-2.0 * m::log(p));
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= 1.0 - lo {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = m::sqrt(-2.0 * m::log(1.0 - p));
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    let e = phi(x) - p;
    let dens = m::exp(-0.5 * x * x) / m::sqrt(2.0 * m::PI);
    if dens > 0.0 {
        x - e / dens
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{block, fixture};

    /// Total cover over a latitude band on one day, every 2° of longitude.
    fn band(b: &crate::rules::RuleBlock, c: &Conditions, day: f64, lat: f64) -> Vec<f64> {
        let date = Date { day };
        let p = b.prepare(c, date);
        let sys = b.systems_at(c, &p, date, &[]);
        (0..180)
            .map(|i| total_cover(&b.cell_at(c, &p, date, &sys, None, lat, -179.0 + 2.0 * i as f64)))
            .collect()
    }

    #[test]
    fn a_days_cloud_has_clear_gaps_and_overcast_where_the_average_is_flat() {
        let b = block();
        let e = fixture("Earth");
        let mut v = vec![];
        for lat in [-50.0, -40.0, 40.0, 50.0] {
            v.extend(band(&b, &e, 200.5, lat));
        }
        let clear = v.iter().filter(|x| **x < 0.2).count() as f64 / v.len() as f64;
        let overcast = v.iter().filter(|x| **x > 0.9).count() as f64 / v.len() as f64;
        assert!(
            clear > 0.05 && overcast > 0.3 && clear + overcast > 0.6,
            "clear {clear} overcast {overcast}"
        );
    }

    #[test]
    fn the_days_pattern_keeps_the_seasons_cover() {
        // Over many days and places, the day's cover averages back to the zonal one.
        let b = block();
        let e = fixture("Earth");
        let mut drawn = 0.0;
        let mut zonal = 0.0;
        let mut n = 0.0;
        for d in 0..40 {
            let date = Date {
                day: 180.0 + 3.0 * d as f64,
            };
            let p = b.prepare(&e, date);
            for i in 0..24 {
                for lat in [-45.0, 45.0] {
                    let lon = -180.0 + 15.0 * i as f64;
                    let z = b.zonal_cell(&e, &p, lat, lon);
                    let mut cell = z.clone();
                    scatter(&b.formations, &b.settings, &e, &p, date, &[], &mut cell);
                    drawn += total_cover(&cell);
                    zonal += total_cover(&z);
                    n += 1.0;
                }
            }
        }
        let (drawn, zonal) = (drawn / n, zonal / n);
        assert!((drawn - zonal).abs() < 0.08, "drawn {drawn} zonal {zonal}");
    }

    #[test]
    fn a_faster_spin_makes_finer_weather() {
        // The eddies scale with N H / f: count the clear-to-cloudy changes round a latitude.
        let b = block();
        let edges = |c: &Conditions| {
            let mut n = 0;
            for lat in [-45.0, 45.0] {
                let v = band(&b, c, 200.5, lat);
                for i in 0..v.len() {
                    n += ((v[i] > 0.5) != (v[(i + 1) % v.len()] > 0.5)) as u32;
                }
            }
            n
        };
        let (slow, fast) = (
            edges(&fixture("Earth")),
            edges(&fixture("Fast-spinning Earth")),
        );
        assert!(fast > slow, "Earth {slow}, fast spinner {fast}");
    }

    #[test]
    fn inverse_normal_round_trips() {
        for p in [0.001, 0.02, 0.1, 0.3, 0.5, 0.75, 0.97, 0.999] {
            assert!((phi(inv_phi(p)) - p).abs() < 1e-9, "{p}");
        }
    }

    #[test]
    fn value_noise_spread_is_the_measured_one() {
        let mut s2 = 0.0;
        let n = 20000;
        for i in 0..n {
            let x = [
                i as f64 * 0.6180339,
                i as f64 * 0.4142135 + 3.0,
                i as f64 * 0.7320508 - 7.0,
            ];
            let v = value3(12345, 0, 0, 0, x);
            s2 += v * v;
        }
        let sd = m::sqrt(s2 / n as f64);
        assert!((sd / VALUE_SD - 1.0).abs() < 0.03, "{sd}");
    }
}
