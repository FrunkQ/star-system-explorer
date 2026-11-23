//! DM3: the matter ledger of one web cell (galaxy histories design 4.1).
//!
//! ```text
//! matter in the cell (from the web's density)
//!   dark matter     = in catalogued haloes + in haloes below the catalogue floor + never collapsed
//!   ordinary matter = stars + cold gas + hot halo gas + thrown-out gas + black holes
//!                     + gas between galaxies that never fell in
//! ```
//!
//! Every term is computed elsewhere: the cell's matter by the web, its
//! catalogued haloes by G2, the haloes below the floor by the same mass
//! function, and each halo's reservoirs by G3. G3 is run once per half dex
//! of halo mass on a median halo and read off by mass (a halo's baryons by
//! its own mass, not its own history: F48). Hot halo gas is the halo's
//! share of baryons less what its galaxy holds and threw out, so each
//! halo closes; the gas between galaxies is the cell's baryons less every
//! halo's, so the cell closes.

use crate::cosmology::Cosmology;
use crate::galaxy::GalaxyParams;
use crate::haloes::{conditional_dn_dlnm, Haloes};
use crate::web::{Cache, Environment};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Ordinary matter by reservoir, solar masses.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Ordinary {
    pub stars: f64,
    pub cold_gas: f64,
    pub hot_gas: f64,
    pub thrown_out: f64,
    pub black_holes: f64,
    pub between_galaxies: f64,
}

impl Ordinary {
    pub fn total(&self) -> f64 {
        self.stars
            + self.cold_gas
            + self.hot_gas
            + self.thrown_out
            + self.black_holes
            + self.between_galaxies
    }
    fn add(&mut self, o: &Ordinary, k: f64) {
        self.stars += k * o.stars;
        self.cold_gas += k * o.cold_gas;
        self.hot_gas += k * o.hot_gas;
        self.thrown_out += k * o.thrown_out;
        self.black_holes += k * o.black_holes;
    }
}

/// Dark matter by where it is, solar masses.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Dark {
    pub catalogued_haloes: f64,
    pub haloes_below_floor: f64,
    pub never_collapsed: f64,
}

/// One environment's share of a cell's volume and matter.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Share {
    pub environment: Environment,
    pub volume: f64,
    pub matter: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub cell: String,
    pub cell_mpc: f64,
    pub matter_msun: f64,
    pub dark: Dark,
    pub ordinary: Ordinary,
    /// Ordinary over all matter, and the universe's share it must equal.
    pub baryon_share: f64,
    pub f_b: f64,
    /// Haloes counted below the catalogue floor reach down to this mass.
    pub floor_low_msun: f64,
    pub catalogue_floor_msun: f64,
    pub haloes: usize,
    /// The cell's volume and matter by web environment (samples a side cubed).
    pub environments: Vec<Share>,
    pub samples_a_side: u32,
}

/// Per halo mass, its reservoirs as shares of its mass (G3 on a median halo),
/// on a grid of half a dex.
pub struct Reservoirs {
    lm0: f64,
    step: f64,
    rows: Vec<Ordinary>,
}

impl Reservoirs {
    /// G3 on median haloes from `lo` to `hi` solar masses.
    pub fn new(hs: &Haloes, gp: &GalaxyParams, lo: f64, hi: f64) -> Self {
        let c = hs.cosmo;
        let fb = c.p.omega_b / c.p.omega_m;
        let step = 0.5;
        let lm0 = m::log10(lo);
        let n = m::ceil((m::log10(hi) - lm0) / step) as u32 + 1;
        let rows = (0..n)
            .map(|k| {
                let mh = m::pow(10.0, lm0 + step * f64::from(k));
                let h = hs.median(mh, &format!("ledger-{k}"));
                let g = crate::galaxy::evolve(c, gp, &h);
                let n = &g.now;
                let mh = n.halo.max(1.0);
                let held = n.stars + n.gas + n.black_hole;
                let thrown = g.ejected_now.min((fb * mh - held).max(0.0));
                Ordinary {
                    stars: n.stars / mh,
                    cold_gas: n.gas / mh,
                    hot_gas: (fb * mh - held - thrown).max(0.0) / mh,
                    thrown_out: thrown / mh,
                    black_holes: n.black_hole / mh,
                    between_galaxies: 0.0,
                }
            })
            .collect();
        Reservoirs { lm0, step, rows }
    }

    /// Shares of a halo of mass `mh`, linear in log mass between grid points
    /// (clamped at the ends).
    pub fn at(&self, mh: f64) -> Ordinary {
        let x = ((m::log10(mh.max(1.0)) - self.lm0) / self.step).max(0.0);
        let i = (x as usize).min(self.rows.len() - 1);
        let j = (i + 1).min(self.rows.len() - 1);
        let f = (x - i as f64).clamp(0.0, 1.0);
        let (a, b) = (&self.rows[i], &self.rows[j]);
        let l = |p: f64, q: f64| p + (q - p) * f;
        Ordinary {
            stars: l(a.stars, b.stars),
            cold_gas: l(a.cold_gas, b.cold_gas),
            hot_gas: l(a.hot_gas, b.hot_gas),
            thrown_out: l(a.thrown_out, b.thrown_out),
            black_holes: l(a.black_holes, b.black_holes),
            between_galaxies: 0.0,
        }
    }
}

/// Settings (`provides.other.cosmology.ledger`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LedgerParams {
    /// Haloes below the catalogue floor are counted down to this mass.
    pub floor_low_msun: f64,
    /// Below this, a halo holds no galaxy: its baryons count as between
    /// galaxies (about the filtering mass after reionisation, G3).
    pub galaxy_low_msun: f64,
    /// Environment samples a side.
    pub samples: u32,
}

impl Default for LedgerParams {
    fn default() -> Self {
        LedgerParams {
            floor_low_msun: 1.0e6,
            galaxy_low_msun: 1.0e9,
            samples: 8,
        }
    }
}

/// The ledger of one cell today.
pub fn cell(hs: &Haloes, res: &Reservoirs, lp: &LedgerParams, idx: [i64; 3]) -> Ledger {
    let c: &Cosmology = hs.cosmo;
    let cell = hs.cell(idx);
    let side = hs.params.cell_mpc;
    let volume = side * side * side;
    let fb = c.p.omega_b / c.p.omega_m;
    let mut ord = Ordinary::default();
    for h in &cell.haloes {
        ord.add(&res.at(h.mass), h.mass);
    }
    // Below the catalogue floor, by the same conditional mass function.
    let lo = m::ln(lp.floor_low_msun);
    let hi = m::ln(hs.params.min_mass);
    let gal = m::ln(lp.galaxy_low_msun);
    let room = (cell.mass - cell.halo_mass).max(0.0);
    let below_all = crate::cosmology::quad(
        &|x| {
            let mm = m::exp(x);
            conditional_dn_dlnm(c, mm, 0.0, cell.delta_l) * mm
        },
        lo,
        hi,
        8,
    ) * volume;
    let scale = if below_all > room && below_all > 0.0 {
        room / below_all
    } else {
        1.0
    };
    let below = below_all * scale;
    // Those big enough for a galaxy carry G3's reservoirs.
    if hi > gal {
        let n = 16;
        let dx = (hi - gal) / f64::from(n);
        for k in 0..n {
            let x = gal + (f64::from(k) + 0.5) * dx;
            let mm = m::exp(x);
            let mass_in = conditional_dn_dlnm(c, mm, 0.0, cell.delta_l) * mm * dx * volume * scale;
            ord.add(&res.at(mm), mass_in);
        }
    }
    let halo_baryons = ord.stars + ord.cold_gas + ord.hot_gas + ord.thrown_out + ord.black_holes;
    ord.between_galaxies = (fb * cell.mass - halo_baryons).max(0.0);
    let dark = Dark {
        catalogued_haloes: (1.0 - fb) * cell.halo_mass,
        haloes_below_floor: (1.0 - fb) * below,
        never_collapsed: (1.0 - fb) * (cell.mass - cell.halo_mass - below).max(0.0),
    };
    // Environments over the cell's volume.
    let n = lp.samples.max(1);
    let up = hs.web.params.octaves - 1;
    let mut cache = Cache::default();
    let mut vol = [0.0_f64; 4];
    let mut mat = [0.0_f64; 4];
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                let f = |q: u32, id: i64| (id as f64 + (f64::from(q) + 0.5) / f64::from(n)) * side;
                let x = [f(i, idx[0]), f(j, idx[1]), f(k, idx[2])];
                let d = hs.web.density(&mut cache, x, up, 1.0);
                let e = hs.web.environment(&mut cache, x, up, 1.0) as usize;
                vol[e] += 1.0;
                mat[e] += d;
            }
        }
    }
    let (tv, tm) = (
        vol.iter().sum::<f64>(),
        mat.iter().sum::<f64>().max(1.0e-300),
    );
    let environments = [
        Environment::Void,
        Environment::Sheet,
        Environment::Filament,
        Environment::Knot,
    ]
    .iter()
    .map(|&e| Share {
        environment: e,
        volume: vol[e as usize] / tv,
        matter: mat[e as usize] / tm,
    })
    .collect();
    let total_dark = dark.catalogued_haloes + dark.haloes_below_floor + dark.never_collapsed;
    Ledger {
        cell: cell.address.clone(),
        cell_mpc: side,
        matter_msun: cell.mass,
        dark,
        ordinary: ord,
        baryon_share: ord.total() / (ord.total() + total_dark).max(1.0),
        f_b: fb,
        floor_low_msun: lp.floor_low_msun,
        catalogue_floor_msun: hs.params.min_mass,
        haloes: cell.haloes.len(),
        environments,
        samples_a_side: n,
    }
}
