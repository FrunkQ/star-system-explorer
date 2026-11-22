//! Time-lapse frames: a world's surface from the start of its plate history
//! to the epoch, as per-cell keyframes on the S2 plate grid, with the dated
//! events between them. For the Body globe's generation time-lapse.
//!
//! Every frame reads the same world the survey reads: the same plate run
//! (recorded, never re-drawn), the same impacts and volcanism, and the one
//! climate evaluated at each frame's own date (the host star's light then,
//! the land then). Nothing here is drawn afresh.
//!
//! - **elevation_m**: the plate grid's tectonic heights at the date (S2's
//!   ocean-floor ages, orogens, arcs, trenches and plumes), from that date's
//!   sea level. S3 wears only the last window, so earlier frames are unworn.
//! - **vent**, **arc**, **ridge**, 0 to 1 each: how recently the cell
//!   erupted (plumes, floods, and the shields and lava plains active then),
//!   stood on a volcanic arc over a sinking plate, or was made as new ocean
//!   crust at a ridge, decaying over the time between frames.
//! - **temp_k**, **rain_m_yr**, **runoff_m_yr**: the climate at the date,
//!   read at each cell's latitude and height (W7a, W9).
//! - **ice_m**: land whose ice balance is above zero holds an ice sheet,
//!   its thickness the perfectly plastic profile H = √(2 τ₀ x / (ρ g)) with
//!   x the distance in from the ice edge (Nye 1951; τ₀ = 100 kPa).

use crate::climate::world_climate;
use crate::impacts::{impacts, ImpactHistory, ImpactInputs};
use crate::plates::{cells_heights, grid_for, history_recorded, sea_level, Cell, Grid};
use crate::survey::SurveyOptions;
use crate::wear::{base_heights, WaterHistory};
use crate::{envelope, Body, Climate, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::Seed;
use sse_core::math as m;

/// A frame's plates, crust kinds, raw heights and cells.
type FrameState<'a> = (Vec<u16>, Vec<bool>, Vec<f64>, Option<&'a [Cell]>);

pub const FRAMES_VERSION: &str = "surface-frames-v1";

/// Yield stress of ice in a perfectly plastic sheet, Pa (Nye 1951).
const ICE_YIELD_PA: f64 = 1.0e5;
const RHO_ICE: f64 = 917.0;
/// Frames for a world whose plates never moved.
const STILL_FRAMES: u32 = 12;
/// Impacts listed as events: the largest this many.
const IMPACT_EVENTS: usize = 200;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FrameGrid {
    pub n: u32,
    pub cells: u32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub t_myr: f64,
    /// Sea level above the raw datum, m (None on a dry world); every
    /// elevation is from it.
    pub sea_level_m: Option<f64>,
    pub plate: Vec<u16>,
    pub continental: Vec<bool>,
    pub elevation_m: Vec<f32>,
    /// Plume, flood, shield and plains eruptions, 0 to 1.
    pub vent: Vec<f32>,
    /// Arc volcanism over a sinking plate, 0 to 1.
    pub arc: Vec<f32>,
    /// New ocean crust at a spreading ridge, 0 to 1.
    pub ridge: Vec<f32>,
    pub ice_m: Vec<f32>,
    pub temp_k: Vec<f32>,
    pub rain_m_yr: Vec<f32>,
    pub runoff_m_yr: Vec<f32>,
    pub why: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FrameEvent {
    pub t_myr: f64,
    pub kind: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plates: Vec<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius_km: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diameter_km: Option<f64>,
    pub why: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Frames {
    pub version: String,
    pub detail: String,
    pub epoch_myr: f64,
    pub grid: FrameGrid,
    /// Cell centres, unit vectors, x y z per cell.
    pub dirs: Vec<f32>,
    pub frames: Vec<Frame>,
    pub events: Vec<FrameEvent>,
    /// Fields and events not yet dated, held at their final value or absent.
    pub pending: Vec<String>,
    pub why: Vec<String>,
}

fn f32s(v: impl Iterator<Item = f64>) -> Vec<f32> {
    v.map(|x| x as f32).collect()
}

/// How recently a cell erupted, 0 to 1 each, decaying over `span` Myr:
/// (plume and flood eruptions, arc volcanism over a trench, new ocean crust
/// at a ridge).
fn vent_of(c: &Cell, t: f64, span: f64) -> (f64, f64, f64) {
    let k = |date: f64| {
        if date > t + 1e-9 {
            0.0
        } else {
            m::exp(-(t - date) / span)
        }
    };
    let mut v: f64 = 0.0;
    for (d, _) in &c.volcanic {
        v = v.max(k(*d));
    }
    if let Some(f) = c.flood_myr {
        v = v.max(k(f));
    }
    let arc = c.arc_myr.map(k).unwrap_or(0.0);
    let ridge = if c.continental { 0.0 } else { k(c.born_myr) };
    (v, arc, ridge)
}

fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    m::acos((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0))
}

/// Ice sheet thickness over glaciated cells (Nye's plastic profile), m.
fn ice_sheet(grid: &Grid, glaciated: &[bool], cell_m: f64, g: f64) -> Vec<f64> {
    let n = glaciated.len();
    // Steps in from the nearest unglaciated cell, breadth first.
    let mut dist = vec![u32::MAX; n];
    let mut queue = std::collections::VecDeque::new();
    for i in 0..n {
        if !glaciated[i] {
            dist[i] = 0;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        for j in grid.neighbours(i) {
            if dist[j] == u32::MAX {
                dist[j] = dist[i] + 1;
                queue.push_back(j);
            }
        }
    }
    (0..n)
        .map(|i| {
            if !glaciated[i] {
                return 0.0;
            }
            // A world all ice: the sheet is as thick as half a hemisphere
            // allows (dist stays unset).
            let steps = if dist[i] == u32::MAX {
                grid.n as f64
            } else {
                dist[i] as f64 - 0.5
            };
            m::sqrt(2.0 * ICE_YIELD_PA * steps * cell_m / (RHO_ICE * g.max(1e-3)))
        })
        .collect()
}

/// The time-lapse frames of `body` up to `epoch_gyr`. `survey` picks the
/// fine plate grid (Survey level); otherwise the coarse Orbit grid.
pub fn surface_frames(
    s: &SurfaceSettings,
    body: &Body,
    seed: &Seed,
    epoch_gyr: f64,
    opts: &SurveyOptions,
    survey: bool,
    frames_max: Option<usize>,
) -> Result<Frames, String> {
    let s = if survey {
        s.clone()
    } else {
        crate::detail::orbit_settings(s)
    };
    let e = envelope(&s, body);
    if e.kind == "cloud-deck" {
        return Err(format!("{} has no ground to show", body.label));
    }
    let h = history_recorded(&s, &e, seed, epoch_gyr);
    let grid = grid_for(&h);
    let nc = grid.len();
    let r = e.reference_radius_m;
    let g = e.gravity_m_s2;
    let cell_m = m::sqrt(4.0 * m::PI * r * r / nc as f64);
    let epoch_myr = h.epoch_myr;
    let mut why = vec![];
    let mut pending = vec![
        "ice-age onsets (W9 part 2 dates them only inside the orbit-history window)".to_string(),
        "worn elevation (S3 wears only the last window; frames show tectonic heights)".to_string(),
        "oceans over time (held at today's volume)".to_string(),
        "sea ice".to_string(),
    ];
    if matches!(body.climate, Climate::Fixed { .. }) {
        pending.push("temperature history (this world's climate is a fixed value)".into());
    }
    // Frame dates and the cells at each: the plate snapshots, or evenly
    // spaced dates for a world whose plates never moved.
    let still = h.steps == 0 || h.snapshots.iter().all(|x| x.cells.is_empty());
    let mut dates: Vec<(f64, Option<usize>)> = if still {
        why.push("Its plates never moved: the frames show its climate and volcanism over time on one fixed surface.".into());
        (0..=STILL_FRAMES)
            .map(|k| (epoch_myr * k as f64 / STILL_FRAMES as f64, None))
            .collect()
    } else {
        h.snapshots
            .iter()
            .enumerate()
            .map(|(i, x)| (x.t_myr, Some(i)))
            .collect()
    };
    if dates.is_empty() {
        dates.push((epoch_myr, None));
    }
    // At most `frames_max`, evenly spaced, always ending on the epoch.
    if let Some(mx) = frames_max.filter(|mx| *mx < dates.len()) {
        let n = dates.len();
        dates = (0..mx.max(1))
            .map(|i| {
                let k = if mx <= 1 {
                    n - 1
                } else {
                    (i * (n - 1) + (mx - 1) / 2) / (mx - 1)
                };
                dates[k]
            })
            .collect();
    }
    let still_heights = still.then(|| base_heights(&s, &e, seed, &h));
    // Volcanism and impacts, dated (S4), on the same run.
    let water = WaterHistory::steady(body.oceans, body.pressure_bar, epoch_gyr);
    let hist = ImpactHistory::lunar();
    let ii = ImpactInputs {
        history: &hist,
        water: &water,
        pressure_bar: body.pressure_bar,
        plates: Some(&h),
    };
    let (feat, _) = impacts(&s, &e, seed, &ii, epoch_gyr)?;
    let x = opts.extra(opts.rotation_hours);
    let mut frames = vec![];
    let mut prev_t = dates.first().map(|d| d.0).unwrap_or(0.0);
    for (k, (t, snap)) in dates.iter().enumerate() {
        let t = *t;
        let span = if k == 0 {
            (dates.get(1).map(|d| d.0).unwrap_or(t) - t).max(1.0)
        } else {
            (t - prev_t).max(1.0)
        };
        prev_t = t;
        let (plate, continental, raw, cells): FrameState = match snap.map(|i| &h.snapshots[i]) {
            Some(sn) => (
                sn.plate.clone(),
                sn.continental.clone(),
                cells_heights(&s.plates, &sn.cells, t, g),
                Some(&sn.cells[..]),
            ),
            None => (
                vec![0; nc],
                vec![false; nc],
                still_heights.clone().unwrap_or_else(|| vec![0.0; nc]),
                None,
            ),
        };
        let sl = sea_level(&raw, r, body.oceans);
        let elev: Vec<f64> = raw.iter().map(|z| z - sl.unwrap_or(0.0)).collect();
        // Vents: the cell's own eruptions, then the shields and plains
        // erupting at the date.
        let (mut vent, arc, ridge): (Vec<f64>, Vec<f64>, Vec<f64>) = match cells {
            Some(cs) => {
                let v: Vec<(f64, f64, f64)> = cs.iter().map(|c| vent_of(c, t, span)).collect();
                (
                    v.iter().map(|x| x.0).collect(),
                    v.iter().map(|x| x.1).collect(),
                    v.iter().map(|x| x.2).collect(),
                )
            }
            None => (vec![0.0; nc], vec![0.0; nc], vec![0.0; nc]),
        };
        let mut lit = |dir: [f64; 3], radius_km: f64, v: f64| {
            let a = radius_km * 1000.0 / r;
            for (i, d) in grid.dirs.iter().enumerate() {
                if angle(*d, dir) <= a {
                    vent[i] = vent[i].max(v);
                }
            }
        };
        for sh in &feat.shields {
            if sh.from_myr <= t && t <= sh.to_myr + span {
                lit(sh.dir, sh.radius_km, 1.0);
            }
        }
        for p in &feat.plains {
            if p.formed_myr <= t && t - p.formed_myr <= span {
                lit(p.dir, p.radius_km, m::exp(-(t - p.formed_myr) / span));
            }
        }
        // The climate at the date: the host's light then (the world's age
        // set to the date), today's air and water.
        let mut b_t = body.clone();
        b_t.age_gyr = (t / 1000.0).max(1e-3);
        let e_t = envelope(&s, &b_t);
        let mut e_c = e.clone();
        e_c.t_surface_now_k = e_t.t_surface_now_k;
        let wc = world_climate(&s, &b_t, &e_c, &x, t);
        let mut temp = Vec::with_capacity(nc);
        let mut rain = Vec::with_capacity(nc);
        let mut runoff = Vec::with_capacity(nc);
        let mut glaciated = Vec::with_capacity(nc);
        for (i, d) in grid.dirs.iter().enumerate() {
            let lat = m::asin(d[2].clamp(-1.0, 1.0)).to_degrees();
            let land = sl.is_none() || elev[i] > 0.0;
            let w = wc.water_at(&s.climate, lat, elev[i].max(0.0));
            temp.push(w.t_k);
            rain.push(w.precip_m_yr);
            runoff.push(if land { w.runoff_m_yr } else { 0.0 });
            glaciated.push(land && w.ice_balance_m_yr > 0.0);
        }
        let ice = ice_sheet(&grid, &glaciated, cell_m, g);
        let land_pc =
            100.0 * elev.iter().filter(|z| sl.is_none() || **z > 0.0).count() as f64 / nc as f64;
        let ice_pc = 100.0 * glaciated.iter().filter(|x| **x).count() as f64 / nc as f64;
        frames.push(Frame {
            t_myr: t,
            sea_level_m: sl,
            plate,
            continental,
            elevation_m: f32s(elev.into_iter()),
            vent: f32s(vent.into_iter()),
            arc: f32s(arc.into_iter()),
            ridge: f32s(ridge.into_iter()),
            ice_m: f32s(ice.into_iter()),
            temp_k: f32s(temp.into_iter()),
            rain_m_yr: f32s(rain.into_iter()),
            runoff_m_yr: f32s(runoff.into_iter()),
            why: vec![format!(
                "{:.2} Gyr: {land_pc:.0}% land, {ice_pc:.0}% under ice, mean surface {:.0} K.",
                t / 1000.0,
                e_t.t_surface_now_k
            )],
        });
    }
    // Events, dated.
    let mut events: Vec<FrameEvent> = h
        .events
        .iter()
        .map(|ev| FrameEvent {
            t_myr: ev.t_myr,
            kind: ev.kind.clone(),
            text: ev.text.clone(),
            dir: None,
            plates: ev.plates.clone(),
            radius_km: None,
            diameter_km: None,
            why: vec![],
        })
        .collect();
    for sh in &feat.shields {
        events.push(FrameEvent {
            t_myr: sh.from_myr,
            kind: "eruption".into(),
            text: format!(
                "A shield volcano starts to build, {:.0} km across; it erupts until {:.2} Gyr.",
                2.0 * sh.radius_km,
                sh.to_myr / 1000.0
            ),
            dir: Some(sh.dir),
            plates: vec![],
            radius_km: Some(sh.radius_km),
            diameter_km: None,
            why: vec![sh.cause.clone()],
        });
    }
    for p in &feat.plains {
        events.push(FrameEvent {
            t_myr: p.formed_myr,
            kind: "flood-lava".into(),
            text: format!("Lava floods a plain {:.0} km across.", 2.0 * p.radius_km),
            dir: Some(p.dir),
            plates: vec![],
            radius_km: Some(p.radius_km),
            diameter_km: None,
            why: vec![],
        });
    }
    let mut craters: Vec<&crate::impacts::Crater> = feat.craters.iter().collect();
    craters.sort_by(|a, b| b.d_km.total_cmp(&a.d_km).then(a.id.cmp(&b.id)));
    for c in craters.into_iter().take(IMPACT_EVENTS) {
        events.push(FrameEvent {
            t_myr: c.formed_myr,
            kind: "impact".into(),
            text: format!(
                "An impact digs a {} crater {:.0} km across.",
                c.shape, c.d_km
            ),
            dir: Some(c.dir),
            plates: vec![],
            radius_km: None,
            diameter_km: Some(c.d_km),
            why: vec![],
        });
    }
    events.sort_by(|a, b| {
        a.t_myr
            .total_cmp(&b.t_myr)
            .then(a.kind.cmp(&b.kind))
            .then(a.text.cmp(&b.text))
    });
    why.push(format!(
        "{} frames from {:.2} to {:.2} Gyr on the {} plate grid ({} cells a face), {} dated events.",
        frames.len(),
        frames.first().map(|f| f.t_myr).unwrap_or(0.0) / 1000.0,
        epoch_myr / 1000.0,
        if survey { "Survey" } else { "Orbit" },
        grid.n,
        events.len()
    ));
    Ok(Frames {
        version: FRAMES_VERSION.into(),
        detail: if survey { "survey" } else { "orbit" }.into(),
        epoch_myr,
        grid: FrameGrid {
            n: grid.n,
            cells: nc as u32,
        },
        dirs: grid
            .dirs
            .iter()
            .flat_map(|d| d.iter().map(|x| *x as f32))
            .collect(),
        frames,
        events,
        pending,
        why,
    })
}
