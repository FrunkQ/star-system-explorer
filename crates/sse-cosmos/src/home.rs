//! GZ12 (grand zoom design 6.3 point 1): home is one of the web's own
//! galaxies. When a universe is opened the kernel searches the halo cells
//! outward from the origin for the catalogued halo that best fits the
//! pack's home wish, and every op that used to ask for the median
//! "milky-way-like" halo reads that one instead. The search is pure: the
//! same seed and settings give the same home on every device, so it is
//! worked out once and kept, never stored as a second source of truth.
//!
//! The wish is plain data (`provides.other.cosmology.home`). By default it
//! is "Like the Milky Way", the wish the Find a home screen already uses
//! and cites: a halo of 0.5 to 2 x 10^12 solar masses, no major merger in
//! the last 8 Gyr, and a bound big neighbour within 1 Mpc.
//!
//! Shortcut (fudge register, F37's successor): the best fit among a few
//! dozen cells, not a constrained realisation of the real Local Group.

use crate::haloes::{Halo, Haloes};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// What the universe's home should be like (`provides.other.cosmology.home`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeWish {
    /// Dark-matter halo mass today, solar masses.
    pub mass_min: f64,
    pub mass_max: f64,
    /// No major merger for at least this long before now, Gyr; 0 asks nothing.
    pub quiet_gyr: f64,
    /// "bound" (a bigger neighbour bound to it), "alone" (none within
    /// `neighbour_mpc`) or "any".
    pub neighbour: String,
    pub neighbour_mpc: f64,
    /// With "bound": the neighbour at most this many times our halo's mass
    /// (a partner like Andromeda, not a group we are a satellite of); 0
    /// asks nothing.
    pub neighbour_mass_ratio_max: f64,
    /// The disc's scale length today, kpc, from the halo's own spin and
    /// size (the G3 disc rule); 0 for either asks nothing.
    pub disc_kpc_min: f64,
    pub disc_kpc_max: f64,
    /// How many shells of cells round the origin cell are searched (0 is
    /// the origin cell alone, 1 the 27 cells round it, 2 the 125).
    pub search_shells: u32,
}

impl Default for HomeWish {
    fn default() -> Self {
        HomeWish {
            mass_min: 5.0e11,
            mass_max: 2.0e12,
            quiet_gyr: 8.0,
            neighbour: "bound".to_string(),
            neighbour_mpc: 1.0,
            neighbour_mass_ratio_max: 3.0,
            disc_kpc_min: 0.0,
            disc_kpc_max: 0.0,
            search_shells: 2,
        }
    }
}

/// The home the search found, and why.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HomeChoice {
    pub halo: Halo,
    /// The halo cell it was drawn in.
    pub cell: [i64; 3],
    /// True when it meets every wish; false when no halo in the searched
    /// cells did and the one nearest the wished mass was taken instead.
    pub fits_wish: bool,
    /// One line per wish, in words.
    pub why: Vec<String>,
    /// Cells read before the search stopped.
    pub cells_read: usize,
}

/// The last major merger's time (Gyr), if any.
pub fn last_major_gyr(h: &Halo, major_ratio: f64) -> Option<f64> {
    h.mergers
        .iter()
        .filter(|g| g.ratio >= major_ratio)
        .map(|g| g.t_gyr)
        .fold(None, |a: Option<f64>, t| Some(a.map_or(t, |x| x.max(t))))
}

/// Whether one halo fits, in words; `None` when it does not. The same rules
/// as the Find a home screen (`ui-contract/home-finder.ts`, `homeFits`).
/// The disc's scale length (kpc) is given by `disc_kpc` (the G3 rule,
/// `disc_kpc_of`); it is only said here, and ranks the fits in `choose`.
pub fn fits(
    h: &Halo,
    wish: &HomeWish,
    now_gyr: f64,
    major_ratio: f64,
    disc_kpc: &dyn Fn(&Halo) -> f64,
    neighbour_mass: Option<f64>,
) -> Option<Vec<String>> {
    if !(h.mass >= wish.mass_min && h.mass <= wish.mass_max) {
        return None;
    }
    let mut why = vec![format!("a halo of {:.2e} suns", h.mass)];
    if wish.disc_kpc_min > 0.0 && wish.disc_kpc_max >= wish.disc_kpc_min {
        // A preference, not a cut: it ranks the fits (`choose`).
        why.push(format!("a disc of scale length {:.1} kpc", disc_kpc(h)));
    }
    if wish.quiet_gyr > 0.0 {
        let last = last_major_gyr(h, major_ratio);
        let quiet = last.map_or(f64::INFINITY, |t| now_gyr - t);
        if quiet < wish.quiet_gyr {
            return None;
        }
        why.push(match last {
            None => "no major merger on record".to_string(),
            Some(_) => format!("quiet for {quiet:.1} Gyr since its last major merger"),
        });
    }
    let n = h.neighbour.as_ref();
    match wish.neighbour.as_str() {
        "bound" => {
            let n = n.filter(|n| n.bound && n.distance_mpc <= wish.neighbour_mpc)?;
            let ratio = neighbour_mass.map(|mn| mn / h.mass);
            if wish.neighbour_mass_ratio_max > 0.0
                && ratio.is_none_or(|r| r > wish.neighbour_mass_ratio_max)
            {
                return None;
            }
            why.push(match ratio {
                Some(r) => format!(
                    "a bound neighbour {r:.1} times its mass, {:.2} Mpc away",
                    n.distance_mpc
                ),
                None => format!("a bound bigger neighbour {:.2} Mpc away", n.distance_mpc),
            });
        }
        "alone" => {
            if n.is_some_and(|n| n.distance_mpc <= wish.neighbour_mpc) {
                return None;
            }
            why.push(format!(
                "no bigger neighbour within {} Mpc",
                wish.neighbour_mpc
            ));
        }
        _ => {}
    }
    Some(why)
}

/// The cells of one shell round the origin cell (Chebyshev distance `d`),
/// nearest first and then by index, as `cellsAround` orders them.
pub fn shell(d: i64) -> Vec<[i64; 3]> {
    let mut out = vec![];
    for x in -d..=d {
        for y in -d..=d {
            for z in -d..=d {
                if x.abs().max(y.abs()).max(z.abs()) == d {
                    out.push([x, y, z]);
                }
            }
        }
    }
    out.sort_by_key(|c| (c[0] * c[0] + c[1] * c[1] + c[2] * c[2], *c));
    out
}

/// Every cell within `search_shells` of cell 0.0.0 is read (nearest shell
/// first); the home is the fitting halo nearest the middle of the wished
/// mass range and disc size (in decades, each over its range's half width),
/// ties by the nearer shell and then address. A
/// best fit in a few dozen cells (grand zoom design 6.6), so a universe's
/// home sits where its wish is best met, not merely nearest the origin. With no fit anywhere
/// searched, the halo nearest the middle of the range is taken and marked.
/// A halo's disc scale length today by G3's rule (`galaxy.rs`: the disc
/// factor times spin over root 2 times the virial radius), kpc.
pub fn disc_kpc_of(hs: &Haloes, gp: &crate::galaxy::GalaxyParams, h: &Halo) -> f64 {
    let (r_vir, _) = gp.virial(hs.cosmo, h.mass, 1.0);
    gp.disc_factor * h.spin / m::sqrt(2.0) * r_vir
}

pub fn choose(
    hs: &Haloes,
    gp: &crate::galaxy::GalaxyParams,
    wish: &HomeWish,
    now_gyr: f64,
) -> Result<HomeChoice, String> {
    if !(wish.mass_min > 0.0 && wish.mass_max >= wish.mass_min) {
        return Err("home wish: mass_min must be above 0 and at most mass_max".into());
    }
    if !matches!(wish.neighbour.as_str(), "bound" | "alone" | "any") {
        return Err("home wish: neighbour is bound, alone or any".into());
    }
    // How far a halo is from the middle of each wished range, in units of
    // the range's half width in decades, added in quadrature: mass, and the
    // disc's scale length when one is wished for.
    let mid = 0.5 * (m::log10(wish.mass_min) + m::log10(wish.mass_max));
    let half = (0.5 * (m::log10(wish.mass_max) - m::log10(wish.mass_min))).max(1.0e-3);
    let disc = wish.disc_kpc_min > 0.0 && wish.disc_kpc_max >= wish.disc_kpc_min;
    let d_mid = if disc {
        0.5 * (m::log10(wish.disc_kpc_min) + m::log10(wish.disc_kpc_max))
    } else {
        0.0
    };
    let d_half = if disc {
        (0.5 * (m::log10(wish.disc_kpc_max) - m::log10(wish.disc_kpc_min))).max(1.0e-3)
    } else {
        1.0
    };
    let off = |h: &Halo| {
        let a = (m::log10(h.mass) - mid) / half;
        let b = if disc {
            (m::log10(disc_kpc_of(hs, gp, h)) - d_mid) / d_half
        } else {
            0.0
        };
        a * a + b * b
    };
    let better = |a: &Halo, b: &Halo| off(a).total_cmp(&off(b)).then(a.address.cmp(&b.address));
    let mut nearest: Option<(Halo, [i64; 3])> = None;
    let mut cells_read = 0;
    // Only haloes in the wished range can fit, and a neighbour is always
    // bigger, so the cells are read from the bottom of the range up.
    let bins = hs.bin_table();
    let mut best: Option<(Halo, [i64; 3], Vec<String>)> = None;
    for d in 0..=i64::from(wish.search_shells) {
        for idx in shell(d) {
            cells_read += 1;
            let cell = hs.cell_above_full(idx, &bins, wish.mass_min);
            let mass_of = |a: &str| cell.iter().find(|x| x.address == a).map(|x| x.mass);
            for h in cell.iter().cloned() {
                let nm = h.neighbour.as_ref().and_then(|n| mass_of(&n.address));
                if nearest
                    .as_ref()
                    .is_none_or(|(n, _)| better(&h, n) == std::cmp::Ordering::Less)
                {
                    nearest = Some((h.clone(), idx));
                }
                if let Some(why) = fits(
                    &h,
                    wish,
                    now_gyr,
                    hs.params.major_ratio,
                    &|x| disc_kpc_of(hs, gp, x),
                    nm,
                ) {
                    if best
                        .as_ref()
                        .is_none_or(|(b, _, _)| better(&h, b) == std::cmp::Ordering::Less)
                    {
                        best = Some((h, idx, why));
                    }
                }
            }
        }
    }
    if let Some((halo, cell, why)) = best {
        return Ok(HomeChoice {
            halo,
            cell,
            fits_wish: true,
            why,
            cells_read,
        });
    }
    let (halo, cell) = nearest.ok_or("home: no catalogued halo in the searched cells")?;
    Ok(HomeChoice {
        why: vec![
            format!("a halo of {:.2e} suns", halo.mass),
            "no halo in the searched cells met every wish; this is the one nearest the wished mass"
                .to_string(),
        ],
        halo,
        cell,
        fits_wish: false,
        cells_read,
    })
}

/// The cell index a catalogued halo's address names (`.../cell:x.y.z/halo:n`).
pub fn cell_of(address: &str) -> Option<[i64; 3]> {
    let (_, rest) = address.rsplit_once("/cell:")?;
    let (idx, _) = rest.split_once("/halo:")?;
    let mut it = idx.split('.').map(|s| s.parse::<i64>().ok());
    let c = [it.next()??, it.next()??, it.next()??];
    it.next().is_none().then_some(c)
}

/// A catalogued halo by its address, read from its own cell.
pub fn halo_at(hs: &Haloes, address: &str) -> Result<(Halo, [i64; 3]), String> {
    let idx = cell_of(address)
        .ok_or_else(|| format!("{address} is not a catalogued halo (…/cell:x.y.z/halo:n)"))?;
    let cell = hs.cell(idx);
    if !address.starts_with(&format!("{}/", cell.address)) {
        return Err(format!("{address} is not in this universe"));
    }
    cell.haloes
        .into_iter()
        .find(|h| h.address == address)
        .map(|h| (h, idx))
        .ok_or_else(|| format!("no halo {address} in {}", hs.cell_address(idx)))
}
