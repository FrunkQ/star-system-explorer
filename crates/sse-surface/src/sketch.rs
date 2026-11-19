//! The cheap regime sketch (surface design section 1, "Survey and disclosure").
//!
//! An unexamined or scanned world shows a globe that is statistically of its
//! kind (two-peaked heights for plates, one narrow peak for an episodic lid, a
//! hemispheric split and giant shields for a stagnant lid, bands for a giant)
//! without running its history. It is a pure function of the seed, the
//! world's address and its envelope, on a latitude-longitude grid. The relief
//! is a sum of keyed smooth bumps on the sphere in three octaves; each regime
//! has a grammar that turns that field into height, albedo and liquid cover.
//! Surveyed worlds get the full atlas (S1) and plate history (S2) instead.

use crate::{RegimeDef, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurveyState {
    Unexamined,
    Scanned,
    Surveyed,
    Landed,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sketch {
    pub regime: String,
    /// The grammar that drew it (a dial regime draws as its default choice).
    pub style: String,
    pub state: SurveyState,
    pub nlat: u32,
    pub nlon: u32,
    /// Height against the reference radius, whole metres, rows from north to
    /// south, columns from 180° W eastward. Empty when the state withholds it.
    pub height_m: Vec<i32>,
    /// Albedo in thousandths.
    pub albedo: Vec<u16>,
    /// Liquid (water, magma, hydrocarbon) at the surface.
    pub liquid: Vec<bool>,
    pub note: String,
}

const OCTAVES: [(u64, f64, f64); 3] = [(16, 0.25, 1.0), (64, 0.04, 0.5), (256, 0.006, 0.25)];

fn unit_vec(d: &Draws, i: u64) -> [f64; 3] {
    let (x, y, z) = (d.normal(3 * i), d.normal(3 * i + 1), d.normal(3 * i + 2));
    let n = m::sqrt(x * x + y * y + z * z).max(1e-12);
    [x / n, y / n, z / n]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

struct Blob {
    c: [f64; 3],
    w: f64,
    amp: f64,
}

fn blobs(d: &Draws) -> Vec<Vec<Blob>> {
    let mut out = vec![];
    let mut base = 0u64;
    for &(n, w, amp) in &OCTAVES {
        let mut v = vec![];
        for k in 0..n {
            let i = base + k;
            let sign = if d.unit(1_000_000 + i) < 0.5 {
                -1.0
            } else {
                1.0
            };
            let size = 0.5 + d.unit(2_000_000 + i);
            v.push(Blob {
                c: unit_vec(d, i),
                w: w * size,
                amp: amp * sign,
            });
        }
        base += n;
        out.push(v);
    }
    out
}

fn field(bs: &[Blob], v: [f64; 3]) -> f64 {
    let mut f = 0.0;
    for b in bs {
        let x = (1.0 - dot(v, b.c)) / b.w;
        if x < 20.0 {
            f += b.amp * m::exp(-x);
        }
    }
    f
}

/// Area-weighted mean and standard deviation.
fn moments(x: &[f64], w: &[f64]) -> (f64, f64) {
    let sw: f64 = w.iter().sum();
    let mean = x.iter().zip(w).map(|(a, b)| a * b).sum::<f64>() / sw;
    let var = x
        .iter()
        .zip(w)
        .map(|(a, b)| (a - mean) * (a - mean) * b)
        .sum::<f64>()
        / sw;
    (mean, m::sqrt(var))
}

fn normalised(x: &[f64], w: &[f64]) -> Vec<f64> {
    let (mean, sd) = moments(x, w);
    let sd = if sd > 0.0 { sd } else { 1.0 };
    x.iter().map(|a| (a - mean) / sd).collect()
}

/// Area-weighted quantile.
fn quantile(x: &[f64], w: &[f64], q: f64) -> f64 {
    let mut idx: Vec<usize> = (0..x.len()).collect();
    idx.sort_by(|&a, &b| x[a].total_cmp(&x[b]).then(a.cmp(&b)));
    let total: f64 = w.iter().sum();
    let mut acc = 0.0;
    for i in idx {
        acc += w[i];
        if acc >= q * total {
            return x[i];
        }
    }
    x.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// The regime sketch globe at a survey state.
pub fn sketch(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    state: SurveyState,
) -> Sketch {
    let def0 = &s.regimes[&env.regime_now];
    // A dial draws as its default choice.
    let (def, style): (&RegimeDef, String) = if def0.style == "dial" {
        let first = def0
            .dial
            .first()
            .cloned()
            .unwrap_or_else(|| "stagnant-lid".into());
        let d = &s.regimes[&first];
        (d, d.style.clone())
    } else {
        (def0, def0.style.clone())
    };
    let (nlat, nlon) = match state {
        SurveyState::Unexamined => (8u32, 16u32),
        SurveyState::Scanned => (32, 64),
        SurveyState::Surveyed | SurveyState::Landed => (64, 128),
    };
    let d = Draws::new(
        *seed,
        env.address.clone(),
        Stream::new("surface.sketch.v1").expect("valid"),
    );
    let bs = blobs(&d);
    let n = (nlat * nlon) as usize;
    let mut vs = Vec::with_capacity(n);
    let mut lat = Vec::with_capacity(n);
    let mut lon = Vec::with_capacity(n);
    let mut w = Vec::with_capacity(n);
    for i in 0..nlat {
        let la = m::PI * (0.5 - (i as f64 + 0.5) / nlat as f64);
        for j in 0..nlon {
            let lo = m::TAU * ((j as f64 + 0.5) / nlon as f64 - 0.5);
            let (cl, sl) = (m::cos(la), m::sin(la));
            vs.push([cl * m::cos(lo), cl * m::sin(lo), sl]);
            lat.push(la);
            lon.push(lo);
            w.push(cl);
        }
    }
    let low: Vec<f64> = vs.iter().map(|v| field(&bs[0], *v)).collect();
    let mid: Vec<f64> = vs.iter().map(|v| field(&bs[1], *v)).collect();
    let fine: Vec<f64> = vs.iter().map(|v| field(&bs[2], *v)).collect();
    let low = normalised(&low, &w);
    let mid = normalised(&mid, &w);
    let fine = normalised(&fine, &w);
    let full: Vec<f64> = (0..n)
        .map(|k| low[k] + 0.5 * mid[k] + 0.25 * fine[k])
        .collect();
    let full = normalised(&full, &w);
    let detail: Vec<f64> = (0..n).map(|k| mid[k] + 0.5 * fine[k]).collect();
    let detail = normalised(&detail, &w);
    // Low gravity lets relief stand taller (surface design section 3 term 2).
    let g = env.gravity_m_s2.max(0.1);
    let gscale = (9.81 / g).clamp(0.5, 3.0);
    let relief = def.relief_m * gscale;
    let [a_lo, a_hi] = def.albedo;
    let mut h = vec![0.0; n];
    let mut alb = vec![0.0; n];
    let mut liq = vec![false; n];
    let mut note: String;
    match style.as_str() {
        "plates" => {
            let land = def.land_fraction.unwrap_or(0.3);
            let c: Vec<f64> = (0..n).map(|k| low[k] + 0.3 * mid[k]).collect();
            let tau = quantile(&c, &w, 1.0 - land);
            for k in 0..n {
                let x = c[k] - tau;
                let t = ((x + 0.1) / 0.2).clamp(0.0, 1.0);
                let h_land =
                    0.18 * relief + 0.15 * relief * detail[k].max(0.0) + 0.03 * relief * fine[k];
                let h_sea = -0.85 * relief + 0.08 * relief * detail[k];
                h[k] = lerp(h_sea, h_land, t);
                liq[k] = h[k] < 0.0;
                alb[k] = if liq[k] { a_lo } else { a_hi };
            }
            note = format!(
                "continents cover about {:.0}% (pack land fraction)",
                land * 100.0
            );
        }
        "lid" => {
            let axis = unit_vec(&d, 9_000);
            let shields: Vec<[f64; 3]> = (0..3).map(|i| unit_vec(&d, 9_001 + i)).collect();
            let split = 0.6 + 0.4 * d.unit(3_000_000);
            for k in 0..n {
                let dich = m::tanh(3.0 * dot(vs[k], axis));
                let mut x = relief * (0.45 * split * dich + 0.3 * full[k]);
                for (i, c) in shields.iter().enumerate() {
                    let y = (1.0 - dot(vs[k], *c)) / 0.004;
                    if y < 20.0 {
                        x += relief * (1.2 + 0.8 * d.unit(3_000_001 + i as u64)) * m::exp(-y);
                    }
                }
                h[k] = x;
            }
            let (_, sd) = moments(&h, &w);
            for k in 0..n {
                let t = (0.5 + h[k] / (4.0 * sd.max(1.0))).clamp(0.0, 1.0);
                alb[k] = lerp(a_lo, a_hi, t);
            }
            note = "one global plate: a hemispheric split, a few giant shield volcanoes".into();
        }
        "plains" => {
            for k in 0..n {
                h[k] = relief * (0.45 * full[k] + 0.1 * fine[k]);
                alb[k] = lerp(a_lo, a_hi, (0.5 + 0.2 * mid[k]).clamp(0.0, 1.0));
            }
            note = "young plains with few craters: one narrow peak of heights".into();
        }
        "ice" => {
            for k in 0..n {
                h[k] = relief * (0.4 * full[k] + 0.3 * fine[k]);
                alb[k] = lerp(a_lo, a_hi, (0.5 + 0.25 * detail[k]).clamp(0.0, 1.0));
            }
            note = "an ice shell: low relief, bright, fractured".into();
        }
        "smooth" => {
            let molten_all = env.regime_now == "magma-ocean" || !env.locked;
            for k in 0..n {
                h[k] = relief * 0.3 * full[k];
                liq[k] = molten_all || m::cos(lon[k]) > 0.0;
                alb[k] = if liq[k] { a_lo } else { a_hi };
            }
            note = if molten_all {
                "molten everywhere".into()
            } else {
                "locked: a molten day side centred on 0° longitude, crust on the night side".into()
            };
        }
        "ocean" => {
            for k in 0..n {
                h[k] = -relief + 0.3 * relief * full[k];
                liq[k] = true;
                alb[k] = a_lo;
            }
            note = "global ocean: heights are the sea floor".into();
        }
        "dunes" => {
            let lake = quantile(&full, &w, 0.25);
            for k in 0..n {
                h[k] = relief * 0.45 * full[k];
                let polar = lat[k].abs() > m::PI / 3.0;
                liq[k] = polar && full[k] < lake;
                let dune = lat[k].abs() < m::PI / 6.0;
                alb[k] = if liq[k] {
                    0.05
                } else if dune {
                    a_lo
                } else {
                    a_hi
                };
            }
            note = "equatorial dune seas, lakes near the poles".into();
        }
        "lumpy" => {
            let r = env.reference_radius_m;
            for k in 0..n {
                h[k] = 0.12 * r * full[k];
                alb[k] = lerp(a_lo, a_hi, (0.5 + 0.15 * fine[k]).clamp(0.0, 1.0));
            }
            note = "not round: heights are the shape's departure from the mean radius".into();
        }
        "bands" => {
            let bands = (6.0 * m::sqrt(10.0 / s.tide.spin_hours.max(1.0))).clamp(1.0, 20.0);
            let bands = m::round(bands);
            for k in 0..n {
                let b = m::cos(2.0 * bands * lat[k]);
                alb[k] = (lerp(a_lo, a_hi, 0.5 + 0.45 * b) + 0.02 * mid[k]).clamp(0.0, 1.0);
            }
            note = format!("{bands:.0} cloud bands each side of the equator, from the rotation");
        }
        other => {
            note = format!("no sketch grammar named {other}; drawn flat");
        }
    }
    let height_m = if state == SurveyState::Unexamined {
        vec![]
    } else {
        h.iter().map(|x| m::round(*x) as i32).collect()
    };
    if state == SurveyState::Unexamined {
        note.push_str("; heights withheld until scanned");
    }
    if state >= SurveyState::Surveyed {
        note.push_str("; the surveyed atlas replaces this sketch once S1 and S2 run");
    }
    Sketch {
        regime: env.regime_now.clone(),
        style,
        state,
        nlat,
        nlon,
        height_m,
        albedo: alb
            .iter()
            .map(|a| m::round(a.clamp(0.0, 1.0) * 1000.0) as u16)
            .collect(),
        liquid: liq,
        note,
    }
}

/// Area weights of a sketch's cells, row-major.
pub fn weights(sk: &Sketch) -> Vec<f64> {
    let mut w = vec![];
    for i in 0..sk.nlat {
        let la = m::PI * (0.5 - (i as f64 + 0.5) / sk.nlat as f64);
        for _ in 0..sk.nlon {
            w.push(m::cos(la));
        }
    }
    w
}

/// Hypsometry statistics: mean, standard deviation and Sarle's bimodality
/// coefficient (above 5/9 suggests two peaks).
pub fn hypsometry(sk: &Sketch) -> (f64, f64, f64) {
    let w = weights(sk);
    let x: Vec<f64> = sk.height_m.iter().map(|h| *h as f64).collect();
    let (mean, sd) = moments(&x, &w);
    let sw: f64 = w.iter().sum();
    let sd = sd.max(1e-9);
    let m3 = x
        .iter()
        .zip(&w)
        .map(|(a, b)| ((a - mean) / sd).powi_free(3) * b)
        .sum::<f64>()
        / sw;
    let m4 = x
        .iter()
        .zip(&w)
        .map(|(a, b)| ((a - mean) / sd).powi_free(4) * b)
        .sum::<f64>()
        / sw;
    (mean, sd, (m3 * m3 + 1.0) / m4)
}

/// Amplitude of the degree-1 (hemispheric) part of the heights, metres.
pub fn hemispheric_split(sk: &Sketch) -> f64 {
    let w = weights(sk);
    let mut dv = [0.0; 3];
    let mut sw = 0.0;
    let mut k = 0;
    for i in 0..sk.nlat {
        let la = m::PI * (0.5 - (i as f64 + 0.5) / sk.nlat as f64);
        for j in 0..sk.nlon {
            let lo = m::TAU * ((j as f64 + 0.5) / sk.nlon as f64 - 0.5);
            let v = [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)];
            let h = sk.height_m[k] as f64;
            for a in 0..3 {
                dv[a] += w[k] * h * v[a];
            }
            sw += w[k];
            k += 1;
        }
    }
    3.0 * m::sqrt(dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]) / sw
}

trait PowiFree {
    fn powi_free(self, n: u32) -> f64;
}

impl PowiFree for f64 {
    fn powi_free(self, n: u32) -> f64 {
        let mut r = 1.0;
        for _ in 0..n {
            r *= self;
        }
        r
    }
}
