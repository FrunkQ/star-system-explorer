//! H: haloes (galaxy design section 4, stage G2).
//!
//! A region cell (default 8 Mpc comoving) takes its mean linear overdensity
//! from the web (G1). Then:
//! - how many haloes of each mass: the Sheth and Tormen 1999 mass function
//!   with the peak-background split (their eq. 12 bias), so dense cells get
//!   more and bigger haloes; counts are keyed Poisson draws per mass bin,
//!   taken top-down from the largest bin under the cell's mass budget;
//! - where: the highest of a few keyed candidate points on the finer octaves;
//! - growth: M(z) = M0 (1+z)^alpha exp(beta z), the McBride et al. 2009 form
//!   (written there (1+z)^β exp(−γ z), so alpha = β and beta = −γ),
//!   with alpha and beta from the cosmology by Correa et al. 2015 (paper I,
//!   MNRAS 450, 1514, doi:10.1093/mnras/stv689; eqs. 14, 15 and 20 to 23 with B2), shifted earlier in denser cells and scattered per halo;
//! - spin: lognormal, median 0.035, sigma ln = 0.5 (Bullock et al. 2001);
//! - major mergers (ratio above 1:4): a keyed Poisson process in redshift with
//!   the Fakhouri, Ma and Boylan-Kolchin 2010 rate (eq. 1), each a dated
//!   chronicle event with its ratio and an orbit sketch;
//! - neighbours: each halo's nearest larger halo in the cell, with a two-body
//!   sketch (bound or not; time to first approach).

use crate::cosmology::{Cosmology, DELTA_C, YEAR_S};
use crate::web::{Cache, Web};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, ContentHash, Seed};
use std::collections::BTreeMap;

/// Pack settings (`provides.other.cosmology.haloes`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HaloParams {
    pub cell_mpc: f64,
    /// Smallest halo catalogued, solar masses.
    pub min_mass: f64,
    pub max_mass: f64,
    /// Mass bins per decade.
    pub bins_per_dex: u32,
    /// Keyed candidate points per halo; the highest peak wins.
    pub position_candidates: u32,
    /// Formation-time scatter (sigma of ln of Correa's f(M0)).
    pub formation_scatter: f64,
    /// Shift of ln f(M0) per unit of the cell's linear overdensity, subtracted (denser forms earlier).
    pub formation_environment: f64,
    pub spin_median: f64,
    pub spin_sigma_ln: f64,
    /// Smallest ratio counted as a major merger.
    pub major_ratio: f64,
    /// Earliest redshift mergers are drawn from.
    pub merger_z_max: f64,
    /// Neighbour search radius, Mpc.
    pub neighbour_mpc: f64,
    /// GH1: how each halo's assembly tree is drawn.
    #[serde(default)]
    pub assembly: crate::assembly::AssemblyParams,
}

impl Default for HaloParams {
    fn default() -> Self {
        HaloParams {
            cell_mpc: 8.0,
            min_mass: 1.0e10,
            max_mass: 1.0e16,
            bins_per_dex: 4,
            position_candidates: 6,
            formation_scatter: 0.3,
            formation_environment: 0.3,
            spin_median: 0.035,
            spin_sigma_ln: 0.5,
            major_ratio: 0.25,
            merger_z_max: 6.0,
            neighbour_mpc: 3.0,
            assembly: Default::default(),
        }
    }
}

/// Sheth and Tormen 1999 constants (A, a, p).
pub const ST_A: f64 = 0.3222;
pub const ST_SMALL_A: f64 = 0.707;
pub const ST_P: f64 = 0.3;

/// Sheth-Tormen multiplicity (ST99 eq. 10, where nu = (delta_c / sigma)^2; written here as
/// f(sigma) = 2 nu f(nu) in nu = delta_c / sigma).
pub fn st_f(nu: f64) -> f64 {
    let an2 = ST_SMALL_A * nu * nu;
    ST_A * m::sqrt(2.0 * ST_SMALL_A / m::PI) * (1.0 + m::pow(an2, -ST_P)) * nu * m::exp(-0.5 * an2)
}

/// Sheth-Tormen Lagrangian-to-Eulerian bias b(nu) (Sheth and Tormen 1999 eq. 12; Mo and White 1996 form).
pub fn st_bias(nu: f64) -> f64 {
    let an2 = ST_SMALL_A * nu * nu;
    1.0 + (an2 - 1.0) / DELTA_C + 2.0 * ST_P / (DELTA_C * (1.0 + m::pow(an2, ST_P)))
}

/// dn/dln M (per cubic Mpc) at redshift z, unconditional.
pub fn dn_dlnm(c: &Cosmology, mass: f64, z: f64) -> f64 {
    let d = c.growth(1.0 / (1.0 + z));
    let s = c.sigma_m(mass) * d;
    let nu = DELTA_C / s;
    let h = 0.01;
    let dlns =
        (m::ln(c.sigma_m(mass * m::exp(h))) - m::ln(c.sigma_m(mass * m::exp(-h)))) / (2.0 * h);
    c.rho_m() / mass * st_f(nu) * dlns.abs()
}

/// Expected haloes per cubic Mpc in a cell of linear overdensity `delta_l`
/// (at z), by the peak-background split: (1 + b(nu) delta_l), floored at 0.
pub fn conditional_dn_dlnm(c: &Cosmology, mass: f64, z: f64, delta_l: f64) -> f64 {
    let d = c.growth(1.0 / (1.0 + z));
    let nu = DELTA_C / (c.sigma_m(mass) * d);
    dn_dlnm(c, mass, z) * (1.0 + st_bias(nu) * delta_l).max(0.0)
}

/// Poisson draw from a keyed uniform: inversion for small means, a rounded
/// normal for large ones (fixed work either way).
pub fn poisson(d: &Draws, i: u64, lambda: f64) -> u64 {
    if lambda <= 0.0 {
        return 0;
    }
    if lambda > 50.0 {
        let x = lambda + m::sqrt(lambda) * d.normal(i);
        return m::round(x.max(0.0)) as u64;
    }
    let u = d.unit(2 * i);
    let mut p = m::exp(-lambda);
    let mut cdf = p;
    let mut k = 0u64;
    while u > cdf && k < 200 {
        k += 1;
        p *= lambda / k as f64;
        cdf += p;
    }
    k
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Merger {
    pub z: f64,
    /// Universe time, Gyr.
    pub t_gyr: f64,
    /// Mass ratio (smaller over larger).
    pub ratio: f64,
    /// Halo mass at the merger, solar masses.
    pub mass: f64,
    /// Orbit sketch: circularity and pericentre in virial radii (Benson 2005 ranges).
    pub circularity: f64,
    pub pericentre_rvir: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Neighbour {
    pub address: String,
    pub distance_mpc: f64,
    pub bound: bool,
    /// Time to first approach on a radial orbit from rest, Gyr (bound only).
    pub approach_gyr: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Halo {
    pub address: String,
    /// Mass today, solar masses.
    pub mass: f64,
    /// Comoving position, Mpc.
    pub position: [f64; 3],
    /// Growth history M(z) = M0 (1+z)^alpha exp(beta z).
    pub alpha: f64,
    pub beta: f64,
    /// Redshift at which half today's mass was in place.
    pub z_half: f64,
    pub spin: f64,
    pub mergers: Vec<Merger>,
    pub neighbour: Option<Neighbour>,
}

impl Halo {
    pub fn mass_at(&self, z: f64) -> f64 {
        self.mass * m::pow(1.0 + z, self.alpha) * m::exp(self.beta * z)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub address: String,
    pub index: [i64; 3],
    /// Linear overdensity today (smoothed at the cell's scale).
    pub delta_l: f64,
    /// Matter in the cell, solar masses (lognormal density times volume).
    pub mass: f64,
    /// Expected and drawn counts per bin: (log10 M at the bin centre, expected, drawn).
    pub bins: Vec<(f64, f64, u64)>,
    pub haloes: Vec<Halo>,
    /// Mass in catalogued haloes; never above `mass`.
    pub halo_mass: f64,
}

/// Correa et al. 2015 (paper I, MNRAS 450, 1514) eqs. 14, 15 and 20 to 23 (constant 1.8837 from eq. B2): alpha and beta of the
/// McBride form from the power spectrum, for a halo of mass M0 today.
pub fn correa(c: &Cosmology, m0: f64, shift_ln_f: f64) -> (f64, f64) {
    let lm = m::log10(m0);
    let zf = -0.0064 * lm * lm + 0.0237 * lm + 1.8837;
    let q = 4.137 * m::pow(zf, -0.9476);
    let s0 = c.sigma_m(m0);
    let sq = c.sigma_m(m0 / q);
    let f = m::exp(shift_ln_f) / m::sqrt((sq * sq - s0 * s0).max(1.0e-6));
    // dD/dz at z = 0 by central difference of D(1/(1+z)).
    let h = 1.0e-3;
    let dd = (c.growth(1.0 / (1.0 + h)) - c.growth(1.0 / (1.0 - h))) / (2.0 * h);
    let alpha = (1.686 * m::sqrt(2.0 / m::PI) * dd + 1.0) * f;
    let beta = -f;
    (alpha, beta)
}

/// Fakhouri, Ma and Boylan-Kolchin 2010 eq. 1: mergers per halo per unit z
/// per unit ratio xi.
pub fn fmbk_rate(mass: f64, xi: f64, z: f64) -> f64 {
    let (a, alpha, beta, gamma, eta, xt) = (0.0104, 0.133, -1.995, 0.263, 0.0993, 9.72e-3);
    a * m::pow(mass / 1.0e12, alpha)
        * m::pow(xi, beta)
        * m::exp(m::pow(xi / xt, gamma))
        * m::pow(1.0 + z, eta)
}

/// Major mergers per unit z (xi from `min_ratio` to 1), by fixed quadrature.
pub fn major_rate(mass: f64, min_ratio: f64, z: f64) -> f64 {
    crate::cosmology::quad(&|xi| fmbk_rate(mass, xi, z), min_ratio, 1.0, 4)
}

pub struct Haloes<'a> {
    pub cosmo: &'a Cosmology,
    pub web: &'a Web,
    pub params: HaloParams,
    pub seed: Seed,
    pub universe: Address,
}

fn stream(s: &str) -> Stream {
    Stream::new(s).expect("valid stream")
}

impl Haloes<'_> {
    /// The octave whose spacing matches the cell (the cell's smoothing scale).
    fn cell_octave(&self) -> u32 {
        let mut j = 0;
        while j + 1 < self.web.params.octaves && self.web.spacing(j + 1) >= self.params.cell_mpc {
            j += 1;
        }
        j
    }

    pub fn cell_address(&self, idx: [i64; 3]) -> Address {
        self.universe
            .child_key("cell", &format!("{}.{}.{}", idx[0], idx[1], idx[2]))
            .expect("valid cell key")
    }

    /// The cell's halo catalogue. Pure: the same call gives the same cell on
    /// every device, in any order.
    pub fn cell(&self, idx: [i64; 3]) -> Cell {
        let bins = self.bin_table();
        self.cell_from(idx, &bins, self.params.min_mass, true)
    }

    /// Each mass bin's unconditional abundance and bias today, top bin first
    /// (what `conditional_dn_dlnm` multiplies out per cell; the same numbers,
    /// worked once for a block of cells).
    pub fn bin_table(&self) -> Vec<(f64, f64)> {
        let p = &self.params;
        let c = self.cosmo;
        let nb = m::round(m::log10(p.max_mass / p.min_mass) * f64::from(p.bins_per_dex)) as u32;
        let w = 1.0 / f64::from(p.bins_per_dex);
        let d = c.growth(1.0);
        (0..nb)
            .rev()
            .map(|b| {
                let mm = m::pow(10.0, m::log10(p.min_mass) + (f64::from(b) + 0.5) * w);
                let nu = DELTA_C / (c.sigma_m(mm) * d);
                (dn_dlnm(c, mm, 0.0), st_bias(nu))
            })
            .collect()
    }

    /// The haloes of a cell from `floor` solar masses up, without merger
    /// histories or neighbours: for drawing many cells at once (the web
    /// cube's galaxies). Bins are drawn from the top down and every draw is
    /// keyed by its bin and its halo's number, so these are exactly the
    /// haloes `cell` gives above the floor.
    pub fn cell_above(&self, idx: [i64; 3], bins: &[(f64, f64)], floor: f64) -> Vec<Halo> {
        self.cell_from(idx, bins, floor, false).haloes
    }

    fn cell_from(&self, idx: [i64; 3], table: &[(f64, f64)], floor: f64, full: bool) -> Cell {
        let p = &self.params;
        let c = self.cosmo;
        let addr = self.cell_address(idx);
        let side = p.cell_mpc;
        let centre = [
            (idx[0] as f64 + 0.5) * side,
            (idx[1] as f64 + 0.5) * side,
            (idx[2] as f64 + 0.5) * side,
        ];
        let mut cache = Cache::default();
        let j = self.cell_octave();
        let delta_l = self.web.delta0(&mut cache, centre, j);
        let volume = side * side * side;
        let s2 = self.web.variance[j as usize];
        let mass = c.rho_m() * volume * m::exp(delta_l - 0.5 * s2);
        let counts = Draws::new(self.seed, addr.clone(), stream("cosmos.halo.count.v1"));
        // Bins from the top down; the budget is the cell's matter.
        let nb = m::round(m::log10(p.max_mass / p.min_mass) * f64::from(p.bins_per_dex)) as u32;
        let w = 1.0 / f64::from(p.bins_per_dex);
        let mut bins = vec![];
        let mut drawn: Vec<(f64, u64)> = vec![];
        let mut budget = mass;
        for (b, (dn, bias)) in (0..nb).rev().zip(table) {
            let lm = m::log10(p.min_mass) + (f64::from(b) + 0.5) * w;
            if !full && m::pow(10.0, lm + 0.5 * w) < floor {
                // Every halo in this bin and those below is under the floor.
                break;
            }
            // conditional_dn_dlnm, from the table.
            let expected = dn * (1.0 + bias * delta_l).max(0.0) * w * m::ln(10.0) * volume;
            let mut n = poisson(&counts, u64::from(b), expected);
            let top = m::pow(10.0, lm + 0.5 * w);
            let fit = (budget / top).max(0.0) as u64;
            n = n.min(fit);
            budget -= n as f64 * top;
            bins.push((lm, expected, n));
            drawn.push((lm, n));
        }
        // Each halo: mass within its bin, place, growth, spin, mergers.
        let within = Draws::new(self.seed, addr.clone(), stream("cosmos.halo.mass.v1"));
        let place = Draws::new(self.seed, addr.clone(), stream("cosmos.halo.place.v1"));
        let grow = Draws::new(self.seed, addr.clone(), stream("cosmos.halo.growth.v1"));
        let mut haloes = vec![];
        let mut k = 0u64;
        for (lm, n) in drawn {
            for _ in 0..n {
                let mh = m::pow(10.0, lm - 0.5 * w + w * within.unit(k));
                if !full && mh < floor {
                    k += 1;
                    continue;
                }
                let ha = addr.child("halo", k).expect("valid");
                // Position: the highest of a few keyed candidates on the finer octaves.
                let mut best = ([0.0; 3], f64::NEG_INFINITY);
                for q in 0..u64::from(p.position_candidates) {
                    let i = k * 64 + q * 3;
                    let x = [
                        centre[0] + side * (place.unit(i) - 0.5),
                        centre[1] + side * (place.unit(i + 1) - 0.5),
                        centre[2] + side * (place.unit(i + 2) - 0.5),
                    ];
                    let v = self.web.delta0(&mut cache, x, self.web.params.octaves - 1);
                    if v > best.1 {
                        best = (x, v);
                    }
                }
                let shift =
                    p.formation_scatter * grow.normal(4 * k) - p.formation_environment * delta_l;
                let (alpha, beta) = correa(c, mh, shift);
                let spin = p.spin_median * m::exp(p.spin_sigma_ln * grow.normal(4 * k + 1));
                let mut h = Halo {
                    address: ha.to_string(),
                    mass: mh,
                    position: best.0,
                    alpha,
                    beta,
                    z_half: 0.0,
                    spin,
                    mergers: vec![],
                    neighbour: None,
                };
                h.z_half = half_mass_z(&h);
                if full {
                    h.mergers = self.mergers(&h, &ha);
                }
                haloes.push(h);
                k += 1;
            }
        }
        if full {
            self.link_neighbours(&mut haloes);
        }
        let halo_mass = haloes.iter().map(|h| h.mass).sum();
        Cell {
            address: addr.to_string(),
            index: idx,
            delta_l,
            mass,
            bins,
            haloes,
            halo_mass,
        }
    }

    /// A halo of mass `mass` today with the median growth history and spin and
    /// its own keyed mergers (for reference galaxies and volume sums).
    pub fn median(&self, mass: f64, key: &str) -> Halo {
        let addr = self.universe.child_key("halo", key).expect("valid key");
        self.median_at(mass, &addr)
    }

    /// As `median`, at a given address (a progenitor in an assembly tree).
    pub fn median_at(&self, mass: f64, addr: &Address) -> Halo {
        let addr = addr.clone();
        let (alpha, beta) = correa(self.cosmo, mass, 0.0);
        let mut h = Halo {
            address: addr.to_string(),
            mass,
            position: [0.0; 3],
            alpha,
            beta,
            z_half: 0.0,
            spin: self.params.spin_median,
            mergers: vec![],
            neighbour: None,
        };
        h.z_half = half_mass_z(&h);
        h.mergers = self.mergers(&h, &addr);
        h
    }

    /// Major mergers of one halo: a keyed Poisson process in z from merger_z_max to 0
    /// on a fixed grid of 60 slices, the rate at the halo's mass then.
    fn mergers(&self, h: &Halo, addr: &Address) -> Vec<Merger> {
        let p = &self.params;
        let d = Draws::new(self.seed, addr.clone(), stream("cosmos.halo.merger.v1"));
        let n = 60u32;
        let dz = p.merger_z_max / f64::from(n);
        let mut out = vec![];
        for s in 0..n {
            let z = p.merger_z_max - (f64::from(s) + 0.5) * dz;
            let mz = h.mass_at(z);
            if mz < p.min_mass * 0.1 {
                continue;
            }
            let lambda = major_rate(mz, p.major_ratio, z) * dz;
            let count = poisson(&d, u64::from(s) * 16, lambda);
            for c in 0..count.min(3) {
                let i = u64::from(s) * 16 + 4 + c * 4;
                // Ratio drawn from the full FMBK eq. 1 shape on [major_ratio, 1],
                // xi^beta exp((xi / xi_t)^gamma), by inverting its integral with a
                // fixed 40-step bisection. (The exponential factor rises about 3 times
                // from xi = 0.25 to 1, so the bare power law is not enough.)
                let u = d.unit(i);
                let total = major_rate(mz, p.major_ratio, z);
                let (mut lo, mut hi) = (p.major_ratio, 1.0_f64);
                for _ in 0..40 {
                    let mid = 0.5 * (lo + hi);
                    let part =
                        crate::cosmology::quad(&|x| fmbk_rate(mz, x, z), p.major_ratio, mid, 4);
                    if part < u * total {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let xi = 0.5 * (lo + hi);
                let zz = z + dz * (d.unit(i + 1) - 0.5);
                out.push(Merger {
                    z: zz,
                    t_gyr: self.cosmo.time_of_scale(1.0 / (1.0 + zz)),
                    ratio: xi,
                    mass: h.mass_at(zz),
                    circularity: 0.3 + 0.7 * d.unit(i + 2),
                    pericentre_rvir: 0.05 + 0.45 * d.unit(i + 3),
                });
            }
        }
        out.sort_by(|a, b| b.z.total_cmp(&a.z));
        out
    }

    /// Each halo's nearest more massive halo within the search radius, with a two-body sketch.
    fn link_neighbours(&self, haloes: &mut [Halo]) {
        let r = self.params.neighbour_mpc;
        let g_mpc = 4.300_917e-9; // G in Mpc (km/s)^2 / Msun
        let h0 = self.cosmo.p.h0;
        let snapshot: Vec<(String, f64, [f64; 3])> = haloes
            .iter()
            .map(|h| (h.address.clone(), h.mass, h.position))
            .collect();
        for h in haloes.iter_mut() {
            let mut best: Option<(usize, f64)> = None;
            for (k, (_, mm, x)) in snapshot.iter().enumerate() {
                if *mm <= h.mass {
                    continue;
                }
                let d = m::sqrt(
                    (0..3)
                        .map(|i| (x[i] - h.position[i]) * (x[i] - h.position[i]))
                        .sum::<f64>(),
                );
                if d < r && best.is_none_or(|b| d < b.1) {
                    best = Some((k, d));
                }
            }
            if let Some((k, d)) = best {
                // Physical separation today equals comoving (a = 1). Hubble-flow speed against escape speed.
                let mt = h.mass + snapshot[k].1;
                let v_esc2 = 2.0 * g_mpc * mt / d;
                let v_h = h0 * d;
                let bound = v_h * v_h < v_esc2;
                let approach = bound.then(|| {
                    // Radial free fall from rest at d: t = (pi / 2) sqrt(d^3 / (2 G M)).
                    let t_s =
                        0.5 * m::PI * m::sqrt(d * d * d / (2.0 * g_mpc * mt)) * 3.085_677_581e19;
                    t_s / YEAR_S / 1.0e9
                });
                h.neighbour = Some(Neighbour {
                    address: snapshot[k].0.clone(),
                    distance_mpc: d,
                    bound,
                    approach_gyr: approach,
                });
            }
        }
    }

    /// Record a cell's major mergers as `halo.merger` events (derived; a GM may
    /// add, delete or move one: authored edits win when the cell is read back).
    pub fn record(&self, ch: &mut Chronicle, cell: &Cell) -> Result<Vec<ContentHash>, String> {
        let mut evs: Vec<(UniverseTime, &Halo, &Merger)> = vec![];
        for h in &cell.haloes {
            for mg in &h.mergers {
                let s = m::round(mg.t_gyr * 1.0e9 * YEAR_S) as i64;
                evs.push((UniverseTime::seconds(s), h, mg));
            }
        }
        evs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.address.cmp(&b.1.address)));
        let mut ids = vec![];
        for (at, h, mg) in evs {
            let mut data = BTreeMap::new();
            data.insert("ratio".into(), Value::Float(mg.ratio));
            data.insert("z".into(), Value::Float(mg.z));
            data.insert("mass".into(), Value::Float(mg.mass));
            data.insert("circularity".into(), Value::Float(mg.circularity));
            data.insert("pericentre_rvir".into(), Value::Float(mg.pericentre_rvir));
            data.insert(
                "why".into(),
                Value::Text(format!(
                    "a halo of {:.1e} solar masses swallows one {:.0} percent of its mass at z = {:.2} (major-merger rate of Fakhouri, Ma and Boylan-Kolchin 2010)",
                    mg.mass,
                    mg.ratio * 100.0,
                    mg.z
                )),
            );
            let e = Event::new(EventBody {
                kind: "halo.merger".into(),
                subject: vec![h.address.parse().map_err(|e| format!("{e:?}"))?],
                occurred_at: at,
                activated_at: at,
                recognised_at: ch.next_stamp(),
                causes: vec![],
                payload: Payload {
                    data: Some(Value::Map(data)),
                    ..Default::default()
                },
                evidence: Evidence::Derived,
                publisher: Publisher::new("sse.base-cosmos", "haloes", "1"),
            })
            .map_err(|e| format!("{e:?}"))?;
            ids.push(ch.append(e).map_err(|e| format!("{e:?}"))?);
        }
        Ok(ids)
    }
}

/// Redshift at which the halo had half its present mass (fixed bisection).
pub fn half_mass_z(h: &Halo) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 20.0_f64);
    if h.mass_at(hi) > 0.5 * h.mass {
        return hi;
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if h.mass_at(mid) > 0.5 * h.mass {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}
