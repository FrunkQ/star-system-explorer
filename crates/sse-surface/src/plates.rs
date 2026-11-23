//! S2: the plate sketch (surface design section 2.2).
//!
//! A coarse kinematic model on the sphere: moved geometry, not fluid dynamics.
//! The sphere is the cube grid of `cells_per_face`² cells a face (64 gives
//! 24,576 cells). Plates are grown from keyed seed points; each turns about
//! its own keyed pole at a rate scaled by the world's heat flow. History is
//! stepped (25 Myr) through every mobile-lid span of the S0 timeline.
//!
//! Each step is semi-Lagrangian: every fixed cell asks each plate where its
//! material would have come from (the cell rotated back by that plate's turn)
//! and whether that source belonged to the plate.
//! - No plate claims it: the plates pulled apart here, so new ocean crust is
//!   born (a ridge), stamped with the date.
//! - One plate: the crust moves on unchanged.
//! - Several: plates converge. Continent beats ocean and younger ocean beats
//!   older (the older, denser plate sinks at a trench); the overriding side
//!   gets a volcanic arc, which thickens ocean crust towards continent.
//!   Continent meets continent: the crust thickens and a mountain-building
//!   episode is stamped with its date.
//!
//! Reorganisations (a pole jump, a plate splitting, a plate consumed) are keyed
//! events at a rate tied to heat flow, recorded with dates. Mantle plumes are
//! fixed in the deep frame; plates moving over them carry age-ordered volcano
//! chains, and rare floods of lava cover whole provinces. A snapshot of every
//! cell's plate is kept every few steps so the time ruler can show the drift.

use crate::atlas::{direction, face_tangents};
use crate::sketch::dot;
use crate::{SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use sse_core::Address;
use std::collections::BTreeMap;

pub const PLATES_VERSION: &str = "surface-plates-v2";

/// Samples along a step when dating new sea floor at a ridge.
const BIRTH_SAMPLES: u32 = 8;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlateSettings {
    pub cells_per_face: u32,
    pub step_myr: f64,
    pub plates_min: u32,
    pub plates_max: u32,
    pub speed_cm_yr: f64,
    pub q_ref_w_m2: f64,
    pub reorg_myr: f64,
    pub split_myr: f64,
    pub split_min_fraction: f64,
    /// A continental collision front this long (fraction of all cells) sutures two plates.
    pub suture_fraction: f64,
    pub initial_continent_fraction: f64,
    pub continental_km: f64,
    pub oceanic_km: f64,
    pub arc_growth_km: f64,
    pub continental_threshold_km: f64,
    pub collision_thickening_km: f64,
    pub max_thickness_km: f64,
    pub plumes: u32,
    pub lip_per_gyr: f64,
    pub lip_radius_rad: f64,
    pub orogeny_height_m: f64,
    pub erosion_tau_myr: f64,
    pub max_relief_m: f64,
    pub arc_height_m: f64,
    pub arc_tau_myr: f64,
    pub trench_depth_m: f64,
    pub trench_tau_myr: f64,
    pub plume_height_m: f64,
    pub plume_tau_myr: f64,
    pub continent_base_m: f64,
    pub airy: f64,
    pub snapshot_every: u32,
    /// The coarse Orbit run this run refines (S16); None runs free.
    #[serde(default)]
    pub guide: Option<PlateGuide>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plate {
    pub id: u16,
    pub pole: [f64; 3],
    /// Radians per Myr, about the pole.
    pub rate: f64,
}

/// One cell's crust and its history, carried with the moving material.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub plate: u16,
    pub continental: bool,
    /// Ocean crust: when it was born at a ridge. Continent: when it became continent.
    pub born_myr: f64,
    pub thickness_km: f64,
    /// Mountain-building episodes: (date, strength), newest last, at most four.
    pub orogeny: Vec<(f64, f64)>,
    /// Last date it stood on a volcanic arc.
    pub arc_myr: Option<f64>,
    /// Last date older crust sank beneath it at a trench.
    pub trench_myr: Option<f64>,
    /// Plume volcanism: (date, plume), newest last, at most six.
    pub volcanic: Vec<(f64, u16)>,
    pub flood_myr: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlateEvent {
    pub t_myr: f64,
    pub kind: String,
    pub plates: Vec<u16>,
    pub text: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub t_myr: f64,
    pub plate: Vec<u16>,
    pub continental: Vec<bool>,
    /// Every cell's crust at the date, kept only by [`history_recorded`]
    /// (the time-lapse frames); never serialised.
    #[serde(skip)]
    pub cells: Vec<Cell>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundaryKind {
    Spreading,
    Colliding,
    Sliding,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Boundary {
    pub kind: BoundaryKind,
    pub distance_km: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlateHistory {
    pub address: Address,
    pub n: u32,
    /// Myr since the world formed.
    pub epoch_myr: f64,
    pub steps: u32,
    /// The coarse grid (cells a face) this history was pinned to, if any.
    #[serde(default)]
    pub guided_by: Option<u32>,
    pub plates: Vec<Plate>,
    pub plumes: Vec<[f64; 3]>,
    pub cells: Vec<Cell>,
    pub events: Vec<PlateEvent>,
    pub snapshots: Vec<Snapshot>,
    /// Per cell, the nearest boundary of each kind (spreading, colliding, sliding).
    pub boundaries: Vec<[Option<Boundary>; 3]>,
}

// ---------------------------------------------------------------- grid

#[derive(Clone, Debug)]
pub struct Grid {
    pub n: u32,
    pub dirs: Vec<[f64; 3]>,
    nb: Vec<[usize; 4]>,
    /// Cell edges along a face as tangents of the warped coordinate, so
    /// `cell_at` finds a cell by comparison instead of two arctangents.
    edges: Vec<f64>,
}

impl Grid {
    pub fn new(n: u32) -> Self {
        let mut dirs = Vec::with_capacity((6 * n * n) as usize);
        for f in 0..6u8 {
            for j in 0..n {
                for i in 0..n {
                    let u = -1.0 + 2.0 * (i as f64 + 0.5) / n as f64;
                    let v = -1.0 + 2.0 * (j as f64 + 0.5) / n as f64;
                    dirs.push(direction(f, u, v));
                }
            }
        }
        let edges = (1..n)
            .map(|i| m::tan((-1.0 + 2.0 * i as f64 / n as f64) * m::PI / 4.0))
            .collect();
        let mut g = Grid {
            n,
            dirs,
            nb: vec![],
            edges,
        };
        g.nb = (0..g.dirs.len()).map(|c| g.neighbours_of(c)).collect();
        g
    }

    pub fn len(&self) -> usize {
        self.dirs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dirs.is_empty()
    }

    pub fn cell_at(&self, d: [f64; 3]) -> usize {
        let (f, a, b) = face_tangents(d);
        let i = self.edges.partition_point(|e| *e <= a);
        let j = self.edges.partition_point(|e| *e <= b);
        (f as usize) * (self.n * self.n) as usize + j * self.n as usize + i
    }

    /// The four neighbours (left, right, down, up), across face edges.
    pub fn neighbours(&self, c: usize) -> [usize; 4] {
        self.nb[c]
    }

    fn neighbours_of(&self, c: usize) -> [usize; 4] {
        let nn = (self.n * self.n) as usize;
        let f = (c / nn) as u8;
        let r = c % nn;
        let (j, i) = (r / self.n as usize, r % self.n as usize);
        let n = self.n as f64;
        let at = |di: f64, dj: f64| {
            let u = -1.0 + 2.0 * (i as f64 + 0.5 + di) / n;
            let v = -1.0 + 2.0 * (j as f64 + 0.5 + dj) / n;
            self.cell_at(direction(f, u, v))
        };
        [at(-1.0, 0.0), at(1.0, 0.0), at(0.0, -1.0), at(0.0, 1.0)]
    }
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

/// Rodrigues rotation of `v` about unit `k` by an angle with cosine `c`, sine `s`.
fn rotate(v: [f64; 3], k: [f64; 3], c: f64, s: f64) -> [f64; 3] {
    let kv = cross(k, v);
    let kd = dot(k, v) * (1.0 - c);
    [
        v[0] * c + kv[0] * s + k[0] * kd,
        v[1] * c + kv[1] * s + k[1] * kd,
        v[2] * c + kv[2] * s + k[2] * kd,
    ]
}

fn unit(d: &Draws, i: u64) -> [f64; 3] {
    norm([d.normal(3 * i), d.normal(3 * i + 1), d.normal(3 * i + 2)])
}

fn push_capped<T>(v: &mut Vec<T>, x: T, cap: usize) {
    v.push(x);
    if v.len() > cap {
        v.remove(0);
    }
}

// ---------------------------------------------------------------- history

/// Heat flow (radiogenic plus tidal) at a date, from the envelope's spans.
fn heat_at(env: &SurfaceEnvelope, t_gyr: f64) -> f64 {
    for s in &env.timeline {
        if t_gyr >= s.from_gyr && t_gyr <= s.to_gyr {
            let w = if s.to_gyr > s.from_gyr {
                (t_gyr - s.from_gyr) / (s.to_gyr - s.from_gyr)
            } else {
                0.0
            };
            // Heat falls roughly exponentially: interpolate in log.
            let (a, b) = (s.heat_flow_w_m2[0].max(1e-9), s.heat_flow_w_m2[1].max(1e-9));
            return m::exp(m::ln(a) + (m::ln(b) - m::ln(a)) * w);
        }
    }
    0.0
}

fn mobile_at(env: &SurfaceEnvelope, t_gyr: f64) -> bool {
    env.timeline
        .iter()
        .any(|s| s.regime == "mobile-lid" && t_gyr >= s.from_gyr && t_gyr < s.to_gyr)
}

struct Run<'a> {
    p: &'a PlateSettings,
    d: Draws,
    grid: Grid,
    plates: Vec<Plate>,
    next_id: u16,
    cells: Vec<Cell>,
    events: Vec<PlateEvent>,
    draw_i: u64,
    /// The plates as they moved in the last step, before that step's
    /// events turned or split them: the boundaries are classified by the
    /// motion that made the crust beside them.
    moved: Vec<Plate>,
}

impl Run<'_> {
    fn next(&mut self) -> u64 {
        self.draw_i += 1;
        self.draw_i
    }

    fn rate_for(&mut self, q: f64) -> f64 {
        // Plate speed in metres per Myr (1 cm/yr is 1e4 m/Myr), scaled by
        // heat flow; the caller divides by the radius for radians per Myr.
        let i = self.next();
        let spread = 0.3 + 1.4 * self.d.unit(i);
        self.p.speed_cm_yr * 1.0e4 * spread * (q / self.p.q_ref_w_m2).clamp(0.2, 5.0)
    }

    fn new_plate(&mut self, q: f64, radius_m: f64) -> Plate {
        let i = self.next();
        let pole = unit(&self.d, 10_000 + i);
        let rate = self.rate_for(q) / radius_m;
        let id = self.next_id;
        self.next_id += 1;
        Plate { id, pole, rate }
    }
}

/// The coarse run that guides a fine one (S16, detail levels): the Orbit
/// level's grid and step. A fine run with a guide takes its plates, their
/// poles and speeds and every dated event from the coarse run, so the world
/// seen from orbit and the surveyed world are the same world.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlateGuide {
    pub cells_per_face: u32,
    pub step_myr: f64,
}

/// The plate settings of the coarse (Orbit) run that guides `p`, or None
/// when `p` runs free (no guide, or a guide no coarser than `p`).
pub fn guide_settings(p: &PlateSettings) -> Option<PlateSettings> {
    // A guide must be coarser than the run it guides; otherwise run free.
    let g = p
        .guide
        .as_ref()
        .filter(|g| g.cells_per_face < p.cells_per_face)?;
    let mut c = p.clone();
    c.cells_per_face = g.cells_per_face;
    c.step_myr = g.step_myr;
    c.guide = None;
    Some(c)
}

/// Run the plate history of a world up to `epoch_gyr` (Gyr since it formed).
/// Worlds with no mobile-lid span get a single still plate and no steps.
///
/// With `plates.guide` set, the run is the Survey level of S16: the coarse
/// Orbit run (`guide_settings`) is stepped, and the fine grid moves with the
/// same plates inside each coarse step, then is pinned to it (see `pin`).
/// Without, the run is free: one grid, its own events.
pub fn history(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
) -> PlateHistory {
    run_history(s, env, seed, epoch_gyr, false)
}

/// [`history`], keeping every cell's crust at each snapshot (for frames).
/// The same run: recording changes nothing it computes.
pub fn history_recorded(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
) -> PlateHistory {
    run_history(s, env, seed, epoch_gyr, true)
}

fn run_history(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
    record: bool,
) -> PlateHistory {
    let coarse_p = guide_settings(&s.plates);
    let p = coarse_p.as_ref().unwrap_or(&s.plates);
    let fine_p = coarse_p.as_ref().map(|_| &s.plates);
    let d = Draws::new(
        *seed,
        env.address.clone(),
        Stream::new("surface.plates.v1").expect("valid"),
    );
    let r = env.reference_radius_m.max(1.0);
    let end = env.timeline.last().map(|x| x.to_gyr).unwrap_or(0.0);
    let epoch = epoch_gyr.clamp(0.0, end);
    let start = env
        .timeline
        .iter()
        .find(|x| x.regime == "mobile-lid")
        .map(|x| x.from_gyr);
    let mut run = Run {
        p,
        d,
        grid: Grid::new(p.cells_per_face),
        plates: vec![],
        next_id: 0,
        cells: vec![],
        events: vec![],
        draw_i: 0,
        moved: vec![],
    };
    let plumes: Vec<[f64; 3]> = (0..p.plumes)
        .map(|i| unit(&run.d, 20_000 + i as u64))
        .collect();
    let still = start.is_none_or(|t0| t0 >= epoch);
    // Seed plates: multiplicatively weighted Voronoi cells around keyed points.
    let q0 = start.map(|t| heat_at(env, t)).unwrap_or(0.0);
    let count = if still {
        1
    } else {
        p.plates_min + run.d.below(1, (p.plates_max - p.plates_min + 1) as u64) as u32
    };
    let mut seeds = vec![];
    for _ in 0..count {
        let pl = run.new_plate(q0, r);
        let i = run.next();
        let j = run.next();
        seeds.push((unit(&run.d, 30_000 + i), 0.6 + 0.8 * run.d.unit(j)));
        run.plates.push(pl);
    }
    if still {
        run.plates[0].rate = 0.0;
    }
    // Continental nuclei: the top cells of a keyed bump field.
    let nuclei: Vec<([f64; 3], f64)> = (0..6)
        .map(|k| {
            (
                unit(&run.d, 40_000 + k),
                0.02 + 0.05 * run.d.unit(50_000 + k),
            )
        })
        .collect();
    let t0_myr = start.unwrap_or(0.0) * 1000.0;
    let ids: Vec<u16> = run.plates.iter().map(|x| x.id).collect();
    run.cells = initial_cells(&run.grid, p, &seeds, &ids, &nuclei, t0_myr);
    // The fine run, when guided: the same seeds on its own grid. It draws
    // nothing of its own.
    let mut fine = fine_p.map(|fp| {
        let grid = Grid::new(fp.cells_per_face);
        let cells = initial_cells(&grid, fp, &seeds, &ids, &nuclei, t0_myr);
        Run {
            p: fp,
            d: run.d.clone(),
            grid,
            plates: run.plates.clone(),
            next_id: run.next_id,
            cells,
            events: vec![],
            draw_i: 0,
            moved: vec![],
        }
    });
    let snap = |x: &Run, t: f64| Snapshot {
        t_myr: t,
        plate: x.cells.iter().map(|c| c.plate).collect(),
        continental: x.cells.iter().map(|c| c.continental).collect(),
        cells: if record { x.cells.clone() } else { vec![] },
    };
    let mut snapshots = vec![];
    let mut steps = 0u32;
    let shown_every = fine_p.unwrap_or(p).snapshot_every.max(1);
    if !still {
        let end_myr = epoch * 1000.0;
        let mut t = start.expect("start") * 1000.0;
        snapshots.push(snap(fine.as_ref().unwrap_or(&run), t));
        let ratio = fine_p.map_or(1, |fp| (m::round(p.step_myr / fp.step_myr) as u32).max(1));
        // Whole coarse steps, then one shorter step that lands on the epoch
        // exactly, so the coarse and fine runs end at the same date.
        while t < end_myr - 1e-9 {
            let dt = p.step_myr.min(end_myr - t);
            let t_gyr = t / 1000.0;
            let t1 = t + dt;
            if mobile_at(env, t_gyr) {
                let q = heat_at(env, t_gyr);
                match fine.as_mut() {
                    Some(f) => {
                        let k = (m::ceil(dt / p.step_myr * ratio as f64 - 1e-9) as u32).max(1);
                        let fdt = dt / k as f64;
                        f.plates = run.plates.clone();
                        f.moved = run.plates.clone();
                        for i in 1..=k {
                            let (next, _) = drift(f, &plumes, t + fdt * i as f64, fdt);
                            f.cells = next;
                            steps += 1;
                            if i < k && steps.is_multiple_of(shown_every) {
                                snapshots.push(snap(f, t + fdt * i as f64));
                            }
                        }
                        step_once(&mut run, &plumes, t1, dt, q, r);
                        pin(f, &run, t1);
                        if steps.is_multiple_of(shown_every) {
                            snapshots.push(snap(f, t1));
                        }
                    }
                    None => {
                        step_once(&mut run, &plumes, t1, dt, q, r);
                        steps += 1;
                        if steps.is_multiple_of(shown_every) {
                            snapshots.push(snap(&run, t1));
                        }
                    }
                }
            }
            t = t1;
        }
        // The time ruler always ends on the epoch itself.
        if snapshots.last().is_some_and(|x| x.t_myr < end_myr - 1e-9) {
            snapshots.push(snap(fine.as_ref().unwrap_or(&run), end_myr));
        }
    }
    let guided_by = fine.as_ref().map(|_| p.cells_per_face);
    let out = match fine {
        Some(mut f) => {
            f.events = std::mem::take(&mut run.events);
            f
        }
        None => run,
    };
    let boundaries = if still {
        vec![[None, None, None]; out.grid.len()]
    } else {
        boundaries(&out, r)
    };
    PlateHistory {
        address: env.address.clone(),
        n: out.p.cells_per_face,
        epoch_myr: epoch * 1000.0,
        steps,
        guided_by,
        plates: out.plates,
        plumes,
        cells: out.cells,
        events: out.events,
        snapshots,
        boundaries,
    }
}

/// Every cell of `grid` at the start: its plate from the weighted Voronoi
/// seeds, continent where the nuclei field is highest.
fn initial_cells(
    grid: &Grid,
    p: &PlateSettings,
    seeds: &[([f64; 3], f64)],
    ids: &[u16],
    nuclei: &[([f64; 3], f64)],
    t0_myr: f64,
) -> Vec<Cell> {
    let nc = grid.len();
    let score: Vec<f64> = grid
        .dirs
        .iter()
        .map(|v| {
            nuclei
                .iter()
                .map(|(c, w)| m::exp(-(1.0 - dot(*v, *c)) / w))
                .fold(0.0, f64::max)
        })
        .collect();
    let mut order: Vec<usize> = (0..nc).collect();
    order.sort_by(|&a, &b| score[b].total_cmp(&score[a]).then(a.cmp(&b)));
    let ncont = m::round(p.initial_continent_fraction * nc as f64) as usize;
    let mut is_cont = vec![false; nc];
    for &c in order.iter().take(ncont) {
        is_cont[c] = true;
    }
    (0..nc)
        .map(|c| {
            let v = grid.dirs[c];
            let pl = seeds
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    ((1.0 - dot(v, a.1 .0)) * a.1 .1)
                        .total_cmp(&((1.0 - dot(v, b.1 .0)) * b.1 .1))
                        .then(a.0.cmp(&b.0))
                })
                .map(|x| x.0)
                .unwrap_or(0);
            Cell {
                plate: ids[pl],
                continental: is_cont[c],
                born_myr: t0_myr,
                thickness_km: if is_cont[c] {
                    p.continental_km
                } else {
                    p.oceanic_km
                },
                orogeny: vec![],
                arc_myr: None,
                trench_myr: None,
                volcanic: vec![],
                flood_myr: None,
            }
        })
        .collect()
}

/// Pin the fine run to the coarse one at the end of a coarse step (S16:
/// "the coarser level is a pin for the finer one"). Plates, poles and speeds
/// are the coarse ones. A fine cell inside a coarse cell whose four
/// neighbours share its plate and crust kind takes that plate and crust
/// kind; near a coarse boundary or coast the fine cell keeps what its own
/// steps made, so belts and coasts get their fine shape but move by at most
/// one coarse cell. Floods the coarse run stamped are stamped here too.
fn pin(fine: &mut Run, coarse: &Run, t: f64) {
    let newborn: Vec<u16> = coarse
        .plates
        .iter()
        .map(|x| x.id)
        .filter(|id| !fine.plates.iter().any(|y| y.id == *id))
        .collect();
    fine.plates = coarse.plates.clone();
    fine.next_id = coarse.next_id;
    let interior: Vec<bool> = (0..coarse.grid.len())
        .map(|c| {
            let a = &coarse.cells[c];
            coarse.grid.neighbours(c).iter().all(|&n| {
                coarse.cells[n].plate == a.plate && coarse.cells[n].continental == a.continental
            })
        })
        .collect();
    let spacing = (m::PI / 2.0) / coarse.grid.n as f64;
    for i in 0..fine.grid.len() {
        let pc = coarse.grid.cell_at(fine.grid.dirs[i]);
        let par = &coarse.cells[pc];
        let f = &mut fine.cells[i];
        let alive = coarse.plates.iter().any(|x| x.id == f.plate);
        if (interior[pc] || newborn.contains(&par.plate) || !alive) && f.plate != par.plate {
            // The fine run put this material on another plate: the coarse
            // run's crust is the one that is here.
            f.plate = par.plate;
            if f.continental == par.continental {
                f.born_myr = par.born_myr;
                f.thickness_km = par.thickness_km;
            }
        }
        // Crust kind from the coarse field around the cell: clear land or
        // clear sea takes the coarse kind; only the band within about half a
        // coarse cell of a coarse coast keeps the fine run's own shape.
        let v = fine.grid.dirs[i];
        let (mut sw, mut sc) = (0.0, 0.0);
        for k in std::iter::once(pc).chain(coarse.grid.neighbours(pc)) {
            let ang = m::sqrt((2.0 * (1.0 - dot(v, coarse.grid.dirs[k]))).max(0.0));
            let w = (1.5 * spacing - ang).max(0.0);
            sw += w;
            if coarse.cells[k].continental {
                sc += w;
            }
        }
        let land = if sw > 0.0 { sc / sw } else { 0.5 };
        let kind = if interior[pc] || land >= 0.6 {
            Some(if interior[pc] { par.continental } else { true })
        } else if land <= 0.4 {
            Some(false)
        } else {
            None
        };
        if let Some(k) = kind {
            if f.continental != k {
                f.continental = k;
                if par.continental == k {
                    f.thickness_km = par.thickness_km;
                    f.born_myr = par.born_myr;
                } else {
                    let p = fine.p;
                    f.thickness_km = if k { p.continental_km } else { p.oceanic_km };
                    f.born_myr = t;
                }
            }
        }
        if par.flood_myr == Some(t) {
            f.flood_myr = Some(t);
        }
    }
}

fn step_once(run: &mut Run, plumes: &[[f64; 3]], t: f64, dt: f64, q: f64, r: f64) {
    run.moved = run.plates.clone();
    let (next, collisions) = drift(run, plumes, t, dt);
    events(run, next, collisions, plumes, t, dt, q, r);
}

/// One step of motion: every cell asks each plate where its material came
/// from; ridges, trenches, arcs and collisions follow; plumes stamp the
/// cells above them. Draws nothing, so a guided fine run can take it alone.
fn drift(
    run: &Run,
    plumes: &[[f64; 3]],
    t: f64,
    dt: f64,
) -> (Vec<Cell>, BTreeMap<(u16, u16), usize>) {
    let p = run.p;
    let nc = run.grid.len();
    let turns: Vec<(u16, [f64; 3], f64, f64)> = run
        .plates
        .iter()
        .map(|pl| {
            let a = -pl.rate * dt;
            (pl.id, pl.pole, m::cos(a), m::sin(a))
        })
        .collect();
    let prev = &run.cells;
    // Which plates can claim each cell: turn every cell forward with its own
    // plate and mark where it lands. A plate claims a cell only if the cell
    // turned back lands on that plate's ground, and turning is an isometry,
    // so the claimant's forward mark falls in the cell or one beside it
    // (diagonals included). Only those plates are tested below.
    let masks: Option<Vec<u64>> = (turns.len() <= 64).then(|| {
        let mut hit = vec![0u64; nc];
        for (c, cell) in prev.iter().enumerate() {
            if let Some(ti) = turns.iter().position(|x| x.0 == cell.plate) {
                let (_, k, co, si) = turns[ti];
                hit[run.grid.cell_at(rotate(run.grid.dirs[c], k, co, -si))] |= 1u64 << ti;
            }
        }
        (0..nc)
            .map(|c| {
                let nb = run.grid.neighbours(c);
                let mut mk = hit[c];
                for &x in &nb {
                    mk |= hit[x];
                }
                for &x in &[nb[0], nb[1]] {
                    let n2 = run.grid.neighbours(x);
                    mk |= hit[n2[2]] | hit[n2[3]];
                }
                mk
            })
            .collect()
    });
    // Plate order for convergence: more continent first, then younger ocean.
    let mut stats: BTreeMap<u16, (usize, usize, f64)> = BTreeMap::new();
    for c in prev {
        let e = stats.entry(c.plate).or_insert((0, 0, 0.0));
        e.0 += 1;
        if c.continental {
            e.1 += 1;
        } else {
            e.2 += t - c.born_myr;
        }
    }
    let mut ranked: Vec<(u16, f64, f64)> = stats
        .iter()
        .map(|(id, (n, cn, age))| {
            let ocean = (n - cn).max(1) as f64;
            (*id, *cn as f64 / *n as f64, age / ocean)
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then(a.2.total_cmp(&b.2))
            .then(a.0.cmp(&b.0))
    });
    let order: BTreeMap<u16, usize> = ranked.iter().enumerate().map(|(i, r)| (r.0, i)).collect();
    let mut next = Vec::with_capacity(nc);
    let mut collisions: BTreeMap<(u16, u16), usize> = BTreeMap::new();
    for c in 0..nc {
        let v = run.grid.dirs[c];
        let mut cands: Vec<(u16, usize)> = vec![];
        for (ti, &(id, k, co, si)) in turns.iter().enumerate() {
            if masks.as_ref().is_some_and(|mk| mk[c] & (1u64 << ti) == 0) {
                continue;
            }
            let src = run.grid.cell_at(rotate(v, k, co, si));
            if prev[src].plate == id {
                cands.push((id, src));
            }
        }
        let cell = match cands.len() {
            0 => {
                // The plates pulled apart: new ocean crust at a ridge. The
                // ridge stays where the plates' old boundary was, so the crust
                // here was born when its plate's motion carried it off that
                // boundary: trace it back along the plate's turn to where it
                // leaves the plate's old ground. Without this a long step
                // would date a whole strip of sea floor to the step's end.
                let own = prev[c].plate;
                let born = match run.plates.iter().find(|x| x.id == own) {
                    Some(pl) => {
                        let a = -pl.rate * dt;
                        let mut s_out = 1.0;
                        for k in 1..=BIRTH_SAMPLES {
                            let s = k as f64 / BIRTH_SAMPLES as f64;
                            let src =
                                run.grid
                                    .cell_at(rotate(v, pl.pole, m::cos(a * s), m::sin(a * s)));
                            if prev[src].plate != own {
                                s_out = (k as f64 - 0.5) / BIRTH_SAMPLES as f64;
                                break;
                            }
                        }
                        t - s_out * dt
                    }
                    None => t,
                };
                Cell {
                    plate: own,
                    continental: false,
                    born_myr: born,
                    thickness_km: p.oceanic_km,
                    orogeny: vec![],
                    arc_myr: None,
                    trench_myr: None,
                    volcanic: vec![],
                    flood_myr: None,
                }
            }
            1 => prev[cands[0].1].clone(),
            _ => {
                // Converging. Continental crust never sinks; otherwise the
                // plate-level order decides (more continent, then younger
                // ocean), so a boundary has one overriding side, not a speckle.
                let key = |x: &(u16, usize)| {
                    let cont = if prev[x.1].continental { 0u8 } else { 1 };
                    (cont, order.get(&x.0).copied().unwrap_or(usize::MAX), x.0)
                };
                let best = *cands.iter().min_by_key(|x| key(x)).expect("candidates");
                let mut cell = prev[best.1].clone();
                let losers: Vec<&(u16, usize)> = cands.iter().filter(|x| x.0 != best.0).collect();
                let cont_loser = losers.iter().any(|x| prev[x.1].continental);
                if cell.continental && cont_loser {
                    // Continent meets continent: thickening and a mountain belt.
                    cell.thickness_km =
                        (cell.thickness_km + p.collision_thickening_km).min(p.max_thickness_km);
                    push_capped(&mut cell.orogeny, (t, 1.0), 4);
                    for l in &losers {
                        let key = (best.0.min(l.0), best.0.max(l.0));
                        *collisions.entry(key).or_default() += 1;
                    }
                } else {
                    // Ocean sinks under the winner: a trench, and an arc above it.
                    cell.trench_myr = Some(t);
                    cell.arc_myr = Some(t);
                    if cell.continental {
                        push_capped(&mut cell.orogeny, (t, 0.5), 4);
                    } else {
                        cell.thickness_km += p.arc_growth_km;
                        if cell.thickness_km >= p.continental_threshold_km {
                            cell.continental = true;
                            cell.born_myr = t;
                        }
                    }
                }
                cell
            }
        };
        next.push(cell);
    }
    // Sampling on a fixed grid leaves stray cells claimed by the wrong plate;
    // a cell that disagrees with at least three of its four neighbours, and
    // those agree, joins them (its crust is kept).
    let mut fixed = vec![];
    for c in 0..nc {
        let nb = run.grid.neighbours(c);
        let own = next[c].plate;
        let mut counts: Vec<(u16, u32)> = vec![];
        for &x in &nb {
            let pl = next[x].plate;
            match counts.iter_mut().find(|e| e.0 == pl) {
                Some(e) => e.1 += 1,
                None => counts.push((pl, 1)),
            }
        }
        if let Some(&(pl, k)) = counts
            .iter()
            .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        {
            if pl != own && k >= 3 {
                fixed.push((c, pl));
            }
        }
    }
    for (c, pl) in fixed {
        next[c].plate = pl;
    }
    // Plumes in the deep frame.
    for (i, pl) in plumes.iter().enumerate() {
        let c = run.grid.cell_at(*pl);
        push_capped(&mut next[c].volcanic, (t, i as u16), 6);
    }
    (next, collisions)
}

/// The step's events, all drawn: floods, sutures and collision turns,
/// consumed plates, pole jumps and splits.
#[allow(clippy::too_many_arguments)]
fn events(
    run: &mut Run,
    mut next: Vec<Cell>,
    collisions: BTreeMap<(u16, u16), usize>,
    plumes: &[[f64; 3]],
    t: f64,
    dt: f64,
    q: f64,
    r: f64,
) {
    let p = run.p;
    let nc = run.grid.len();
    // Rare flood eruptions.
    let i = run.next();
    if run.d.unit(60_000 + i) < p.lip_per_gyr * dt / 1000.0 && !plumes.is_empty() {
        let j = run.next();
        let k = run.d.below(j, plumes.len() as u64) as usize;
        let cosr = m::cos(p.lip_radius_rad);
        let mut n = 0;
        for (c, cell) in next.iter_mut().enumerate() {
            if dot(run.grid.dirs[c], plumes[k]) >= cosr {
                cell.flood_myr = Some(t);
                n += 1;
            }
        }
        run.events.push(PlateEvent {
            t_myr: t,
            kind: "flood-basalt".into(),
            plates: vec![],
            text: format!("A mantle plume floods {n} cells of crust with lava."),
        });
    }
    // Continents that collide along a front: a long front sutures the two
    // plates into one (the smaller joins the larger); a short one turns the
    // smaller plate onto a new pole (the collision reorganises it).
    let mut joined: BTreeMap<u16, u16> = BTreeMap::new();
    let mut turned: Vec<u16> = vec![];
    let mut sizes: BTreeMap<u16, usize> = BTreeMap::new();
    for c in &next {
        *sizes.entry(c.plate).or_default() += 1;
    }
    for ((a, b), n) in collisions {
        let (a, b) = (*joined.get(&a).unwrap_or(&a), *joined.get(&b).unwrap_or(&b));
        if n < 3 || a == b {
            continue;
        }
        let (keep, gone) = if sizes.get(&a) >= sizes.get(&b) {
            (a, b)
        } else {
            (b, a)
        };
        let alive = sizes.len();
        if n as f64 >= p.suture_fraction * nc as f64 && alive > p.plates_min as usize {
            for c in next.iter_mut() {
                if c.plate == gone {
                    c.plate = keep;
                }
            }
            for v in joined.values_mut() {
                if *v == gone {
                    *v = keep;
                }
            }
            joined.insert(gone, keep);
            let ng = sizes.remove(&gone).unwrap_or(0);
            *sizes.entry(keep).or_default() += ng;
            run.events.push(PlateEvent {
                t_myr: t,
                kind: "suture".into(),
                plates: vec![keep, gone],
                text: format!(
                    "Continents on plates {keep} and {gone} collide along {n} cells: a mountain belt rises and the plates suture into one."
                ),
            });
        } else {
            turned.push(gone);
            let recent = run
                .events
                .iter()
                .rev()
                .take_while(|e| t - e.t_myr < 100.0)
                .any(|e| e.kind == "collision" && e.plates == vec![keep, gone]);
            if recent {
                continue;
            }
            run.events.push(PlateEvent {
                t_myr: t,
                kind: "collision".into(),
                plates: vec![keep, gone],
                text: format!(
                    "Continents on plates {keep} and {gone} collide along {n} cells: a mountain belt rises and plate {gone} is turned."
                ),
            });
        }
    }
    for id in turned {
        if let Some(k) = run.plates.iter().position(|x| x.id == id) {
            let j = run.next();
            run.plates[k].pole = unit(&run.d, 110_000 + j);
            run.plates[k].rate = run.rate_for(q) / r;
        }
    }
    run.cells = next;
    // Plates that lost all their cells are gone.
    let mut size: BTreeMap<u16, usize> = BTreeMap::new();
    for c in &run.cells {
        *size.entry(c.plate).or_default() += 1;
    }
    let before: Vec<u16> = run.plates.iter().map(|x| x.id).collect();
    run.plates
        .retain(|x| size.get(&x.id).copied().unwrap_or(0) > 0);
    for id in before {
        if !size.contains_key(&id) && !joined.contains_key(&id) {
            run.events.push(PlateEvent {
                t_myr: t,
                kind: "plate-consumed".into(),
                plates: vec![id],
                text: format!("Plate {id} has been consumed at trenches."),
            });
        }
    }
    // Reorganisations, faster on a hotter world.
    let hot = m::sqrt((q / p.q_ref_w_m2).clamp(0.2, 5.0));
    for k in 0..run.plates.len() {
        let i = run.next();
        if run.d.unit(70_000 + i) < dt * hot / p.reorg_myr {
            let j = run.next();
            run.plates[k].pole = unit(&run.d, 80_000 + j);
            run.plates[k].rate = run.rate_for(q) / r;
            let id = run.plates[k].id;
            run.events.push(PlateEvent {
                t_myr: t,
                kind: "pole-jump".into(),
                plates: vec![id],
                text: format!("Plate {id} changes direction."),
            });
        }
    }
    let i = run.next();
    let few = if run.plates.len() < p.plates_min as usize {
        4.0
    } else {
        1.0
    };
    if run.d.unit(90_000 + i) < few * dt * hot / p.split_myr {
        let (big, n) = size
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(a, b)| (*a, *b))
            .unwrap_or((0, 0));
        if n as f64 >= p.split_min_fraction * nc as f64 && run.plates.len() < p.plates_max as usize
        {
            let j = run.next();
            let cut = unit(&run.d, 100_000 + j);
            // Through the plate's centre: cut plane normal orthogonal to the centroid.
            let mut cen = [0.0; 3];
            for (c, cell) in run.cells.iter().enumerate() {
                if cell.plate == big {
                    let v = run.grid.dirs[c];
                    cen = [cen[0] + v[0], cen[1] + v[1], cen[2] + v[2]];
                }
            }
            let cen = norm(cen);
            let nrm = norm(cross(cen, cut));
            let newp = run.new_plate(q, r);
            let nid = newp.id;
            for (c, cell) in run.cells.iter_mut().enumerate() {
                if cell.plate == big && dot(run.grid.dirs[c], nrm) > 0.0 {
                    cell.plate = nid;
                }
            }
            run.plates.push(newp);
            run.events.push(PlateEvent {
                t_myr: t,
                kind: "plate-split".into(),
                plates: vec![big, nid],
                text: format!(
                    "Plate {big} rifts in two; the new plate {nid} moves off on its own."
                ),
            });
        }
    }
}

fn boundaries(run: &Run, r: f64) -> Vec<[Option<Boundary>; 3]> {
    let nc = run.grid.len();
    let motion = if run.moved.is_empty() {
        &run.plates
    } else {
        &run.moved
    };
    let vel = |id: u16, x: [f64; 3]| {
        let pl = motion.iter().find(|p| p.id == id);
        match pl {
            Some(p) => {
                let w = cross(p.pole, x);
                [w[0] * p.rate, w[1] * p.rate, w[2] * p.rate]
            }
            None => [0.0; 3],
        }
    };
    let mut seed: [Vec<usize>; 3] = [vec![], vec![], vec![]];
    for c in 0..nc {
        let x = run.grid.dirs[c];
        let a = run.cells[c].plate;
        for nb in run.grid.neighbours(c) {
            let b = run.cells[nb].plate;
            if b == a {
                continue;
            }
            let rel = {
                let (va, vb) = (vel(a, x), vel(b, run.grid.dirs[nb]));
                [va[0] - vb[0], va[1] - vb[1], va[2] - vb[2]]
            };
            let nv = norm([
                run.grid.dirs[nb][0] - x[0],
                run.grid.dirs[nb][1] - x[1],
                run.grid.dirs[nb][2] - x[2],
            ]);
            let vn = dot(rel, nv);
            let mag = m::sqrt(dot(rel, rel));
            let kind = if vn > 0.3 * mag {
                1
            } else if vn < -0.3 * mag {
                0
            } else {
                2
            };
            seed[kind].push(c);
        }
    }
    let spacing_km = r * (m::PI / 2.0) / run.grid.n as f64 / 1000.0;
    let mut out = vec![[None, None, None]; nc];
    for (k, s) in seed.iter().enumerate() {
        let mut dist = vec![u32::MAX; nc];
        let mut queue = std::collections::VecDeque::new();
        for &c in s {
            if dist[c] == u32::MAX {
                dist[c] = 0;
                queue.push_back(c);
            }
        }
        while let Some(c) = queue.pop_front() {
            for nb in run.grid.neighbours(c) {
                if dist[nb] == u32::MAX {
                    dist[nb] = dist[c] + 1;
                    queue.push_back(nb);
                }
            }
        }
        let kind = [
            BoundaryKind::Spreading,
            BoundaryKind::Colliding,
            BoundaryKind::Sliding,
        ][k];
        for c in 0..nc {
            if dist[c] != u32::MAX {
                out[c][k] = Some(Boundary {
                    kind,
                    distance_km: dist[c] as f64 * spacing_km,
                });
            }
        }
    }
    out
}

// ---------------------------------------------------------------- heights

/// Height of every cell at the history's epoch, metres against the datum
/// before sea level: sea floor from crust age (GDH1), continents by Airy
/// isostasy, mountain belts, arcs, trenches and plume volcanoes worn down
/// with their age.
pub fn cell_heights(s: &SurfaceSettings, h: &PlateHistory, gravity: f64) -> Vec<f64> {
    cells_heights(&s.plates, &h.cells, h.epoch_myr, gravity)
}

/// Heights of `cells` at `now` Myr (before wear), metres from the raw datum.
pub fn cells_heights(p: &PlateSettings, cells: &[Cell], now: f64, gravity: f64) -> Vec<f64> {
    let g = (sse_core::sheet::G0 / gravity.max(0.1)).clamp(0.5, 3.0);
    let cap = p.max_relief_m * g;
    cells
        .iter()
        .map(|c| {
            let (mut z, thick) = if c.continental {
                (
                    p.continent_base_m,
                    p.airy * (c.thickness_km - p.continental_km) * 1000.0,
                )
            } else {
                let age = (now - c.born_myr).max(0.0);
                (-gdh1_depth(age), 0.0)
            };
            let mut mtn = thick;
            for (t, st) in &c.orogeny {
                mtn +=
                    p.orogeny_height_m * g * st * m::exp(-(now - t).max(0.0) / p.erosion_tau_myr);
            }
            if let Some(t) = c.arc_myr {
                mtn += p.arc_height_m * g * m::exp(-(now - t).max(0.0) / p.arc_tau_myr);
            }
            for (t, _) in &c.volcanic {
                mtn += p.plume_height_m * g * m::exp(-(now - t).max(0.0) / p.plume_tau_myr) / 3.0;
            }
            // Soft cap: relief saturates at the strength of rock under this gravity.
            z += cap * m::tanh(mtn / cap);
            if let Some(t) = c.trench_myr {
                z -= p.trench_depth_m * m::exp(-(now - t).max(0.0) / p.trench_tau_myr);
            }
            z
        })
        .collect()
}

/// Sea-floor depth from crust age, metres (Stein and Stein 1992, GDH1).
pub fn gdh1_depth(age_myr: f64) -> f64 {
    if age_myr < 20.0 {
        2600.0 + 365.0 * m::sqrt(age_myr)
    } else {
        5651.0 - 2473.0 * m::exp(-0.0278 * age_myr)
    }
}

/// Sea level that holds `oceans` Earth oceans over the cells, metres.
pub fn sea_level(heights: &[f64], radius_m: f64, oceans: f64) -> Option<f64> {
    if oceans <= 0.0 || heights.is_empty() {
        return None;
    }
    let area = 4.0 * m::PI * radius_m * radius_m / heights.len() as f64;
    let want = oceans * 1.335e18;
    let lo0 = heights.iter().copied().fold(f64::INFINITY, f64::min);
    let hi0 = heights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let vol = |sl: f64| heights.iter().map(|h| (sl - h).max(0.0)).sum::<f64>() * area;
    let (mut lo, mut hi) = (lo0, hi0 + want / (4.0 * m::PI * radius_m * radius_m));
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if vol(mid) < want {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// The plate map at a date from the kept snapshots (nearest at or before).
pub fn plates_at(h: &PlateHistory, t_myr: f64) -> Option<&Snapshot> {
    h.snapshots
        .iter()
        .rev()
        .find(|s| s.t_myr <= t_myr + 1e-9)
        .or(h.snapshots.first())
}

pub fn grid_for(h: &PlateHistory) -> Grid {
    Grid::new(h.n)
}
