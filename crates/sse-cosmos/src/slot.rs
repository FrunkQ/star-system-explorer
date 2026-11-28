//! GZ13 (grand zoom design 6.3 point 2): every system has a star slot in a
//! cell. A system grown from a drawn star has one already. For an authored
//! or imported system the kernel finds one when it is adopted: with a map
//! position (a V3 starmap), the cell that far from the Sun's place; with
//! none, the drawn star nearest the Sun's place that fits the system's own
//! star (mass, age and number of companions). The search is pure: the same
//! seed, settings and pins give the same slot on every device.
//!
//! The settings are plain data (`provides.other.cosmology.slot`).
//!
//! Shortcut (fudge register F37c): "fits" is a tolerance on three numbers,
//! not a likelihood over the cell's population, and a map position is laid
//! on the galaxy's grid axes as given (V3 maps have no galactic frame).

use crate::stars::{StarSystem, Stars};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// How a slot is looked for (`provides.other.cosmology.slot`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SlotParams {
    /// A drawn star fits when its mass is within this fraction of the
    /// system's star's.
    pub mass_tol: f64,
    /// ... and its age within this fraction of the system's age, or within
    /// `age_tol_gyr_min`, whichever is wider.
    pub age_tol: f64,
    pub age_tol_gyr_min: f64,
    /// ... and it has the same number of companions.
    pub same_companions: bool,
    /// Shells of 10 pc cells searched round the starting cell (0 is the cell
    /// alone, 1 the 27 round it, 3 the 343).
    pub search_shells: u32,
}

impl Default for SlotParams {
    fn default() -> Self {
        SlotParams {
            mass_tol: 0.1,
            age_tol: 0.1,
            age_tol_gyr_min: 0.5,
            same_companions: true,
            search_shells: 3,
        }
    }
}

/// The system's own star, as authored (each part may be missing).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Wanted {
    pub mass_msun: Option<f64>,
    pub age_gyr: Option<f64>,
    pub companions: Option<usize>,
}

/// The slot found.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Found {
    /// The drawn star, as today's rules draw it (not yet pinned).
    pub star: StarSystem,
    /// It meets every tolerance.
    pub fits: bool,
    /// "map" (by the map position) or "fit" (nearest fitting star).
    pub by: String,
    pub cells_read: usize,
    /// From the starting point (the Sun's place, or the map position), pc.
    pub distance_pc: f64,
    /// The cell searching started from.
    pub start_cell: [i64; 3],
    pub why: Vec<String>,
}

/// Each term is 1 at its tolerance, so a star fits when all are 1 or less.
fn terms(s: &StarSystem, w: &Wanted, p: &SlotParams, now_gyr: f64) -> [f64; 3] {
    let mass = w.mass_msun.map_or(0.0, |want| {
        m::ln(s.mass_msun / want).abs() / m::ln(1.0 + p.mass_tol)
    });
    let age = w.age_gyr.map_or(0.0, |want| {
        let tol = (p.age_tol * want).max(p.age_tol_gyr_min);
        (now_gyr - s.birth_gyr - want).abs() / tol
    });
    let comp = match (w.companions, p.same_companions) {
        (Some(n), true) if s.companions.len() != n => 2.0,
        _ => 0.0,
    };
    [mass, age, comp]
}

fn score(t: &[f64; 3]) -> f64 {
    t.iter().map(|x| x * x).sum()
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (x, y, z) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    m::sqrt(x * x + y * y + z * z)
}

/// The galactic position of the Sun's place at a time: the Sun's radius on
/// the grid's x axis (as `star_cell`'s default cell).
pub fn suns_place(st: &Stars, t_myr: f64) -> [f64; 3] {
    st.from_grid([st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0], t_myr)
}

/// Find a slot. `map_pc` is the map position's offset from the Sun's
/// place in pc (grid axes), when the system has one. `taken` are stars
/// already pinned (another system's slot or a looked-at star): never
/// chosen. With a map position the best match in the nearest shell that
/// holds any free star is taken, fit or not; with none, the best match in
/// the nearest shell holding a fitting star, or with no fit anywhere
/// searched the best match overall, marked.
pub fn find(
    st: &Stars,
    t_myr: f64,
    map_pc: Option<[f64; 3]>,
    pins: &[StarSystem],
    w: &Wanted,
    p: &SlotParams,
) -> Result<Found, String> {
    let now_gyr = t_myr / 1000.0;
    let sun = suns_place(st, t_myr);
    let grid_sun = [st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0];
    let start = match map_pc {
        Some(d) => st.from_grid(
            [grid_sun[0] + d[0], grid_sun[1] + d[1], grid_sun[2] + d[2]],
            t_myr,
        ),
        None => sun,
    };
    let c0 = st.cell_at(start, t_myr);
    let frame = st.frame_of(t_myr);
    let mut best: Option<(bool, f64, f64, StarSystem)> = None;
    let mut read = 0;
    for d in 0..=i64::from(p.search_shells) {
        let cells: Vec<[i64; 3]> = crate::home::shell(d)
            .into_iter()
            .map(|o| [c0[0] + o[0], c0[1] + o[1], c0[2] + o[2]])
            .collect();
        read += cells.len();
        for list in st.cells(&cells, frame, pins)? {
            for s in list {
                if s.pinned || pins.iter().any(|q| q.address == s.address) {
                    continue;
                }
                let t = terms(&s, w, p, now_gyr);
                let fits = t.iter().all(|x| *x <= 1.0);
                let key = (fits, score(&t), dist(st.at(&s, t_myr).xyz_pc, start));
                let better = match &best {
                    None => true,
                    Some((f, sc, dd, b)) => {
                        (key.0 && !f)
                            || (key.0 == *f
                                && (key.1 < *sc
                                    || (key.1 == *sc
                                        && (key.2 < *dd
                                            || (key.2 == *dd && s.address < b.address)))))
                    }
                };
                if better {
                    best = Some((key.0, key.1, key.2, s));
                }
            }
        }
        match &best {
            Some(_) if map_pc.is_some() => break,
            Some((true, ..)) => break,
            _ => {}
        }
    }
    let (fits, _, distance_pc, star) = best.ok_or_else(|| {
        format!(
            "no free star within {} shells of cell {c0:?}",
            p.search_shells
        )
    })?;
    let age = now_gyr - star.birth_gyr;
    let mut why = vec![format!(
        "drawn star of {:.2} suns, {:.2} Gyr old, {} companion(s), {:.0} pc from the {}",
        star.mass_msun,
        age,
        star.companions.len(),
        distance_pc,
        if map_pc.is_some() {
            "map position"
        } else {
            "Sun's place"
        }
    )];
    if !fits {
        why.push("no drawn star nearby meets every tolerance: the closest match is taken".into());
    }
    Ok(Found {
        star,
        fits,
        by: if map_pc.is_some() { "map" } else { "fit" }.into(),
        cells_read: read,
        distance_pc,
        start_cell: c0,
        why,
    })
}
