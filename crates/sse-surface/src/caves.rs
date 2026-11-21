//! S14a part 2: lava-tube passages, resolved on approach (Site level).
//!
//! Part 1 ([`crate::tubes`]) gives each lava field an envelope: how much
//! passage it holds, how wide, how much roof is left. Nothing was placed.
//! A cave survey at a site places the passages near it, and only there
//! ("never do heavy work for a world that only gets a click").
//!
//! The same world at every level: passages are keyed by the field and by a
//! fixed grid cell in the field's own frame, never by the window asked for,
//! so a tube found from one site is the same tube found from the next.
//!
//! - **Where**: each grid cell of a field starts a Poisson number of tubes,
//!   its mean set so the field's passage per km² matches the envelope.
//! - **Path**: lava runs downhill. Each step tries a few headings within a
//!   turn limit and takes the lowest ground on the world's one height map.
//! - **Size**: a tube's width scatters about the field's width and never
//!   passes the widest roof that holds itself up (Blair et al. 2017). The
//!   passage is a little lower than it is wide (Sauro et al. 2020: Earth's
//!   tubes are mostly wider than high); the floor lies a roof plus a
//!   passage below the ground.
//! - **Roof**: each segment's roof is standing or fallen, as a two-state
//!   chain whose share of fallen segments is the envelope's lost roof.
//!   Falls come in runs: one fallen segment is a skylight, a few a pit
//!   chain, a long run an open rille. Every place a roof ends is an
//!   entrance.

use crate::atlas::Atlas;
use crate::tubes::TubeField;
use serde::{Deserialize, Serialize};
use sse_core::draw::Draws;
use sse_core::math as m;

pub const CAVES_VERSION: &str = "surface-caves-v1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CaveSettings {
    /// Grid cell, km, in each field's own frame (the key of every tube).
    pub cell_km: f64,
    /// Shortest and longest tube, km (log-uniform between; Sauro et al.
    /// 2020: most Earth tubes run a few km, Kazumura about 65 km).
    pub length_km: [f64; 2],
    /// Steps along a tube, at most, and the shortest and longest step, m.
    pub steps_max: u32,
    pub step_m: [f64; 2],
    /// Largest turn between steps, degrees.
    pub turn_max_deg: f64,
    /// Passage height over width (a fudge, registered).
    pub height_over_width: f64,
    /// Scatter of a tube's width about the field's, as a log-normal sigma.
    pub width_sigma: f64,
    /// Mean run of fallen segments, in segments.
    pub fallen_run: f64,
    /// Widest window a survey may ask for, km.
    pub window_max_km: f64,
}

impl Default for CaveSettings {
    fn default() -> Self {
        CaveSettings {
            cell_km: 10.0,
            length_km: [0.5, 15.0],
            steps_max: 40,
            step_m: [50.0, 400.0],
            turn_max_deg: 40.0,
            height_over_width: 0.6,
            width_sigma: 0.25,
            fallen_run: 2.0,
            window_max_km: 25.0,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CaveNode {
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// East and north of the survey's centre, m (the survey map's frame).
    pub x_m: f64,
    pub y_m: f64,
    pub ground_m: f64,
    pub floor_m: f64,
    /// An entrance here, if a roof ends here: skylight, pit chain or rille.
    pub entrance: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CaveSegment {
    pub from: usize,
    pub to: usize,
    pub length_m: f64,
    pub width_m: f64,
    pub height_m: f64,
    pub roofed: bool,
    /// What a fallen run reads as: skylight, pit chain or rille.
    pub open: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Passage {
    pub id: String,
    /// The tube field ([`TubeField::id`]) it belongs to.
    pub field: String,
    pub length_km: f64,
    pub width_m: f64,
    pub roof_m: f64,
    pub depth_m: f64,
    pub nodes: Vec<CaveNode>,
    pub segments: Vec<CaveSegment>,
    pub entrances: u32,
    pub speculative: bool,
    pub cause: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CaveSurvey {
    pub version: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub radius_km: f64,
    /// Atlas level the paths were read at, and its spacing.
    pub level: u8,
    pub spacing_m: f64,
    /// The tube fields that reach the window.
    pub fields: Vec<String>,
    pub passages: Vec<Passage>,
    pub why: String,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = m::sqrt(dot(a, a)).max(1e-300);
    [a[0] / l, a[1] / l, a[2] / l]
}

/// East and north at a point on the unit sphere.
fn frame(p: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let up = if p[2].abs() > 0.999_999 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let e = norm(cross(up, p));
    (e, cross(p, e))
}

/// Gnomonic east and north of `p` about `c`, km on a sphere of `r_km`.
fn plane(c: [f64; 3], e: [f64; 3], n: [f64; 3], p: [f64; 3], r_km: f64) -> (f64, f64) {
    let z = dot(p, c).max(1e-6);
    (r_km * dot(p, e) / z, r_km * dot(p, n) / z)
}

fn unplane(c: [f64; 3], e: [f64; 3], n: [f64; 3], x: f64, y: f64, r_km: f64) -> [f64; 3] {
    let (a, b) = (x / r_km, y / r_km);
    norm([
        c[0] + a * e[0] + b * n[0],
        c[1] + a * e[1] + b * n[1],
        c[2] + a * e[2] + b * n[2],
    ])
}

/// Step `ang` radians from `p` along azimuth `az` (from north, east positive).
fn step(p: [f64; 3], az: f64, ang: f64) -> [f64; 3] {
    let (e, n) = frame(p);
    let (s, c) = (m::sin(az), m::cos(az));
    let u = [
        c * n[0] + s * e[0],
        c * n[1] + s * e[1],
        c * n[2] + s * e[2],
    ];
    let (sa, ca) = (m::sin(ang), m::cos(ang));
    norm([
        p[0] * ca + u[0] * sa,
        p[1] * ca + u[1] * sa,
        p[2] * ca + u[2] * sa,
    ])
}

fn lat_lon(p: [f64; 3]) -> (f64, f64) {
    (
        m::asin(p[2].clamp(-1.0, 1.0)).to_degrees(),
        m::atan2(p[1], p[0]).to_degrees(),
    )
}

fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (la, lo) = (lat_deg.to_radians(), lon_deg.to_radians());
    [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)]
}

/// One counter from several parts (a fixed multiply-xor mix).
fn key(parts: &[u64]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &p in parts {
        h ^= p;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        h ^= h >> 29;
    }
    // Under 2^63, so `Draws::normal`'s 2i + 1 cannot overflow.
    h >> 1
}

fn poisson(d: &Draws, i: u64, lam: f64) -> u64 {
    if lam <= 0.0 {
        return 0;
    }
    let u = d.unit(i);
    let mut p = m::exp(-lam.min(50.0));
    let mut c = p;
    let mut k = 0u64;
    while u > c && k < 200 {
        k += 1;
        p *= lam / k as f64;
        c += p;
    }
    k
}

fn open_label(run: usize) -> &'static str {
    match run {
        1 => "skylight",
        2..=4 => "pit chain",
        _ => "rille",
    }
}

/// The passages near a site, within `radius_km` of it.
pub fn passages(
    s: &CaveSettings,
    atlas: &Atlas,
    fields: &[TubeField],
    d: &Draws,
    lat_deg: f64,
    lon_deg: f64,
    radius_km: f64,
) -> CaveSurvey {
    let r_km = atlas.radius_m / 1000.0;
    let radius_km = radius_km.clamp(0.1, s.window_max_km);
    let w = from_lat_lon(lat_deg, lon_deg);
    let (we, wn) = frame(w);
    let (lo, hi) = (s.length_km[0].max(1e-3), s.length_km[1].max(s.length_km[0]));
    let mean_len = if hi > lo * 1.0001 {
        (hi - lo) / m::ln(hi / lo)
    } else {
        lo
    };
    let reach = radius_km + hi;
    // The paths are read at the step's own spacing, so every step sees
    // ground of its own size.
    let step_floor = s.step_m[0].max(1.0);
    let level = atlas.level_for(step_floor.max(hi * 1000.0 / s.steps_max.max(1) as f64));
    let spacing_m = atlas.spacing_m(level);
    let mut out = CaveSurvey {
        version: CAVES_VERSION.into(),
        lat_deg,
        lon_deg,
        radius_km,
        level,
        spacing_m,
        fields: vec![],
        passages: vec![],
        why: String::new(),
    };
    for (fi, f) in fields.iter().enumerate() {
        let gap_km = r_km * m::acos(dot(w, f.dir).clamp(-1.0, 1.0));
        if gap_km > f.radius_km + reach || f.length_km <= 0.0 {
            continue;
        }
        out.fields.push(f.id.clone());
        let c = f.dir;
        let (e, n) = frame(c);
        let (wx, wy) = plane(c, e, n, w, r_km);
        let density = f.length_km / (m::PI * f.radius_km * f.radius_km).max(1e-9);
        let lam = density * s.cell_km * s.cell_km / mean_len;
        let cell = s.cell_km.max(1e-3);
        let (i0, i1) = (
            m::floor((wx - reach) / cell) as i64,
            m::floor((wx + reach) / cell) as i64,
        );
        let (j0, j1) = (
            m::floor((wy - reach) / cell) as i64,
            m::floor((wy + reach) / cell) as i64,
        );
        let fk = fi as u64 + 1;
        for i in i0..=i1 {
            for j in j0..=j1 {
                let ck = |slot: u64, x: u64| key(&[fk, i as u64, j as u64, slot, x]);
                let count = poisson(d, ck(0, 0), lam);
                for t in 1..=count {
                    let x = (i as f64 + d.unit(ck(t, 1))) * cell;
                    let y = (j as f64 + d.unit(ck(t, 2))) * cell;
                    if m::hypot(x, y) > f.radius_km {
                        continue;
                    }
                    let len = lo * m::pow(hi / lo, d.unit(ck(t, 3)));
                    if m::hypot(x - wx, y - wy) > radius_km + len {
                        continue;
                    }
                    let start = unplane(c, e, n, x, y, r_km);
                    let p = trace(s, atlas, f, d, &|x| ck(t, x), start, len, level);
                    let near = p
                        .iter()
                        .any(|q| r_km * m::acos(dot(w, q.0).clamp(-1.0, 1.0)) <= radius_km);
                    if near {
                        let id = format!("{}:{}:{}:{}", f.id, i, j, t);
                        out.passages.push(build(
                            s,
                            f,
                            d,
                            &|x| ck(t, x),
                            id,
                            &p,
                            w,
                            we,
                            wn,
                            atlas.radius_m,
                        ));
                    }
                }
            }
        }
    }
    out.why = if out.fields.is_empty() {
        "No lava field with open tubes reaches this site: either no low-viscosity lava was laid here, or later resurfacing buried it.".into()
    } else if out.passages.is_empty() {
        format!(
            "{} tube field(s) reach this site, but no passage runs within {:.1} km of it.",
            out.fields.len(),
            radius_km
        )
    } else {
        let ent: u32 = out.passages.iter().map(|p| p.entrances).sum();
        format!(
            "{} passage(s) from {} tube field(s) run within {:.1} km of the site, with {} entrance(s) where the roof has fallen.",
            out.passages.len(),
            out.fields.len(),
            radius_km,
            ent
        )
    };
    out
}

/// Points along a tube: (direction, ground height m).
#[allow(clippy::too_many_arguments)]
fn trace(
    s: &CaveSettings,
    atlas: &Atlas,
    f: &TubeField,
    d: &Draws,
    k: &dyn Fn(u64) -> u64,
    start: [f64; 3],
    len_km: f64,
    level: u8,
) -> Vec<([f64; 3], f64)> {
    let r = atlas.radius_m;
    let steps = s.steps_max.max(1);
    let step_m = (len_km * 1000.0 / steps as f64).clamp(s.step_m[0], s.step_m[1].max(s.step_m[0]));
    let n = m::ceil(len_km * 1000.0 / step_m).clamp(1.0, steps as f64) as u32;
    let ang = step_m / r;
    let mut hr = atlas.heights(level);
    let mut h = |p: [f64; 3]| hr.at(p);
    let turn = s.turn_max_deg.to_radians();
    // Start down the steepest of eight headings; level ground: a drawn one.
    let h0 = h(start);
    let mut az = d.unit(k(10)) * m::TAU;
    let mut best = h0;
    for q in 0..8 {
        let a = m::TAU * q as f64 / 8.0;
        let hq = h(step(start, a, ang));
        if hq < best {
            best = hq;
            az = a;
        }
    }
    let mut pts = vec![(start, h0)];
    let mut p = start;
    for st in 0..n {
        let mut pick = None;
        for (ci, frac) in [0.0, -0.5, 0.5, -1.0, 1.0].iter().enumerate() {
            let jitter = 0.15 * turn * (d.unit(k(100 + 8 * st as u64 + ci as u64)) - 0.5);
            let a = az + frac * turn + jitter;
            let q = step(p, a, ang);
            let hq = h(q);
            if pick.map(|(_, _, hb)| hq < hb).unwrap_or(true) {
                pick = Some((a, q, hq));
            }
        }
        let (a, q, hq) = pick.expect("five headings");
        az = a;
        p = q;
        pts.push((q, hq));
        // Lava does not run uphill past its own source's edge.
        if r / 1000.0 * m::acos(dot(q, f.dir).clamp(-1.0, 1.0)) > 1.2 * f.radius_km {
            break;
        }
    }
    pts
}

#[allow(clippy::too_many_arguments)]
fn build(
    s: &CaveSettings,
    f: &TubeField,
    d: &Draws,
    k: &dyn Fn(u64) -> u64,
    id: String,
    pts: &[([f64; 3], f64)],
    w: [f64; 3],
    we: [f64; 3],
    wn: [f64; 3],
    radius_m: f64,
) -> Passage {
    let width = (f.width_m * m::exp(s.width_sigma * d.normal(k(20)))).min(f.width_max_m);
    let roof = f.roof_m * width / f.width_m.max(1e-6);
    let depth = roof + s.height_over_width * width;
    // Two-state roof chain whose fallen share is the envelope's lost roof.
    let lost = (1.0 - f.roof_kept).clamp(0.0, 1.0);
    // Mean runs: fallen rf, standing rs, so that rf / (rf + rs) = lost.
    // Where most of the roof is gone, standing runs shrink to one segment
    // and the fallen runs lengthen instead.
    let (p01, p10) = if lost >= 0.999 {
        (1.0, 0.0)
    } else if lost <= 1e-9 {
        (0.0, 1.0)
    } else {
        let mut rf = s.fallen_run.max(1.0);
        let mut rs = rf * (1.0 - lost) / lost;
        if rs < 1.0 {
            rs = 1.0;
            rf = lost / (1.0 - lost);
        }
        (1.0 / rs, 1.0 / rf)
    };
    let nseg = pts.len().saturating_sub(1);
    let mut fallen = Vec::with_capacity(nseg);
    let mut state = d.unit(k(30)) < lost;
    for i in 0..nseg {
        if i > 0 {
            let u = d.unit(k(1000 + i as u64));
            state = if state { u >= p10 } else { u < p01 };
        }
        fallen.push(state);
    }
    // Label each fallen run by its length.
    let mut label: Vec<Option<String>> = vec![None; nseg];
    let mut i = 0;
    while i < nseg {
        if fallen[i] {
            let mut j = i;
            while j < nseg && fallen[j] {
                j += 1;
            }
            for l in label.iter_mut().take(j).skip(i) {
                *l = Some(open_label(j - i).into());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    let r_km = radius_m / 1000.0;
    let mut nodes: Vec<CaveNode> = pts
        .iter()
        .map(|(p, g)| {
            let (la, lo) = lat_lon(*p);
            let (x, y) = plane(w, we, wn, *p, r_km);
            CaveNode {
                lat_deg: la,
                lon_deg: lo,
                x_m: x * 1000.0,
                y_m: y * 1000.0,
                ground_m: *g,
                floor_m: g - depth,
                entrance: None,
            }
        })
        .collect();
    let mut segments = Vec::with_capacity(nseg);
    let mut length_m = 0.0;
    for i in 0..nseg {
        let a = pts[i].0;
        let b = pts[i + 1].0;
        let len = radius_m * m::acos(dot(a, b).clamp(-1.0, 1.0));
        length_m += len;
        let sw = (width * (0.8 + 0.4 * d.unit(k(2000 + i as u64)))).min(f.width_max_m);
        segments.push(CaveSegment {
            from: i,
            to: i + 1,
            length_m: len,
            width_m: sw,
            height_m: s.height_over_width * sw,
            roofed: !fallen[i],
            open: label[i].clone(),
        });
    }
    // An entrance wherever a roof ends: between a standing and a fallen
    // segment, or at a tube's end under a fallen one.
    let mut entrances = 0u32;
    for (ni, node) in nodes.iter_mut().enumerate() {
        let before = ni.checked_sub(1).map(|x| fallen[x]);
        let after = (ni < nseg).then(|| fallen[ni]);
        let here = match (before, after) {
            (Some(a), Some(b)) if a != b => label[if a { ni - 1 } else { ni }].clone(),
            (None, Some(true)) => label[ni].clone(),
            (Some(true), None) => label[ni - 1].clone(),
            _ => None,
        };
        if here.is_some() {
            entrances += 1;
        }
        node.entrance = here;
    }
    let drop = pts.first().map(|p| p.1).unwrap_or(0.0) - pts.last().map(|p| p.1).unwrap_or(0.0);
    let standing = fallen.iter().filter(|x| !**x).count();
    let cause = format!(
        "A lava tube in {field}: lava ran {len:.1} km downhill (dropping {drop:.0} m) under its own crust. It is about {width:.0} m wide and {ht:.0} m high, its floor {depth:.0} m below the ground under a {roof:.0} m roof. {kept}% of its roof stands ({lost:.0}% of the field's roof has fallen, in runs: one fallen segment is a skylight, a few a pit chain, a long run an open rille).",
        field = f.id,
        len = length_m / 1000.0,
        ht = s.height_over_width * width,
        kept = (100 * standing).checked_div(nseg).unwrap_or(100),
        lost = 100.0 * lost,
    );
    Passage {
        id,
        field: f.id.clone(),
        length_km: length_m / 1000.0,
        width_m: width,
        roof_m: roof,
        depth_m: depth,
        nodes,
        segments,
        entrances,
        speculative: f.speculative,
        cause: if f.speculative {
            format!("{cause} Cryolava tubes on an ice world are speculative.")
        } else {
            cause
        },
    }
}
