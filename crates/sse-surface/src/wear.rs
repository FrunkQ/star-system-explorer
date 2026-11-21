//! S3: wearing down (surface design section 3, term 6).
//!
//! Runs on the cell grid the relief came from: the plate history's cells on a
//! mobile-lid world, the regime sketch sampled at the same cells otherwise.
//! Each epoch (100 Myr):
//! - depressions are filled by priority-flood, so every land cell drains to the
//!   sea or to a lake and no river runs uphill;
//! - water is routed by steepest descent on the filled surface and
//!   accumulated from each cell's own runoff (W9: the one climate's rain and
//!   snowmelt at the cell's latitude and height, less what evaporates, by
//!   Fu's Budyko curve; none on a dry world);
//! - a cell is glaciated where its snow outlasts the summer (W9's ice
//!   balance at its height), not below a fixed temperature;
//! - valleys are cut by the stream-power rule (incision grows with discharge
//!   and slope; glaciers cut harder), solved implicitly in drainage order;
//! - what is worn off is carried downstream and laid down in lakes and at river
//!   mouths, so eroded and deposited volume balance exactly;
//! - slopes relax by diffusion, wet or dry;
//! - sea level is refitted to the water volume, so shelves flood and drain.
//!
//! On a mobile-lid world the relief is young (S2 already wore old belts down),
//! so only the last `plate_window_myr` is carved; elsewhere the whole history
//! from the magma ocean's end is.

use crate::climate::WorldClimate;
use crate::plates::{sea_level, Grid, PlateHistory};
use crate::sketch::SketchModel;
use crate::{SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::Seed;
use sse_core::math as m;
use std::collections::BTreeSet;

pub const WEAR_VERSION: &str = "surface-wear-v1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WearSettings {
    pub epoch_myr: f64,
    pub plate_window_myr: f64,
    pub k_stream: f64,
    pub m: f64,
    pub glacial_factor: f64,
    /// A world wet only in the past rains at least this warm while wet, K.
    pub wet_floor_k: f64,
    pub diffusion_m2_yr: f64,
    pub river_area_cells: f64,
    /// Dunes where Lancaster's mobility (W9 `sse.wind.saltation`) is at least this.
    pub dune_activity: f64,
    pub dune_slope: f64,
    pub fill_epsilon_m: f64,
    /// Sediment reaching the sea builds out a shelf to this depth below sea
    /// level, then spills to deeper water (the mean shelf break, about 130 m;
    /// Kennett 1982, Marine Geology).
    #[serde(default = "shelf_depth")]
    pub shelf_depth_m: f64,
}

fn shelf_depth() -> f64 {
    130.0
}

/// When the world had surface water, and how much.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WaterHistory {
    /// Gyr since the world formed.
    pub wet_from_gyr: f64,
    pub wet_until_gyr: f64,
    /// Earth oceans while wet.
    pub oceans: f64,
    pub pressure_bar: f64,
}

impl WaterHistory {
    /// Wet for its whole life with today's water (the default reading of a body).
    pub fn steady(oceans: f64, pressure_bar: f64, age_gyr: f64) -> Self {
        WaterHistory {
            wet_from_gyr: 0.0,
            wet_until_gyr: if oceans > 0.0 { age_gyr } else { 0.0 },
            oceans,
            pressure_bar,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WearEpoch {
    pub t_myr: f64,
    pub wet: bool,
    /// Metres on the raw datum; None without surface water.
    pub sea_level_m: Option<f64>,
    pub eroded_m3: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Wear {
    pub n: u32,
    pub epoch_myr: f64,
    /// Final heights, metres from the datum (sea level when there is a sea).
    pub heights: Vec<f64>,
    pub sea_level_raw_m: Option<f64>,
    /// Downstream cell (itself at an outlet).
    pub receiver: Vec<u32>,
    pub area_m2: Vec<f64>,
    /// m³/yr in the last wet epoch.
    pub discharge: Vec<f64>,
    /// Length of the longest stream above the cell, metres.
    pub stream_m: Vec<f64>,
    pub river: Vec<bool>,
    pub lake: Vec<bool>,
    /// In a filled depression (a lake when wet, a dry basin otherwise).
    pub pit: Vec<bool>,
    pub incision_m: Vec<f64>,
    /// Sediment still lying where it was laid by the epoch, metres: rivers
    /// and creep strip it before they cut the rock beneath.
    pub sediment_m: Vec<f64>,
    pub glaciated: Vec<bool>,
    pub dunes: Vec<bool>,
    /// W9: each land cell's water in the last epoch (m of liquid a year):
    /// runoff, snow and the ice balance, and its dune activity (0 to 1).
    #[serde(default)]
    pub runoff_m_yr: Vec<f64>,
    #[serde(default)]
    pub snow_m_yr: Vec<f64>,
    #[serde(default)]
    pub ice_balance_m_yr: Vec<f64>,
    #[serde(default)]
    pub dune_activity: Vec<f64>,
    /// Each land cell's climate label (W9, the climate pack's area rules;
    /// empty under the sea). Nothing reads it back.
    #[serde(default)]
    pub areas: Vec<String>,
    pub eroded_m3: f64,
    pub deposited_m3: f64,
    pub epochs: Vec<WearEpoch>,
}

fn key(h: f64) -> u64 {
    let b = h.to_bits();
    if b >> 63 == 1 {
        !b
    } else {
        b | (1 << 63)
    }
}

struct Net {
    filled: Vec<f64>,
    receiver: Vec<usize>,
    /// Cells in ascending filled height (outlets first).
    order: Vec<usize>,
}

/// Priority-flood fill and steepest-descent receivers on the filled surface.
fn route(g: &Grid, nb: &[[usize; 4]], h: &[f64], sea: Option<f64>, eps: f64) -> Net {
    let nc = h.len();
    let mut filled = h.to_vec();
    let mut done = vec![false; nc];
    let mut open: BTreeSet<(u64, usize)> = BTreeSet::new();
    let mut outlet = vec![false; nc];
    if let Some(sl) = sea {
        for (o, x) in outlet.iter_mut().zip(h) {
            *o = *x < sl;
        }
    }
    if !outlet.iter().any(|x| *x) {
        // No sea: the lowest cell is the one closed basin's floor.
        let low = (0..nc)
            .min_by(|&a, &b| h[a].total_cmp(&h[b]).then(a.cmp(&b)))
            .expect("cells");
        outlet[low] = true;
    }
    for c in 0..nc {
        if outlet[c] {
            done[c] = true;
            open.insert((key(filled[c]), c));
        }
    }
    let mut order = Vec::with_capacity(nc);
    while let Some(&(k, c)) = open.iter().next() {
        open.remove(&(k, c));
        order.push(c);
        for &x in &nb[c] {
            if !done[x] {
                done[x] = true;
                filled[x] = filled[x].max(filled[c] + eps);
                open.insert((key(filled[x]), x));
            }
        }
    }
    let _ = g;
    let mut receiver: Vec<usize> = (0..nc).collect();
    for c in 0..nc {
        if outlet[c] {
            continue;
        }
        let mut best = c;
        let mut drop = 0.0;
        for &x in &nb[c] {
            let d = filled[c] - filled[x];
            if d > drop || (d == drop && d > 0.0 && x < best) {
                drop = d;
                best = x;
            }
        }
        receiver[c] = best;
    }
    // Ascending filled height, ties by flood order (which is consistent with it).
    let mut rank = vec![0usize; nc];
    for (i, c) in order.iter().enumerate() {
        rank[*c] = i;
    }
    let mut asc: Vec<usize> = (0..nc).collect();
    asc.sort_by(|&a, &b| filled[a].total_cmp(&filled[b]).then(rank[a].cmp(&rank[b])));
    Net {
        filled,
        receiver,
        order: asc,
    }
}

const RHO_ROCK: f64 = 2700.0;
const RHO_SEDIMENT: f64 = 2300.0;
const RHO_MANTLE: f64 = 3300.0;
const YOUNG_PA: f64 = 7.0e10;
const POISSON: f64 = 0.25;
/// The base of the elastic lithosphere sits near the 450 °C isotherm
/// (McNutt 1984, JGR 89, 11180; Watts 2001, Isostasy and Flexure of the
/// Lithosphere), at conductivity about 3 W/m/K.
const T_ELASTIC_K: f64 = 723.15;
const K_ROCK: f64 = 3.0;

/// How fully the lithosphere answers a load the size of the wearing grid's
/// features (two cells), 0 to 1: Airy isostasy filtered by flexure,
/// C = 1 / (1 + D k⁴ / (ρ_m g)), with D = E Te³ / 12(1 − ν²) and the
/// elastic thickness Te from today's heat flow (Turcotte and Schubert 2014,
/// Geodynamics, 3.17). Earth's continents come out about three quarters
/// compensated at that size; Mars's thick, cold lid barely at all.
pub fn compensation(env: &SurfaceEnvelope, cells: usize) -> f64 {
    let q = env
        .timeline
        .last()
        .map(|x| x.heat_flow_w_m2[1])
        .unwrap_or(0.0);
    if q <= 0.0 {
        return 0.0;
    }
    let te = (K_ROCK * (T_ELASTIC_K - env.t_surface_now_k).max(0.0) / q).max(1.0);
    let d = YOUNG_PA * te * te * te / (12.0 * (1.0 - POISSON * POISSON));
    let r = env.reference_radius_m.max(1.0);
    let cell = m::sqrt(4.0 * m::PI * r * r / cells.max(1) as f64);
    let k = 2.0 * m::PI / (2.0 * cell);
    let g = env.gravity_m_s2.max(1e-3);
    1.0 / (1.0 + d * k * k * k * k / (RHO_MANTLE * g))
}

/// What rivers bring to the sea builds a shelf: each mouth's load fills the
/// sea floor up to `top` (the shelf break below sea level), nearest cells
/// first through the sea, and spreads on until it is laid. Nothing is lost:
/// a load that fills its whole sea to the shelf stays at the mouth.
fn shelf_out(
    nb: &[[usize; 4]],
    h: &[f64],
    sea_level: f64,
    top: f64,
    area: f64,
    flux: &[f64],
    put: &mut [f64],
) {
    let nc = h.len();
    let mut mark = vec![usize::MAX; nc];
    for mouth in 0..nc {
        let mut rest = flux[mouth];
        if rest <= 0.0 || h[mouth] >= sea_level {
            continue;
        }
        // Breadth first through the sea from the mouth.
        let mut queue = std::collections::VecDeque::from([mouth]);
        mark[mouth] = mouth;
        while let Some(c) = queue.pop_front() {
            let room = ((top - h[c]) * area - put[c]).max(0.0);
            let d = rest.min(room);
            put[c] += d;
            rest -= d;
            if rest <= 0.0 {
                break;
            }
            for &x in &nb[c] {
                if mark[x] != mouth && h[x] + put[x] / area < sea_level {
                    mark[x] = mouth;
                    queue.push_back(x);
                }
            }
        }
        if rest > 0.0 {
            put[mouth] += rest;
        }
    }
}

/// Heights on a cell grid for wearing: the plate history's when its plates
/// moved, else the regime sketch sampled at the same cells (raw datum).
pub fn base_heights(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    plates: &PlateHistory,
) -> Vec<f64> {
    if plates.steps > 0 {
        crate::plates::cell_heights(s, plates, env.gravity_m_s2)
    } else {
        let regime = env
            .timeline
            .iter()
            .find(|x| {
                plates.epoch_myr / 1000.0 >= x.from_gyr && plates.epoch_myr / 1000.0 <= x.to_gyr
            })
            .map(|x| x.regime.clone())
            .unwrap_or_else(|| env.regime_now.clone());
        let model = SketchModel::new(s, env, &regime, seed);
        Grid::new(plates.n)
            .dirs
            .iter()
            .map(|d| model.at(*d).height_m)
            .collect()
    }
}

/// Wear a cell grid of heights from `from_myr` to `to_myr` (Myr since the
/// world formed), raining the wet epochs' climate.
#[allow(clippy::too_many_arguments)]
pub fn wear(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    n: u32,
    heights0: &[f64],
    water: &WaterHistory,
    climate: &WorldClimate,
    from_myr: f64,
    to_myr: f64,
    steady: bool,
) -> Wear {
    let w = &s.wear;
    let g = Grid::new(n);
    let nc = g.len();
    let r = env.reference_radius_m.max(1.0);
    let area = 4.0 * m::PI * r * r / nc as f64;
    let nb: Vec<[usize; 4]> = (0..nc).map(|c| g.neighbours(c)).collect();
    // Symmetric edge list for diffusion (exactly conserving).
    let mut edges: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (c, list) in nb.iter().enumerate() {
        for &x in list {
            if x != c {
                edges.insert((c.min(x), c.max(x)));
            }
        }
    }
    let dist = |a: usize, b: usize| {
        let (p, q) = (g.dirs[a], g.dirs[b]);
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        (m::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) * r).max(1.0)
    };
    let lat: Vec<f64> = g
        .dirs
        .iter()
        .map(|d| m::asin(d[2].clamp(-1.0, 1.0)))
        .collect();
    let lat_deg: Vec<f64> = lat.iter().map(|x| x * 180.0 / m::PI).collect();
    // W9: the one climate's water at each cell's latitude and height.
    let supply = |c: usize, above_m: f64| climate.water_at(&s.climate, lat_deg[c], above_m);
    let mut cell_water: Vec<sse_climate::water::WaterSupply> = vec![Default::default(); nc];
    let comp = compensation(env, g.len());
    let mut h = heights0.to_vec();
    let mut eroded_total = 0.0;
    let mut deposited = vec![0.0; nc];
    // Sediment still lying where it was laid: later erosion strips it first.
    let mut standing = vec![0.0; nc];
    let mut epochs = vec![];
    let mut last_q = vec![0.0; nc];
    let mut glaciated = vec![false; nc];
    let span = (to_myr - from_myr).max(0.0);
    let steps = m::ceil(span / w.epoch_myr.max(1.0)).max(0.0) as u32;
    let dt_myr = if steps > 0 { span / steps as f64 } else { 0.0 };
    let mut net = route(&g, &nb, &h, None, w.fill_epsilon_m);
    let mut sl_now = None;
    for k in 0..steps {
        let t = from_myr + dt_myr * (k as f64 + 0.5);
        let wet = t >= water.wet_from_gyr * 1000.0
            && t < water.wet_until_gyr * 1000.0
            && water.oceans > 0.0;
        let sl = if wet {
            sea_level(&h, r, water.oceans)
        } else {
            None
        };
        sl_now = sl;
        let start = h.clone();
        let mut laid = vec![0.0; nc];
        net = route(&g, &nb, &h, sl, w.fill_epsilon_m);
        let dt = dt_myr * 1.0e6;
        let mut eroded_epoch = 0.0;
        if wet {
            let datum = sl.unwrap_or(0.0);
            for c in 0..nc {
                cell_water[c] = supply(c, h[c] - datum);
            }
            // Discharge from each land cell's runoff, accumulated upstream to
            // downstream (the sea's own rain does not run off).
            let mut q: Vec<f64> = (0..nc)
                .map(|c| {
                    if sl.is_some_and(|x| h[c] < x) {
                        0.0
                    } else {
                        cell_water[c].runoff_m_yr * area
                    }
                })
                .collect();
            for &c in net.order.iter().rev() {
                let rc = net.receiver[c];
                if rc != c {
                    q[rc] += q[c];
                }
            }
            for c in 0..nc {
                glaciated[c] = cell_water[c].ice_balance_m_yr > 0.0 && sl.is_none_or(|x| h[c] >= x);
            }
            // Implicit stream power, outlets first.
            let old = h.clone();
            for &c in &net.order {
                let rc = net.receiver[c];
                if rc == c || sl.is_some_and(|x| h[c] < x) {
                    continue;
                }
                let kf = w.k_stream * if glaciated[c] { w.glacial_factor } else { 1.0 };
                // Q / (1 m/yr) is the drainage area a metre of rain a year would fill.
                let f = kf * m::pow(q[c], w.m) * dt / dist(c, rc);
                let new = (h[c] + f * h[rc]) / (1.0 + f);
                if new < h[c] {
                    h[c] = new;
                }
            }
            // Carry what was worn off downstream; lay it in pits and at the sea.
            let mut flux = vec![0.0; nc];
            for c in 0..nc {
                let e = (old[c] - h[c]).max(0.0) * area;
                flux[c] += e;
                eroded_epoch += e;
            }
            let mut put = vec![0.0; nc];
            for &c in net.order.iter().rev() {
                let rc = net.receiver[c];
                let in_sea = sl.is_some_and(|x| old[c] < x);
                // Room is the depression as it stood before this epoch's
                // cutting: a cell's own incision is not a hole to refill.
                if in_sea {
                    // Laid down in the sea below (shelf_out, after the loop).
                    continue;
                }
                let room = if rc == c {
                    f64::INFINITY
                } else {
                    ((net.filled[c] - h[c]) * area).max(0.0)
                };
                let d = flux[c].min(room);
                put[c] += d;
                let rest = flux[c] - d;
                if rc != c && rest > 0.0 {
                    flux[rc] += rest;
                }
            }
            if let Some(x) = sl {
                shelf_out(&nb, &old, x, x - w.shelf_depth_m, area, &flux, &mut put);
            }
            // Net worn off each cell: its cut less what was laid back in it.
            let off: Vec<f64> = (0..nc)
                .map(|c| (old[c] - h[c] - put[c] / area).max(0.0))
                .collect();
            for c in 0..nc {
                // Sediment left lying: what was laid less what was cut, the
                // old cover going first.
                let cut = (old[c] - h[c]).max(0.0);
                standing[c] = (standing[c] + put[c] / area - cut).max(0.0);
            }
            if !steady {
                // The crust answers its unloading (isostasy, filtered by
                // flexure): a rebound for what was worn off.
                for c in 0..nc {
                    h[c] += comp * RHO_ROCK / RHO_MANTLE * off[c];
                }
            }
            for c in 0..nc {
                // What this epoch left lying here, net of its own cut.
                laid[c] = (put[c] / area - (old[c] - h[c]).max(0.0)).max(0.0)
                    * (1.0 - comp * RHO_SEDIMENT / RHO_MANTLE);
                h[c] += put[c] / area * (1.0 - comp * RHO_SEDIMENT / RHO_MANTLE);
                deposited[c] += put[c];
            }
            last_q = q;
        }
        // Slopes relax, wet or dry (effective coarse-grid diffusivity).
        let kappa = w.diffusion_m2_yr;
        let lmin = edges
            .iter()
            .map(|&(a, b)| dist(a, b))
            .fold(f64::INFINITY, f64::min);
        let sub = m::ceil(dt * kappa / (lmin * lmin) / 0.2).max(1.0) as u32;
        let ds = dt / sub as f64;
        for _ in 0..sub {
            let mut dh = vec![0.0; nc];
            for &(a, b) in &edges {
                let l = dist(a, b);
                let f = ds * kappa * (h[b] - h[a]) / (l * l);
                dh[a] += f;
                dh[b] -= f;
            }
            for c in 0..nc {
                h[c] += dh[c];
                // Creep strips the cover where slopes lower and lays it
                // where they fill.
                standing[c] = (standing[c] + dh[c]).max(0.0);
            }
        }
        if steady {
            // On a world whose plates move, the plate history's heights are
            // already the balance of uplift and wear (S2 wears its belts
            // down), so the land is raised as fast as it is worn here
            // (steady state, Whipple and Tucker 1999): rivers carry the load
            // and lay it down, but the continents keep their height and
            // shape, and rise only by what is laid on them (creep included:
            // the plate heights already hold it).
            for c in 0..nc {
                if sl.is_none_or(|x| start[c] >= x) {
                    h[c] = start[c] + laid[c];
                }
            }
        }
        eroded_total += eroded_epoch;
        epochs.push(WearEpoch {
            t_myr: from_myr + dt_myr * (k as f64 + 1.0),
            wet,
            sea_level_m: sl,
            eroded_m3: eroded_epoch,
        });
    }
    if steps == 0 {
        sl_now = if water.oceans > 0.0 {
            sea_level(&h, r, water.oceans)
        } else {
            None
        };
        net = route(&g, &nb, &h, sl_now, w.fill_epsilon_m);
    }
    // Final drainage on the worn surface.
    let final_net = route(&g, &nb, &h, sl_now, w.fill_epsilon_m);
    let mut a = vec![area; nc];
    let mut stream = vec![0.0; nc];
    for &c in final_net.order.iter().rev() {
        let rc = final_net.receiver[c];
        if rc != c {
            a[rc] += a[c];
            stream[rc] = f64::max(stream[rc], stream[c] + dist(c, rc));
        }
    }
    let _ = net;
    let sea = sl_now;
    let land = |c: usize| sea.is_none_or(|x| h[c] >= x);
    let dry_now = sea.is_none();
    let datum = sea.unwrap_or(0.0);
    // W9: the water and the wind on the final surface. A world dry now reads
    // today's climate (no liquid: no water at all).
    let water_now: Vec<sse_climate::water::WaterSupply> = (0..nc)
        .map(|c| {
            if land(c) && !dry_now {
                supply(c, h[c] - datum)
            } else if land(c) {
                // Dry now: today's temperatures, no water.
                let x = supply(c, h[c] - datum);
                sse_climate::water::WaterSupply {
                    t_k: x.t_k,
                    t_cold_k: x.t_cold_k,
                    t_warm_k: x.t_warm_k,
                    ..Default::default()
                }
            } else {
                Default::default()
            }
        })
        .collect();
    let activity: Vec<f64> = (0..nc)
        .map(|c| {
            if !land(c) || glaciated[c] {
                return 0.0;
            }
            let ss = &s.climate.saltation;
            let t = water_now[c].t_k;
            let sa = sse_climate::saltation::at(
                ss,
                &climate.climate,
                lat_deg[c],
                sse_climate::saltation::grain_for(ss, t),
            );
            sse_climate::saltation::dune_activity(ss, sa.transport_share, water_now[c].aridity)
        })
        .collect();
    let _ = &cell_water;
    let slope = |c: usize| {
        let rc = final_net.receiver[c];
        if rc == c {
            0.0
        } else {
            (h[c] - h[rc]).abs() / dist(c, rc)
        }
    };
    Wear {
        n,
        epoch_myr: to_myr,
        heights: h.iter().map(|x| x - datum).collect(),
        sea_level_raw_m: sea,
        receiver: final_net.receiver.iter().map(|x| *x as u32).collect(),
        river: (0..nc)
            .map(|c| land(c) && !dry_now && a[c] >= w.river_area_cells * area)
            .collect(),
        lake: (0..nc)
            .map(|c| land(c) && !dry_now && final_net.filled[c] - h[c] > 2.0 * w.fill_epsilon_m)
            .collect(),
        pit: (0..nc)
            .map(|c| final_net.filled[c] - h[c] > 2.0 * w.fill_epsilon_m)
            .collect(),
        incision_m: (0..nc).map(|c| (heights0[c] - h[c]).max(0.0)).collect(),
        sediment_m: standing,
        dunes: (0..nc)
            .map(|c| activity[c] >= w.dune_activity && slope(c) < w.dune_slope)
            .collect(),
        runoff_m_yr: water_now.iter().map(|x| x.runoff_m_yr).collect(),
        snow_m_yr: water_now.iter().map(|x| x.snow_m_yr).collect(),
        ice_balance_m_yr: water_now.iter().map(|x| x.ice_balance_m_yr).collect(),
        dune_activity: activity,
        areas: (0..nc)
            .map(|c| {
                if land(c) {
                    sse_climate::water::area(&s.climate, &water_now[c])
                        .0
                        .to_string()
                } else {
                    String::new()
                }
            })
            .collect(),
        glaciated,
        area_m2: a,
        discharge: last_q,
        stream_m: stream,
        eroded_m3: eroded_total,
        deposited_m3: deposited.iter().sum(),
        epochs,
    }
}

/// Wear a world up to its plate history's epoch, with a water history.
pub fn wear_world(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    plates: &PlateHistory,
    water: &WaterHistory,
    climate: &WorldClimate,
    magma_end_myr: f64,
) -> Wear {
    let h0 = base_heights(s, env, seed, plates);
    let to = plates.epoch_myr;
    let from = if plates.steps > 0 {
        (to - s.wear.plate_window_myr).max(0.0)
    } else {
        magma_end_myr.min(to)
    };
    wear(
        s,
        env,
        plates.n,
        &h0,
        water,
        climate,
        from,
        to,
        plates.steps > 0,
    )
}
