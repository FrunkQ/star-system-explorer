//! Remains as places: streams, shells and mixed remains (track GH, package
//! GH4, design section 7.6).
//!
//! Each member of a galaxy's assembly tree that lost stars leaves a
//! **component** in the host: the stars it lost and when, its own spread of
//! ages and metals (its own rings, run once), and the path of its orbit. A
//! component is a density written as a formula, never a set of particles:
//!
//! - stars lost less than `mix_gyr` ago lie along the orbit. On an orbit
//!   that comes close and goes far (closest over farthest approach under
//!   `shell_below`) they pile up at the turning point as **shells**;
//!   otherwise they make a **stream**, a tube along the orbit ahead of and
//!   behind the satellite. Its length grows in proportion to the time since
//!   the stars left and its width and spread in speed scale with the cube
//!   root of the mass ratio (Johnston 1998, doi:10.1086/305273), so a small
//!   satellite leaves a thin stream and a big one a broad one;
//! - stars lost earlier are **mixed remains**: a smooth cloud between the
//!   orbit's nearest and farthest distances, falling off quickly beyond the
//!   farthest (Deason et al. 2018, doi:10.3847/2041-8213/aad0ee).
//!
//! The patch envelope adds every component that overlaps a patch
//! (`GalaxyEnvelope::remains_pc3`), and G5 draws a cell's halo stars from
//! them, each with the ages and metals of the galaxy it was born in.

use crate::feed::{Arrival, Member};
use crate::galaxy::GalaxyParams;
use crate::infall::OrbitParams;
use crate::rings::RingParams;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Seed};
use std::cell::RefCell;

/// One time bin of a component's births: the stars its galaxy made then and
/// the gas they were made from.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Birth {
    pub t_gyr: f64,
    /// Stars formed in the bin, solar masses.
    pub stars: f64,
    /// Mean [Fe/H] of the gas they formed from, and its spread over the rings.
    pub feh: f64,
    pub feh_sd: f64,
    pub alpha_fe: f64,
    /// [C+N/alpha] in dex (C/O = 0.55 × 10^this), [r/Fe] and [K-40/Fe].
    pub cn_alpha: f64,
    pub r_fe: f64,
    pub k40_fe: f64,
}

/// A torn satellite's stars in the host.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Component {
    /// The progenitor's address.
    pub from: String,
    pub fell_gyr: f64,
    pub torn_gyr: Option<f64>,
    pub merged_gyr: Option<f64>,
    /// Stars it has lost by now, and their mean metal fraction.
    pub stars_msun: f64,
    pub z: f64,
    /// Nearest and farthest distance of its orbit where it lost its stars
    /// (weighted by the stars lost), kpc; its period then, Gyr.
    pub peri_kpc: f64,
    pub apo_kpc: f64,
    pub period_gyr: f64,
    /// Cube root of its mass over the host's inside its orbit, where it lost
    /// its stars: the stream's width over its distance.
    pub spread: f64,
    /// +1 turning with the disc, −1 against it.
    pub prograde: f64,
    /// (t, stars lost by t), solar masses.
    #[serde(skip)]
    pub lost: Vec<(f64, f64)>,
    /// Its path, (Gyr, kpc).
    #[serde(skip)]
    pub path: Vec<(f64, [f64; 3])>,
    /// Its galaxy's births before its star formation stopped.
    #[serde(skip)]
    pub births: Vec<Birth>,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Linear in a sorted (t, value) table, flat beyond its ends.
fn at_time(table: &[(f64, f64)], t: f64) -> f64 {
    match table.iter().position(|p| p.0 > t) {
        None => table.last().map_or(0.0, |p| p.1),
        Some(0) => 0.0,
        Some(i) => {
            let (a, b) = (table[i - 1], table[i]);
            let w = if b.0 > a.0 {
                (t - a.0) / (b.0 - a.0)
            } else {
                1.0
            };
            a.1 + w * (b.1 - a.1)
        }
    }
}

impl Component {
    /// Stars lost by time `t`.
    pub fn lost_by(&self, t: f64) -> f64 {
        at_time(&self.lost, t)
    }

    /// Where it is on its path at `t`; beyond the path's end it goes round
    /// again by whole periods.
    pub fn pos(&self, t: f64) -> [f64; 3] {
        let p = &self.path;
        if p.is_empty() {
            return [0.0; 3];
        }
        let (t0, t1) = (p[0].0, p[p.len() - 1].0);
        let mut tt = t;
        let per = self.period_gyr.max(1.0e-3);
        if tt > t1 {
            let k = m::ceil((tt - t1) / per);
            tt -= k * per;
        }
        if tt < t0 {
            tt = t0;
        }
        let i = p.partition_point(|q| q.0 <= tt).clamp(1, p.len() - 1);
        let (a, b) = (p[i - 1], p[i]);
        let w = if b.0 > a.0 {
            ((tt - a.0) / (b.0 - a.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        [
            a.1[0] + w * (b.1[0] - a.1[0]),
            a.1[1] + w * (b.1[1] - a.1[1]),
            a.1[2] + w * (b.1[2] - a.1[2]),
        ]
    }

    /// Mass lost between `a` and `b`, and the mean time it left.
    fn lost_between(&self, a: f64, b: f64) -> (f64, f64) {
        let (mut mass, mut tm) = (0.0, 0.0);
        let mut prev = (a, self.lost_by(a));
        for &(t, v) in self.lost.iter().filter(|p| p.0 > a && p.0 < b) {
            let dm = v - prev.1;
            mass += dm;
            tm += dm * 0.5 * (t + prev.0);
            prev = (t, v);
        }
        let dm = self.lost_by(b) - prev.1;
        mass += dm;
        tm += dm * 0.5 * (b + prev.0);
        (mass, if mass > 0.0 { tm / mass } else { b })
    }
}

/// Pack settings for the shapes (`provides.other.cosmology.sky.satellites.remains`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RemainsParams {
    /// The mixed cloud's fall-off beyond its farthest distance, as a share
    /// of that distance.
    pub edge_share: f64,
    /// A stream's half length grows as `spread × time since the stars left`
    /// along its orbit (in time), at most half a period.
    pub length_k: f64,
    /// A shell's opening: e-folding of its cap in 1 − cos(angle).
    pub shell_cap: f64,
    /// Rings a member's own galaxy runs with for its births.
    pub member_rings: u32,
    /// Time bins of a component's births.
    pub birth_bins: u32,
    /// Smallest stream width, kpc.
    pub min_width_kpc: f64,
    /// Members whose lost stars are fewer than this are left out, M☉.
    pub min_stars_msun: f64,
}

impl Default for RemainsParams {
    fn default() -> Self {
        RemainsParams {
            edge_share: 0.2,
            length_k: 1.0,
            shell_cap: 0.5,
            member_rings: 4,
            birth_bins: 40,
            min_width_kpc: 0.05,
            min_stars_msun: 1.0e3,
        }
    }
}

/// The components of a fed galaxy, from its members and what reached it.
#[allow(clippy::too_many_arguments)]
pub fn components(
    members: &[Member],
    arrivals: &[Arrival],
    gp: &GalaxyParams,
    rings: &RingParams,
    rp: &RemainsParams,
    seed: Seed,
) -> Vec<Component> {
    let mut out = vec![];
    for mb in members {
        let from = &mb.progenitor.address;
        let mut lost = vec![(mb.progenitor.t_infall_gyr, 0.0)];
        let (mut total, mut zm) = (0.0, 0.0);
        for a in arrivals {
            if let Arrival::Halo {
                t_gyr,
                from: f,
                stars,
                z,
            } = a
            {
                if f == from {
                    total += stars;
                    zm += stars * z;
                    lost.push((*t_gyr, total));
                }
            }
        }
        if total < rp.min_stars_msun {
            continue;
        }
        let f = &mb.fate;
        // The orbit's period: the spacing of its closest approaches.
        let peris: Vec<f64> = f
            .passes
            .iter()
            .filter(|p| !p.disc)
            .map(|p| p.t_gyr)
            .collect();
        let period = if peris.len() >= 2 {
            let n = peris.len();
            let k = n.min(4);
            (peris[n - 1] - peris[n - k]) / (k - 1) as f64
        } else {
            let r = f.r_kpc.max(1.0);
            let v = m::sqrt(dot(f.vel_kpc_gyr, f.vel_kpc_gyr)).max(1.0e-3);
            2.0 * m::PI * r / v
        }
        .max(0.05);
        // Nearest and farthest distance within a period of each loss,
        // weighted by the stars lost then; the spread at the nearest pass.
        let (mut wp, mut wa, mut ws, mut w) = (0.0, 0.0, 0.0, 0.0);
        let mut prev = 0.0;
        for &(t, v) in &lost[1..] {
            let dm = v - prev;
            prev = v;
            let near: Vec<f64> = f
                .path
                .iter()
                .filter(|p| (p.0 - t).abs() <= 0.5 * period)
                .map(|p| m::sqrt(dot(p.1, p.1)))
                .collect();
            if near.is_empty() {
                continue;
            }
            let lo = near.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = near.iter().copied().fold(0.0, f64::max);
            let pass = f
                .passes
                .iter()
                .rfind(|p| !p.disc && p.t_gyr <= t + 0.5 * period)
                .or(f.passes.iter().find(|p| !p.disc));
            // Johnston's scaling holds for small ratios; a satellite as
            // heavy as the host inside its orbit spreads over the orbit.
            let s = pass
                .map_or(0.1, |p| {
                    m::cbrt((p.bound_msun / p.host_inside_msun.max(1.0)).max(1.0e-12))
                })
                .min(1.0);
            wp += dm * lo;
            wa += dm * hi;
            ws += dm * s;
            w += dm;
        }
        if w <= 0.0 {
            continue;
        }
        let stop = f.quench_gyr.unwrap_or(f.t_gyr).min(f.t_gyr);
        let births = births_of(mb, gp, rings, rp, seed, stop);
        out.push(Component {
            from: from.clone(),
            fell_gyr: mb.progenitor.t_infall_gyr,
            torn_gyr: f.disrupted_gyr,
            merged_gyr: f.merged_gyr,
            stars_msun: total,
            z: zm / total,
            peri_kpc: wp / w,
            apo_kpc: (wa / w).max(wp / w),
            period_gyr: period,
            spread: ws / w,
            prograde: f.passes.last().map_or(0.0, |p| p.prograde),
            lost,
            path: f.path.clone(),
            births,
        });
    }
    out
}

/// A member's births from its own rings (coarse: `member_rings` rings).
fn births_of(
    mb: &Member,
    gp: &GalaxyParams,
    rings: &RingParams,
    rp: &RemainsParams,
    seed: Seed,
    stop: f64,
) -> Vec<Birth> {
    let addr: Address = match mb.progenitor.address.parse() {
        Ok(a) => a,
        Err(_) => return vec![],
    };
    let p = RingParams {
        rings: rp.member_rings.max(1),
        ..rings.clone()
    };
    let d = crate::rings::evolve(&mb.galaxy, gp, &p, seed, &addr);
    let steps = d.states.first().map_or(0, |s| s.len());
    let t_first = d.t_start_gyr;
    let span = (stop - t_first).max(1.0e-3);
    let nb = rp.birth_bins.max(1) as usize;
    let mut bins = vec![Birth::default(); nb];
    let mut sq = vec![0.0; nb];
    for k in 0..steps {
        let t = d.time(k) + 0.5 * d.step_gyr;
        if t > stop {
            break;
        }
        let b = (((t - t_first) / span * nb as f64) as usize).min(nb - 1);
        for (ring, st) in d.rings.iter().zip(&d.states) {
            let made = ring.formed.get(k).copied().unwrap_or(0.0);
            if made <= 0.0 {
                continue;
            }
            let s = &st[k];
            let x = &mut bins[b];
            x.stars += made;
            x.t_gyr += made * t;
            x.feh += made * s.feh;
            x.alpha_fe += made * (s.alpha_h - s.feh);
            x.cn_alpha += made * (s.cn_h - s.alpha_h);
            x.r_fe += made * (s.r_h - s.feh);
            x.k40_fe += made * (s.k40_h - s.feh);
            sq[b] += made * s.feh * s.feh;
        }
    }
    bins.into_iter()
        .zip(sq)
        .filter(|(b, _)| b.stars > 0.0)
        .map(|(mut b, q)| {
            let w = b.stars;
            b.t_gyr /= w;
            b.feh /= w;
            b.alpha_fe /= w;
            b.cn_alpha /= w;
            b.r_fe /= w;
            b.k40_fe /= w;
            b.feh_sd = m::sqrt((q / w - b.feh * b.feh).max(0.0));
            b
        })
        .collect()
}

/// One shape of a component at a time.
#[derive(Clone, Debug)]
enum Piece {
    /// A smooth cloud between `a` and `b` kpc, falling off as e^(−(r − b)/h).
    Mixed {
        mass: f64,
        a: f64,
        b: f64,
        h: f64,
        norm: f64,
    },
    /// A cap at radius `b` in direction `dir`, radial width `w`.
    Shell {
        mass: f64,
        b: f64,
        w: f64,
        dir: [f64; 3],
        cap: f64,
        cap_norm: f64,
    },
    /// Equal blobs of width `sigma` along the orbit.
    Stream {
        mass: f64,
        sigma: f64,
        blobs: Vec<[f64; 3]>,
        lo: [f64; 3],
        hi: [f64; 3],
    },
}

/// A component's shapes at one time, and which component they belong to.
#[derive(Clone, Debug)]
struct Shapes {
    t_gyr: f64,
    pieces: Vec<(usize, Piece)>,
}

/// The shapes worked out for the last few times (a draw reads one time
/// thousands of times).
#[derive(Debug, Default)]
pub struct Cache {
    at: RefCell<Vec<Shapes>>,
    grids: RefCell<Vec<(u64, u64, std::rc::Rc<ClassGrid>)>>,
}

/// Each IMF point's luminosity class at each age bin (the count tree's).
pub type ClassGrid = Vec<(f64, f64, Vec<usize>)>;

impl Clone for Cache {
    fn clone(&self) -> Self {
        Cache::default()
    }
}

fn shapes(comps: &[Component], op: &OrbitParams, rp: &RemainsParams, t: f64) -> Shapes {
    let mut pieces = vec![];
    for (i, c) in comps.iter().enumerate() {
        if t <= c.fell_gyr || c.lost.is_empty() {
            continue;
        }
        // A stream wraps its whole orbit once its half length reaches half
        // a period: a broad one (from a big satellite) mixes within a few
        // orbits, a thin one takes up to `mix_gyr` (GH2's label rule).
        let mix = (0.5 * c.period_gyr / (rp.length_k * c.spread).max(1.0e-6)).min(op.mix_gyr);
        let old = c.lost_by(t - mix);
        if old > 0.0 {
            let (a, b) = (c.peri_kpc.max(0.05), c.apo_kpc.max(c.peri_kpc.max(0.05)));
            let h = (rp.edge_share * b).max(1.0e-3);
            let norm = 4.0
                * m::PI
                * (a / 3.0 + (b - a) + (b * b * h + 2.0 * b * h * h + 2.0 * h * h * h) / (b * b));
            pieces.push((
                i,
                Piece::Mixed {
                    mass: old,
                    a,
                    b,
                    h,
                    norm,
                },
            ));
        }
        let (young, t_left) = c.lost_between(t - mix, t);
        if young <= 0.0 {
            continue;
        }
        let ratio = if c.apo_kpc > 0.0 {
            c.peri_kpc / c.apo_kpc
        } else {
            1.0
        };
        if ratio < op.shell_below {
            // The turning point: the farthest point of its last period.
            let (mut dir, mut far) = ([1.0, 0.0, 0.0], 0.0);
            let n = 32;
            for k in 0..n {
                let x = c.pos(t - c.period_gyr * k as f64 / n as f64);
                let r = m::sqrt(dot(x, x));
                if r > far {
                    far = r;
                    dir = [x[0] / r, x[1] / r, x[2] / r];
                }
            }
            let cap = rp.shell_cap;
            let cap_norm = 2.0 * m::PI * cap * (1.0 - m::exp(-2.0 / cap));
            pieces.push((
                i,
                Piece::Shell {
                    mass: young,
                    b: far.max(c.apo_kpc),
                    w: (c.spread * far).max(rp.min_width_kpc),
                    dir,
                    cap,
                    cap_norm,
                },
            ));
            continue;
        }
        // A stream: ahead of and behind where it is now, its half length in
        // time along the orbit growing with the time since its stars left.
        let half = (rp.length_k * c.spread * (t - t_left).max(0.0))
            .min(0.5 * c.period_gyr)
            .max(1.0e-3);
        let here = c.pos(t);
        let r = m::sqrt(dot(here, here)).max(0.1);
        let sigma = (c.spread * r).max(rp.min_width_kpc);
        let speed = 2.0 * m::PI * r / c.period_gyr.max(1.0e-3);
        let n = (m::ceil(2.0 * half * speed / sigma) as usize).clamp(4, 96);
        let mut blobs = Vec::with_capacity(n);
        let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        for k in 0..n {
            let tt = t - half + 2.0 * half * (k as f64 + 0.5) / n as f64;
            let x = c.pos(tt);
            for j in 0..3 {
                lo[j] = lo[j].min(x[j] - 4.0 * sigma);
                hi[j] = hi[j].max(x[j] + 4.0 * sigma);
            }
            blobs.push(x);
        }
        pieces.push((
            i,
            Piece::Stream {
                mass: young,
                sigma,
                blobs,
                lo,
                hi,
            },
        ));
    }
    Shapes { t_gyr: t, pieces }
}

impl Piece {
    /// Solar masses per kpc³ at `x` (kpc).
    fn density(&self, x: [f64; 3]) -> f64 {
        match self {
            Piece::Mixed {
                mass,
                a,
                b,
                h,
                norm,
            } => {
                let r = m::sqrt(dot(x, x));
                let f = if r < *a {
                    1.0 / (a * a)
                } else if r <= *b {
                    1.0 / (r * r)
                } else {
                    m::exp(-(r - b) / h) / (b * b)
                };
                mass * f / norm
            }
            Piece::Shell {
                mass,
                b,
                w,
                dir,
                cap,
                cap_norm,
            } => {
                let r = m::sqrt(dot(x, x)).max(1.0e-6);
                let d = (r - b) / w;
                if d.abs() > 6.0 {
                    return 0.0;
                }
                let mu = dot(x, *dir) / r;
                let radial = m::exp(-0.5 * d * d) / (m::sqrt(2.0 * m::PI) * w);
                mass * radial / (r * r) * m::exp((mu - 1.0) / cap) / cap_norm
            }
            Piece::Stream {
                mass,
                sigma,
                blobs,
                lo,
                hi,
            } => {
                if (0..3).any(|j| x[j] < lo[j] || x[j] > hi[j]) {
                    return 0.0;
                }
                let s2 = sigma * sigma;
                let k = mass / blobs.len() as f64 / (m::pow(2.0 * m::PI, 1.5) * s2 * sigma);
                blobs
                    .iter()
                    .map(|c| {
                        let d = [x[0] - c[0], x[1] - c[1], x[2] - c[2]];
                        let q = dot(d, d) / s2;
                        if q > 32.0 {
                            0.0
                        } else {
                            m::exp(-0.5 * q)
                        }
                    })
                    .sum::<f64>()
                    * k
            }
        }
    }
}

impl Cache {
    /// The count tree's class grid for the remains' ages (`oldest`, at
    /// `t_now`), made once.
    pub fn class_grid(
        &self,
        oldest: f64,
        t_now: f64,
        make: impl FnOnce() -> std::rc::Rc<ClassGrid>,
    ) -> std::rc::Rc<ClassGrid> {
        let key = (oldest.to_bits(), t_now.to_bits());
        if let Some(g) = self.grids.borrow().iter().find(|g| (g.0, g.1) == key) {
            return g.2.clone();
        }
        let g = make();
        let mut gs = self.grids.borrow_mut();
        gs.insert(0, (key.0, key.1, g.clone()));
        gs.truncate(4);
        g
    }

    /// Each component's stars per kpc³ at `x` (kpc) at time `t`, in
    /// component order (zeros left out).
    pub fn densities(
        &self,
        comps: &[Component],
        op: &OrbitParams,
        rp: &RemainsParams,
        x: [f64; 3],
        t: f64,
    ) -> Vec<(usize, f64)> {
        if comps.is_empty() {
            return vec![];
        }
        let mut at = self.at.borrow_mut();
        let k = match at.iter().position(|s| s.t_gyr == t) {
            Some(k) => k,
            None => {
                at.insert(0, shapes(comps, op, rp, t));
                at.truncate(4);
                0
            }
        };
        let mut out: Vec<(usize, f64)> = vec![];
        for (i, p) in &at[k].pieces {
            let d = p.density(x);
            if d <= 0.0 {
                continue;
            }
            match out.last_mut() {
                Some(l) if l.0 == *i => l.1 += d,
                _ => out.push((*i, d)),
            }
        }
        out
    }

    /// Which shape of component `comp` is densest at `x` at `t`: `stream`,
    /// `shell` or `mixed remains`.
    pub fn kind_at(
        &self,
        comps: &[Component],
        op: &OrbitParams,
        rp: &RemainsParams,
        x: [f64; 3],
        t: f64,
        comp: usize,
    ) -> &'static str {
        let _ = self.densities(comps, op, rp, x, t);
        let at = self.at.borrow();
        let Some(sh) = at.iter().find(|s| s.t_gyr == t) else {
            return "mixed remains";
        };
        let mut best = ("mixed remains", -1.0);
        for (i, p) in &sh.pieces {
            if *i != comp {
                continue;
            }
            let d = p.density(x);
            if d > best.1 {
                best = (
                    match p {
                        Piece::Mixed { .. } => "mixed remains",
                        Piece::Shell { .. } => "shell",
                        Piece::Stream { .. } => "stream",
                    },
                    d,
                );
            }
        }
        best.0
    }

    /// All components' stars per kpc³ at `x` at `t`.
    pub fn density(
        &self,
        comps: &[Component],
        op: &OrbitParams,
        rp: &RemainsParams,
        x: [f64; 3],
        t: f64,
    ) -> f64 {
        self.densities(comps, op, rp, x, t)
            .iter()
            .map(|d| d.1)
            .sum()
    }
}

/// The mass in each shape, integrated on a grid (tests).
#[cfg(test)]
pub(crate) fn total_on_grid(
    comps: &[Component],
    op: &OrbitParams,
    rp: &RemainsParams,
    t: f64,
    half_kpc: f64,
    n: usize,
) -> f64 {
    let c = Cache::default();
    let h = 2.0 * half_kpc / n as f64;
    let mut s = 0.0;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let x = [
                    -half_kpc + (i as f64 + 0.5) * h,
                    -half_kpc + (j as f64 + 0.5) * h,
                    -half_kpc + (k as f64 + 0.5) * h,
                ];
                s += c.density(comps, op, rp, x, t);
            }
        }
    }
    s * h * h * h
}
