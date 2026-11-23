//! The cheap regime sketch (surface design section 1, "Survey and disclosure").
//!
//! An unexamined or scanned world shows a globe that is statistically of its
//! kind (two-peaked heights for plates, one narrow peak for an episodic lid, a
//! hemispheric split and giant shields for a stagnant lid, bands for a giant)
//! without running its history. [`SketchModel`] is a pure function of the
//! seed, the world's address and its regime, evaluated at any direction on the
//! sphere: a sum of keyed smooth bumps in three octaves, turned into height,
//! albedo and liquid cover by one grammar per regime style. Its normalising
//! constants and thresholds are fixed once per world from a reference grid, so
//! a value at a point never depends on what else was asked for. The globe grid
//! ([`sketch`]) and the surface atlas tiles (S1) both read it. Surveyed worlds
//! replace it with the plate history (S2) and wearing down (S3).

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

/// One point of the sketch.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub height_m: f64,
    pub albedo: f64,
    pub liquid: bool,
}

const OCTAVES: [(u64, f64, f64); 3] = [(16, 0.25, 1.0), (64, 0.04, 0.5), (256, 0.006, 0.25)];
/// Reference grid that fixes the normalising constants and thresholds.
const REF_GRID: (u32, u32) = (32, 64);

fn unit_vec(d: &Draws, i: u64) -> [f64; 3] {
    let (x, y, z) = (d.normal(3 * i), d.normal(3 * i + 1), d.normal(3 * i + 2));
    let n = m::sqrt(x * x + y * y + z * z).max(1e-12);
    [x / n, y / n, z / n]
}

pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[derive(Clone, Debug)]
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

/// Cell centres of a latitude-longitude grid, rows from north: (direction, latitude, longitude, area weight).
fn grid(nlat: u32, nlon: u32) -> Vec<([f64; 3], f64, f64, f64)> {
    let mut out = Vec::with_capacity((nlat * nlon) as usize);
    for i in 0..nlat {
        let la = m::PI * (0.5 - (i as f64 + 0.5) / nlat as f64);
        for j in 0..nlon {
            let lo = m::TAU * ((j as f64 + 0.5) / nlon as f64 - 0.5);
            let (cl, sl) = (m::cos(la), m::sin(la));
            out.push(([cl * m::cos(lo), cl * m::sin(lo), sl], la, lo, cl));
        }
    }
    out
}

/// (mean, sd) pairs for the low, mid and fine octaves, then full and detail.
#[derive(Clone, Debug)]
struct Norms([(f64, f64); 5]);

/// The sketch of one world, as a function on the sphere.
#[derive(Clone, Debug)]
pub struct SketchModel {
    pub regime: String,
    pub style: String,
    def: RegimeDef,
    blobs: Vec<Vec<Blob>>,
    norms: Norms,
    relief: f64,
    locked: bool,
    molten_all: bool,
    bands: f64,
    axis: [f64; 3],
    shields: Vec<([f64; 3], f64)>,
    split: f64,
    /// Style threshold (plates: continent level; dunes: lake level).
    tau: f64,
    /// Lid styles: height spread for the albedo ramp.
    lid_sd: f64,
}

#[derive(Clone, Copy)]
struct Fields {
    low: f64,
    mid: f64,
    fine: f64,
    full: f64,
    detail: f64,
}

impl SketchModel {
    /// The sketch for a world in a regime (normally `env.regime_now`).
    pub fn new(s: &SurfaceSettings, env: &SurfaceEnvelope, regime: &str, seed: &Seed) -> Self {
        let def0 = &s.regimes[regime];
        // A dial draws as its default choice.
        let def = if def0.style == "dial" {
            let first = def0
                .dial
                .first()
                .cloned()
                .unwrap_or_else(|| "stagnant-lid".into());
            s.regimes[&first].clone()
        } else {
            def0.clone()
        };
        let d = Draws::new(
            *seed,
            env.address.clone(),
            Stream::new("surface.sketch.v1").expect("valid"),
        );
        let bs = blobs(&d);
        // Low gravity lets relief stand taller (surface design section 3 term 2).
        let g = env.gravity_m_s2.max(0.1);
        let gscale = (sse_core::sheet::G0 / g).clamp(0.5, 3.0);
        let bands = m::round((6.0 * m::sqrt(10.0 / s.tide.spin_hours.max(1.0))).clamp(1.0, 20.0));
        let mut model = SketchModel {
            regime: regime.to_string(),
            style: def.style.clone(),
            relief: if def.style == "lumpy" {
                0.12 * env.reference_radius_m
            } else {
                def.relief_m * gscale
            },
            def,
            blobs: bs,
            norms: Norms([(0.0, 1.0); 5]),
            locked: env.locked,
            molten_all: regime == "magma-ocean" || !env.locked,
            bands,
            axis: unit_vec(&d, 9_000),
            shields: (0..3)
                .map(|i| (unit_vec(&d, 9_001 + i), 1.2 + 0.8 * d.unit(3_000_001 + i)))
                .collect(),
            split: 0.6 + 0.4 * d.unit(3_000_000),
            tau: 0.0,
            lid_sd: 1.0,
        };
        // Fix the constants on the reference grid.
        let g = grid(REF_GRID.0, REF_GRID.1);
        let w: Vec<f64> = g.iter().map(|c| c.3).collect();
        let raw: Vec<[f64; 3]> = g
            .iter()
            .map(|c| {
                [
                    field(&model.blobs[0], c.0),
                    field(&model.blobs[1], c.0),
                    field(&model.blobs[2], c.0),
                ]
            })
            .collect();
        let mut norms = [(0.0, 1.0); 5];
        for (k, nk) in norms.iter_mut().enumerate().take(3) {
            let x: Vec<f64> = raw.iter().map(|r| r[k]).collect();
            let (mean, sd) = moments(&x, &w);
            *nk = (mean, if sd > 0.0 { sd } else { 1.0 });
        }
        let z = |r: &[f64; 3], k: usize| (r[k] - norms[k].0) / norms[k].1;
        let full: Vec<f64> = raw
            .iter()
            .map(|r| z(r, 0) + 0.5 * z(r, 1) + 0.25 * z(r, 2))
            .collect();
        let detail: Vec<f64> = raw.iter().map(|r| z(r, 1) + 0.5 * z(r, 2)).collect();
        for (k, x) in [(3, &full), (4, &detail)] {
            let (mean, sd) = moments(x, &w);
            norms[k] = (mean, if sd > 0.0 { sd } else { 1.0 });
        }
        model.norms = Norms(norms);
        let fs: Vec<Fields> = g.iter().map(|c| model.fields(c.0)).collect();
        match model.style.as_str() {
            "plates" => {
                let land = model.def.land_fraction.unwrap_or(0.3);
                let c: Vec<f64> = fs.iter().map(|f| f.low + 0.3 * f.mid).collect();
                model.tau = quantile(&c, &w, 1.0 - land);
            }
            "dunes" => {
                let full: Vec<f64> = fs.iter().map(|f| f.full).collect();
                model.tau = quantile(&full, &w, 0.25);
            }
            "lid" => {
                let h: Vec<f64> = g
                    .iter()
                    .zip(&fs)
                    .map(|(c, f)| model.lid_height(c.0, f))
                    .collect();
                model.lid_sd = moments(&h, &w).1.max(1.0);
            }
            _ => {}
        }
        model
    }

    fn fields(&self, v: [f64; 3]) -> Fields {
        let n = &self.norms.0;
        let low = (field(&self.blobs[0], v) - n[0].0) / n[0].1;
        let mid = (field(&self.blobs[1], v) - n[1].0) / n[1].1;
        let fine = (field(&self.blobs[2], v) - n[2].0) / n[2].1;
        let full = (low + 0.5 * mid + 0.25 * fine - n[3].0) / n[3].1;
        let detail = (mid + 0.5 * fine - n[4].0) / n[4].1;
        Fields {
            low,
            mid,
            fine,
            full,
            detail,
        }
    }

    fn lid_height(&self, v: [f64; 3], f: &Fields) -> f64 {
        let dich = m::tanh(3.0 * dot(v, self.axis));
        let mut x = self.relief * (0.45 * self.split * dich + 0.3 * f.full);
        for (c, amp) in &self.shields {
            let y = (1.0 - dot(v, *c)) / 0.004;
            if y < 20.0 {
                x += self.relief * amp * m::exp(-y);
            }
        }
        x
    }

    /// Height (smooth part), albedo and liquid at a unit direction.
    pub fn at(&self, v: [f64; 3]) -> Point {
        let f = self.fields(v);
        let [a_lo, a_hi] = self.def.albedo;
        let r = self.relief;
        let lat = m::asin(v[2].clamp(-1.0, 1.0));
        let (h, albedo, liquid) = match self.style.as_str() {
            "plates" => {
                let x = f.low + 0.3 * f.mid - self.tau;
                let t = ((x + 0.1) / 0.2).clamp(0.0, 1.0);
                let h_land = 0.18 * r + 0.15 * r * f.detail.max(0.0) + 0.03 * r * f.fine;
                let h_sea = -0.85 * r + 0.08 * r * f.detail;
                let h = lerp(h_sea, h_land, t);
                let liq = h < 0.0;
                (h, if liq { a_lo } else { a_hi }, liq)
            }
            "lid" => {
                let h = self.lid_height(v, &f);
                let t = (0.5 + h / (4.0 * self.lid_sd)).clamp(0.0, 1.0);
                (h, lerp(a_lo, a_hi, t), false)
            }
            "plains" => (
                r * (0.45 * f.full + 0.1 * f.fine),
                lerp(a_lo, a_hi, (0.5 + 0.2 * f.mid).clamp(0.0, 1.0)),
                false,
            ),
            "ice" => (
                r * (0.4 * f.full + 0.3 * f.fine),
                lerp(a_lo, a_hi, (0.5 + 0.25 * f.detail).clamp(0.0, 1.0)),
                false,
            ),
            "smooth" => {
                // Locked: the molten day side faces the host at 0° longitude (+x).
                let liq = self.molten_all || v[0] > 0.0;
                (r * 0.3 * f.full, if liq { a_lo } else { a_hi }, liq)
            }
            "ocean" => (-r + 0.3 * r * f.full, a_lo, true),
            "dunes" => {
                let liq = lat.abs() > m::PI / 3.0 && f.full < self.tau;
                let a = if liq {
                    0.05
                } else if lat.abs() < m::PI / 6.0 {
                    a_lo
                } else {
                    a_hi
                };
                (r * 0.45 * f.full, a, liq)
            }
            "lumpy" => (
                r * f.full,
                lerp(a_lo, a_hi, (0.5 + 0.15 * f.fine).clamp(0.0, 1.0)),
                false,
            ),
            "bands" => {
                let b = m::cos(2.0 * self.bands * lat);
                (
                    0.0,
                    (lerp(a_lo, a_hi, 0.5 + 0.45 * b) + 0.02 * f.mid).clamp(0.0, 1.0),
                    false,
                )
            }
            _ => (0.0, 0.5 * (a_lo + a_hi), false),
        };
        Point {
            height_m: h,
            albedo,
            liquid,
        }
    }

    /// Relief scale of this world's sketch, metres.
    pub fn relief_m(&self) -> f64 {
        self.relief
    }

    pub fn def(&self) -> &RegimeDef {
        &self.def
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    fn note(&self) -> String {
        match self.style.as_str() {
            "plates" => format!(
                "continents cover about {:.0}% (pack land fraction)",
                self.def.land_fraction.unwrap_or(0.3) * 100.0
            ),
            "lid" => "one global plate: a hemispheric split, a few giant shield volcanoes".into(),
            "plains" => "young plains with few craters: one narrow peak of heights".into(),
            "ice" => "an ice shell: low relief, bright, fractured".into(),
            "smooth" if self.molten_all => "molten everywhere".into(),
            "smooth" => {
                "locked: a molten day side centred on 0° longitude, crust on the night side".into()
            }
            "ocean" => "global ocean: heights are the sea floor".into(),
            "dunes" => "equatorial dune seas, lakes near the poles".into(),
            "lumpy" => "not round: heights are the shape's departure from the mean radius".into(),
            "bands" => format!(
                "{:.0} cloud bands each side of the equator, from the rotation",
                self.bands
            ),
            other => format!("no sketch grammar named {other}; drawn flat"),
        }
    }
}

/// The regime sketch globe at a survey state.
pub fn sketch(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    state: SurveyState,
) -> Sketch {
    let model = SketchModel::new(s, env, &env.regime_now, seed);
    let (nlat, nlon) = match state {
        SurveyState::Unexamined => (8u32, 16u32),
        SurveyState::Scanned => (32, 64),
        SurveyState::Surveyed | SurveyState::Landed => (64, 128),
    };
    let pts: Vec<Point> = grid(nlat, nlon).iter().map(|c| model.at(c.0)).collect();
    let mut note = model.note();
    let height_m = if state == SurveyState::Unexamined {
        note.push_str("; heights withheld until scanned");
        vec![]
    } else {
        pts.iter().map(|p| m::round(p.height_m) as i32).collect()
    };
    if state >= SurveyState::Surveyed {
        note.push_str("; the surveyed atlas replaces this sketch once S2 runs");
    }
    Sketch {
        regime: env.regime_now.clone(),
        style: model.style.clone(),
        state,
        nlat,
        nlon,
        height_m,
        albedo: pts
            .iter()
            .map(|p| m::round(p.albedo.clamp(0.0, 1.0) * 1000.0) as u16)
            .collect(),
        liquid: pts.iter().map(|p| p.liquid).collect(),
        note,
    }
}

/// Area weights of a sketch's cells, row-major.
pub fn weights(sk: &Sketch) -> Vec<f64> {
    grid(sk.nlat, sk.nlon).iter().map(|c| c.3).collect()
}

fn pow_n(x: f64, n: u32) -> f64 {
    let mut r = 1.0;
    for _ in 0..n {
        r *= x;
    }
    r
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
        .map(|(a, b)| pow_n((a - mean) / sd, 3) * b)
        .sum::<f64>()
        / sw;
    let m4 = x
        .iter()
        .zip(&w)
        .map(|(a, b)| pow_n((a - mean) / sd, 4) * b)
        .sum::<f64>()
        / sw;
    (mean, sd, (m3 * m3 + 1.0) / m4)
}

/// Amplitude of the degree-1 (hemispheric) part of the heights, metres.
pub fn hemispheric_split(sk: &Sketch) -> f64 {
    let g = grid(sk.nlat, sk.nlon);
    let mut dv = [0.0; 3];
    let mut sw = 0.0;
    for (c, h) in g.iter().zip(&sk.height_m) {
        for (d, x) in dv.iter_mut().zip(c.0) {
            *d += c.3 * (*h as f64) * x;
        }
        sw += c.3;
    }
    3.0 * m::sqrt(dot(dv, dv)) / sw
}
