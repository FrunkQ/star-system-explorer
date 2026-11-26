//! S: stars, identity and time (galaxy design section 8, stage G5).
//!
//! Space is cut into cubic cells (default 10 pc) in the galaxy's frame and
//! time into frames (default 1 Myr). The rule of section 8.3:
//! - **fill**: the unresolved systems of a cell in a frame are drawn from the
//!   G4 envelope at the cell's centre and the frame's start, keyed by (cell,
//!   frame): a Poisson count, then per system its birth time (from the ring's
//!   own star formation history), initial mass (Kroupa IMF), companions
//!   (multiplicity and separations by primary mass), abundances (the gas at
//!   birth plus scatter), birth cluster mass, a place in the cell and an
//!   epicyclic orbit through it (dispersions rising with age);
//! - **pins**: a promoted system is permanent. Its orbit and life are closed
//!   form, so it is the same system at any time asked in any order. Pins in a
//!   cell claim their share of the frame's count first: a pin promoted from
//!   this cell and frame keeps its own slot, a pin that has drifted in
//!   replaces the last fill system, so nothing is counted twice;
//! - **promotion** is `promote(fill system)`: the same record, now a pin.
//!
//! Stellar evolution here is analytic (main-sequence power laws, a short
//! giant phase, remnants by initial mass: Kalirai et al. 2008 for white
//! dwarfs); pack tables from published tracks replace it behind the same
//! `state_at`.

use crate::envelope::{GalaxyEnvelope, GreenParams, Place};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

/// Settings (`provides.other.cosmology.stars`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StarParams {
    pub cell_pc: f64,
    pub frame_myr: f64,
    /// Kroupa 2001: dN/dM ~ M^-alpha between the edges (solar masses).
    pub imf_edges: Vec<f64>,
    pub imf_alphas: Vec<f64>,
    /// Main-sequence lifetime = sun_life_gyr x M^-lifetime_slope.
    pub sun_life_gyr: f64,
    pub lifetime_slope: f64,
    /// Giant phase length as a share of the main-sequence life.
    pub giant_share: f64,
    /// Giant luminosity = giant_l_factor x the main-sequence luminosity +
    /// giant_l_floor_lsun, at giant_teff_k (fudge F-S11). The floor is the
    /// red clump, where a low-mass giant spends most of its giant life:
    /// about 50 L☉ at 4,500 to 4,800 K (Girardi 2016,
    /// doi:10.1146/annurev-astro-081915-023354). A massive star leaves the
    /// main sequence about twice as bright as it began (Hurley, Pols and
    /// Tout 2000, doi:10.1046/j.1365-8711.2000.03426.x) and crosses to a
    /// supergiant at about that luminosity.
    pub giant_l_factor: f64,
    pub giant_l_floor_lsun: f64,
    pub giant_teff_k: f64,
    /// Eker et al. 2018's relation is fitted up to 31 M☉; above it the
    /// main-sequence luminosity grows as M^mlr_top_slope, which meets the
    /// Geneva tracks (Ekström et al. 2012, doi:10.1051/0004-6361/201117751:
    /// log L about 6.2 at 120 M☉) instead of Eker's extrapolated 7.
    pub mlr_top_msun: f64,
    pub mlr_top_slope: f64,
    /// No cool supergiant is brighter than the Humphreys-Davidson limit,
    /// log L about 5.8 (Humphreys and Davidson 1979, doi:10.1086/157301):
    /// a star whose giant phase would pass it stays hot instead.
    pub cool_limit_lsun: f64,
    /// White dwarf mass = wd_a x M + wd_b (Kalirai et al. 2008).
    pub wd_a: f64,
    pub wd_b: f64,
    pub ns_min_msun: f64,
    pub bh_min_msun: f64,
    pub ns_mass_msun: f64,
    pub bh_share: f64,
    /// Multiplicity fraction by primary mass: (mass, fraction), interpolated
    /// in log mass (Duchene and Kraus 2013).
    pub multiplicity: Vec<(f64, f64)>,
    /// Companion separation: log10 a (au) normal with this mean and spread
    /// (Raghavan et al. 2010), mass ratio uniform from q_min to 1.
    pub log_a_mean: f64,
    pub log_a_sd: f64,
    pub q_min: f64,
    pub triple_share: f64,
    pub feh_scatter: f64,
    /// Velocity dispersions at 10 Gyr (km/s) and their growth exponent with age.
    pub sigma_r_10: f64,
    pub sigma_z_10: f64,
    pub sigma_floor: f64,
    pub sigma_exponent: f64,
    /// Midplane density for vertical oscillation, solar masses per pc^3.
    pub midplane_density: f64,
    /// Birth cluster mass function ~ M^-2 between these (solar masses).
    pub cluster_min: f64,
    pub cluster_max: f64,
    /// Cells co-rotate with the galaxy at this angular speed (km/s/kpc), so
    /// nearby stars drift through them slowly. This is a frame convention,
    /// not a measured speed: the grid's turn fixes where every cell is at
    /// every time, so it stays at its first value (220 km/s at 8.2 kpc)
    /// whatever the galaxy's rotation curve says (DM4 log).
    pub cell_rotation_kms_kpc: f64,
    /// Version of the fill's draw keys. Raising it redraws every unpinned
    /// system (a change to the fill rules does the same); pins are read
    /// from the record and never move (night sky design, Y7a).
    #[serde(skip_serializing_if = "is_first_key")]
    pub key_version: u32,
    /// Y1, the count tree (`key_version` 2): luminosity class edges, L☉;
    /// levels from a root block to a cell (6: 640 pc blocks of 10 pc cells);
    /// and the keyed attempts a leaf draw makes to find a star of its class.
    pub class_edges_lsun: Vec<f64>,
    pub tree_depth: u32,
    pub class_attempts: u64,
}

fn is_first_key(v: &u32) -> bool {
    *v == 1
}

impl Default for StarParams {
    fn default() -> Self {
        StarParams {
            cell_pc: 10.0,
            frame_myr: 0.1,
            imf_edges: vec![0.08, 0.5, 150.0],
            imf_alphas: vec![1.3, 2.3],
            sun_life_gyr: 10.0,
            lifetime_slope: 2.5,
            giant_share: 0.1,
            giant_l_factor: 2.0,
            giant_l_floor_lsun: 50.0,
            giant_teff_k: 4500.0,
            mlr_top_msun: 31.0,
            mlr_top_slope: 1.5,
            cool_limit_lsun: 6.3e5,
            wd_a: 0.109,
            wd_b: 0.394,
            ns_min_msun: 8.0,
            bh_min_msun: 20.0,
            ns_mass_msun: 1.4,
            bh_share: 0.3,
            // Duchene and Kraus 2013 table 1 (the same points as the formation suite).
            multiplicity: vec![
                (0.1, 0.22),
                (0.3, 0.26),
                (1.0, 0.44),
                (3.0, 0.5),
                (12.0, 0.6),
                (30.0, 0.8),
                (150.0, 0.8),
            ],
            log_a_mean: 1.7,
            log_a_sd: 1.5,
            q_min: 0.1,
            // Raghavan et al. 2010: triples 8 and higher orders 3 of 44 percent of
            // multiples; systems are capped at three stars, so both count here.
            triple_share: 0.25,
            feh_scatter: 0.1,
            sigma_r_10: 40.0,
            sigma_z_10: 25.0,
            sigma_floor: 5.0,
            sigma_exponent: 0.35,
            midplane_density: 0.1,
            cluster_min: 50.0,
            cluster_max: 1.0e5,
            cell_rotation_kms_kpc: 26.83,
            key_version: 2,
            class_edges_lsun: vec![
                0.1, 1.0, 3.0, 10.0, 30.0, 100.0, 300.0, 1.0e3, 3.0e3, 1.0e4, 3.0e4, 1.0e5, 3.0e5,
            ],
            tree_depth: 6,
            class_attempts: 65536,
        }
    }
}

/// A closed-form epicyclic orbit (flat rotation curve, harmonic vertical
/// oscillation), times in Myr of universe time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Orbit {
    pub r_guide_pc: f64,
    pub amp_r_pc: f64,
    pub amp_z_pc: f64,
    pub phase_r: f64,
    pub phase_z: f64,
    pub phi0: f64,
    pub t0_myr: f64,
    /// Angular frequencies, per Myr.
    pub omega: f64,
    pub kappa: f64,
    pub nu: f64,
}

impl Orbit {
    /// Galactocentric (R pc, phi rad, z pc) at universe time t (Myr).
    pub fn at(&self, t_myr: f64) -> (f64, f64, f64) {
        let dt = t_myr - self.t0_myr;
        let pr = self.kappa * dt + self.phase_r;
        let r = self.r_guide_pc + self.amp_r_pc * m::cos(pr);
        let phi = self.phi0 + self.omega * dt
            - 2.0 * self.omega / self.kappa * self.amp_r_pc / self.r_guide_pc * m::sin(pr);
        let z = self.amp_z_pc * m::cos(self.nu * dt + self.phase_z);
        (r, phi, z)
    }
    pub fn xyz(&self, t_myr: f64) -> [f64; 3] {
        let (r, phi, z) = self.at(t_myr);
        [r * m::cos(phi), r * m::sin(phi), z]
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Companion {
    pub mass_msun: f64,
    pub a_au: f64,
}

/// One system: a fill draw or a pin (same record).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StarSystem {
    pub address: Address,
    /// Cell key and frame it was drawn in, and its slot there.
    pub cell: [i64; 3],
    pub frame: i64,
    pub slot: u64,
    pub birth_gyr: f64,
    pub mass_msun: f64,
    pub companions: Vec<Companion>,
    pub feh: f64,
    pub alpha_fe: f64,
    pub c_o: f64,
    pub r_fe: f64,
    pub k40_fe: f64,
    pub cluster_mass_msun: f64,
    pub orbit: Orbit,
    pub pinned: bool,
    /// GH4: a star of the stellar halo, born in a galaxy this one tore apart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub born_in: Option<BornIn>,
    /// A star of the bulge (Local Arm package), drawn by the bulge's own
    /// count tree; its key is `f<frame>b<slot>`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub in_bulge: bool,
}

/// Which of a galaxy's populations a drawn star belongs to: each has its own
/// count tree, keys and slots in a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Disc,
    Halo,
    Bulge,
}

impl StarSystem {
    pub fn kind(&self) -> Kind {
        if self.born_in.is_some() {
            Kind::Halo
        } else if self.in_bulge {
            Kind::Bulge
        } else {
            Kind::Disc
        }
    }
}

/// Where a halo star was born (GH4): the galaxy it came from, and what is
/// left of that galaxy here.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BornIn {
    /// The progenitor's address in the assembly tree.
    pub galaxy: String,
    pub fell_gyr: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub torn_gyr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_gyr: Option<f64>,
    /// Its stars' total in the host now, M☉.
    pub galaxy_stars_msun: f64,
    /// `stream`, `shell` or `mixed remains` where the star is.
    pub remains: String,
    /// The card's line: "born in a dwarf galaxy that fell in 9.6 Gyr ago
    /// and was torn apart 7.2 Gyr ago".
    pub says: String,
}

/// What a star is at a time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StarState {
    /// `unborn`, `main sequence`, `giant`, `white dwarf`, `neutron star`, `black hole`.
    pub phase: String,
    pub mass_msun: f64,
    pub luminosity_lsun: f64,
    pub teff_k: f64,
    pub age_gyr: f64,
}

/// Everything about a system at a time: where it is and what its star is.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SystemAt {
    pub xyz_pc: [f64; 3],
    pub state: StarState,
}

pub struct Stars<'a> {
    pub env: &'a GalaxyEnvelope<'a>,
    pub green: GreenParams,
    pub params: StarParams,
    pub seed: Seed,
    pub galaxy: Address,
}

/// The star grid's turn at a time (radians): the home, fixed in the grid at
/// (R_home, 0, 0), sits at this galactic azimuth.
pub fn grid_angle(params: &StarParams, t_myr: f64) -> f64 {
    params.cell_rotation_kms_kpc * KMS_PC_MYR / 1000.0 * t_myr
}

const KMS_PC_MYR: f64 = 1e3 * 1e6 * sse_core::sheet::YR_JULIAN / sse_core::sheet::PC;
const G_PC3: f64 = sse_core::sheet::GM_SUN
    * (1e6 * sse_core::sheet::YR_JULIAN)
    * (1e6 * sse_core::sheet::YR_JULIAN)
    / (sse_core::sheet::PC * sse_core::sheet::PC * sse_core::sheet::PC);

/// Share of a cumulative history's births by `t`.
fn birth_share(cdf: &[(f64, f64)], acc: f64, t: f64) -> f64 {
    if acc <= 0.0 {
        return 0.0;
    }
    let v = match cdf.windows(2).find(|w| w[1].0 >= t) {
        None => acc,
        Some(w) if t <= w[0].0 => w[0].1,
        Some(w) => {
            let f = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-300);
            w[0].1 + f * (w[1].1 - w[0].1)
        }
    };
    (v / acc).clamp(0.0, 1.0)
}

/// A birth time from the cumulative star formation and a unit draw.
pub(crate) fn birth_from(cdf: &[(f64, f64)], acc: f64, u: f64, t_now_gyr: f64) -> f64 {
    let target = u * acc;
    cdf.windows(2)
        .find(|w| w[1].1 >= target)
        .map_or(t_now_gyr, |w| {
            let f = if w[1].1 > w[0].1 {
                (target - w[0].1) / (w[1].1 - w[0].1)
            } else {
                0.0
            };
            w[0].0 + f * (w[1].0 - w[0].0)
        })
}

pub fn imf_quantile(p: &StarParams, u: f64) -> f64 {
    // Piecewise power law by number.
    let e = &p.imf_edges;
    let a = &p.imf_alphas;
    let seg = |k: usize| {
        let (lo, hi, al) = (e[k], e[k + 1], a[k]);
        // Continuity: scale each piece to join the last.
        let mut c = 1.0;
        for j in 0..k {
            c *= m::pow(e[j + 1], a[j + 1] - a[j]);
        }
        let int = |x: f64| c * m::pow(x, 1.0 - al) / (1.0 - al);
        (int(hi) - int(lo), lo, al, c)
    };
    let pieces: Vec<(f64, f64, f64, f64)> = (0..a.len()).map(seg).collect();
    let total: f64 = pieces.iter().map(|x| x.0).sum();
    let mut target = u * total;
    for (w, lo, al, c) in &pieces {
        if target <= *w {
            let base = m::pow(*lo, 1.0 - al);
            return m::pow(base + target * (1.0 - al) / c, 1.0 / (1.0 - al));
        }
        target -= w;
    }
    *e.last().unwrap()
}

impl StarParams {
    pub fn lifetime_gyr(&self, mass: f64) -> f64 {
        (self.sun_life_gyr * m::pow(mass, -self.lifetime_slope)).max(0.003)
    }
    pub(crate) fn multiplicity_at(&self, mass: f64) -> f64 {
        let t = &self.multiplicity;
        if mass <= t[0].0 {
            return t[0].1;
        }
        for w in t.windows(2) {
            if mass <= w[1].0 {
                let f = (m::ln(mass) - m::ln(w[0].0)) / (m::ln(w[1].0) - m::ln(w[0].0));
                return w[0].1 + f * (w[1].1 - w[0].1);
            }
        }
        t.last().unwrap().1
    }
    /// The primary's state at an age (analytic stand-in for pack tracks).
    pub fn state(&self, mass: f64, age_gyr: f64) -> StarState {
        let life = self.lifetime_gyr(mass);
        // Eker et al. 2018 six-piece MLR and Demircan and Kahraman 1991 radii,
        // the same relations as the base-physics atoms.
        let ms_l = if mass > self.mlr_top_msun {
            sse_atoms::star::luminosity_solar(self.mlr_top_msun)
                * m::pow(mass / self.mlr_top_msun, self.mlr_top_slope)
        } else {
            sse_atoms::star::luminosity_solar(mass)
        };
        let ms_r = sse_atoms::star::radius_solar(mass);
        let teff = |l: f64, r: f64| sse_core::sheet::T_SUN * m::pow(l / (r * r), 0.25);
        if age_gyr < 0.0 {
            return StarState {
                phase: "unborn".into(),
                mass_msun: mass,
                luminosity_lsun: 0.0,
                teff_k: 0.0,
                age_gyr,
            };
        }
        if age_gyr < life {
            return StarState {
                phase: "main sequence".into(),
                mass_msun: mass,
                luminosity_lsun: ms_l,
                teff_k: teff(ms_l, ms_r),
                age_gyr,
            };
        }
        if age_gyr < life * (1.0 + self.giant_share) {
            let l = ms_l * self.giant_l_factor + self.giant_l_floor_lsun;
            let cool = l <= self.cool_limit_lsun;
            return StarState {
                phase: if cool { "giant" } else { "hot supergiant" }.into(),
                mass_msun: mass,
                luminosity_lsun: l,
                teff_k: if cool {
                    self.giant_teff_k
                } else {
                    teff(ms_l, ms_r)
                },
                age_gyr,
            };
        }
        let cool = age_gyr - life * (1.0 + self.giant_share);
        if mass < self.ns_min_msun {
            let wd = self.wd_a * mass + self.wd_b;
            // Mestel-like cooling: L ~ 1e-3 at 1 Gyr, falling as t^-1.4.
            let l = 1.0e-3 * m::pow((cool / 1.0).max(1.0e-3), -1.4);
            return StarState {
                phase: "white dwarf".into(),
                mass_msun: wd,
                luminosity_lsun: l,
                teff_k: teff(l, 0.0126),
                age_gyr,
            };
        }
        if mass < self.bh_min_msun {
            return StarState {
                phase: "neutron star".into(),
                mass_msun: self.ns_mass_msun,
                luminosity_lsun: 0.0,
                teff_k: 0.0,
                age_gyr,
            };
        }
        StarState {
            phase: "black hole".into(),
            mass_msun: self.bh_share * mass,
            luminosity_lsun: 0.0,
            teff_k: 0.0,
            age_gyr,
        }
    }
}

impl<'a> Stars<'a> {
    /// Rotation angle of the cell grid at a time (Myr).
    fn grid_angle(&self, t_myr: f64) -> f64 {
        grid_angle(&self.params, t_myr)
    }
    /// The cell holding a galactic position at a time (co-rotating grid).
    pub fn cell_at(&self, xyz: [f64; 3], t_myr: f64) -> [i64; 3] {
        let a = -self.grid_angle(t_myr);
        let (x, y) = (
            xyz[0] * m::cos(a) - xyz[1] * m::sin(a),
            xyz[0] * m::sin(a) + xyz[1] * m::cos(a),
        );
        let c = self.params.cell_pc;
        [
            m::floor(x / c) as i64,
            m::floor(y / c) as i64,
            m::floor(xyz[2] / c) as i64,
        ]
    }
    /// Galactic position of a point given in grid coordinates at a time.
    pub fn from_grid(&self, g: [f64; 3], t_myr: f64) -> [f64; 3] {
        let a = self.grid_angle(t_myr);
        [
            g[0] * m::cos(a) - g[1] * m::sin(a),
            g[0] * m::sin(a) + g[1] * m::cos(a),
            g[2],
        ]
    }
    pub fn frame_of(&self, t_myr: f64) -> i64 {
        m::floor(t_myr / self.params.frame_myr) as i64
    }
    pub(crate) fn cell_address(&self, cell: [i64; 3]) -> Result<Address, String> {
        self.galaxy
            .child_key("cell", &format!("{}_{}_{}", cell[0], cell[1], cell[2]))
            .map_err(|e| format!("{e:?}"))
    }
    /// Expected systems in a cell at a frame (the envelope at its centre).
    pub fn expected(&self, cell: [i64; 3], frame: i64) -> f64 {
        let c = self.params.cell_pc;
        let t0 = frame as f64 * self.params.frame_myr;
        let g = [
            (cell[0] as f64 + 0.5) * c,
            (cell[1] as f64 + 0.5) * c,
            (cell[2] as f64 + 0.5) * c,
        ];
        self.density_at(g, t0) * c * c * c
    }

    /// Systems per pc³ at a point of the grid (pc) at `t0` (Myr).
    pub(crate) fn density_at(&self, g: [f64; 3], t0: f64) -> f64 {
        let [x, y, z] = self.from_grid(g, t0);
        let t_gyr = t0 / 1000.0;
        self.env.systems_pc3(Place {
            r_kpc: m::hypot(x, y) / 1000.0,
            phi: m::atan2(y, x),
            z_pc: z,
            t_gyr,
        })
    }
    /// GH4: whether this galaxy holds the remains of others.
    pub(crate) fn has_remains(&self) -> bool {
        self.env
            .galaxy
            .assembled
            .as_ref()
            .is_some_and(|a| !a.components.is_empty() && a.shapes_with.is_some())
    }

    /// GH4: each component's systems per pc³ at a point of the grid (pc) at
    /// `t0` (Myr).
    pub(crate) fn remains_weights_at(&self, g: [f64; 3], t0: f64) -> Vec<(usize, f64)> {
        if !self.has_remains() {
            return vec![];
        }
        let [x, y, z] = self.from_grid(g, t0);
        let k = 1.0 / self.env.params.mean_star_msun;
        self.env
            .remains_at(Place {
                r_kpc: m::hypot(x, y) / 1000.0,
                phi: m::atan2(y, x),
                z_pc: z,
                t_gyr: t0 / 1000.0,
            })
            .into_iter()
            .map(|(i, d)| (i, d * k))
            .collect()
    }

    /// GH4: the remains' systems per pc³ at a point of the grid.
    pub(crate) fn remains_density_at(&self, g: [f64; 3], t0: f64) -> f64 {
        self.remains_weights_at(g, t0).iter().map(|w| w.1).sum()
    }

    /// Whether this galaxy has a bulge to draw.
    pub(crate) fn has_bulge(&self) -> bool {
        self.env.galaxy.now.bulge > 0.0 && self.env.galaxy.bulge_size[0] > 0.0
    }

    /// The bulge's systems per pc³ at a point of the grid (pc) at `t0` (Myr).
    pub(crate) fn bulge_density_at(&self, g: [f64; 3], t0: f64) -> f64 {
        let [x, y, z] = self.from_grid(g, t0);
        self.env.bulge_pc3(Place {
            r_kpc: m::hypot(x, y) / 1000.0,
            phi: m::atan2(y, x),
            z_pc: z,
            t_gyr: t0 / 1000.0,
        }) / self.env.params.mean_star_msun
    }

    /// The bulge's cumulative births as it stood at `t_now_gyr`
    /// (`Galaxy::bulge_births_at`).
    pub(crate) fn bulge_cdf(&self, t_now_gyr: f64) -> (Vec<(f64, f64)>, f64) {
        let mut cdf = vec![(
            self.env.disc.t_start_gyr.min(self.env.galaxy.t_start_gyr),
            0.0,
        )];
        let mut acc = 0.0;
        for (t, b) in &self.env.galaxy.bulge_births_at(t_now_gyr) {
            acc += b;
            cdf.push((*t, acc));
        }
        (cdf, acc)
    }

    /// A bulge star: everything but its mass and birth, as a disc star's
    /// (its gas at its radius when it was born, an orbit through where it is:
    /// fudge register "stars.rs bulge orbits"), with the bulge's key.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn complete_bulge(
        &self,
        d: &Draws,
        i: &dyn Fn(u64) -> u64,
        addr: &Address,
        cell: [i64; 3],
        frame: i64,
        k: u64,
        mass: f64,
        birth: f64,
    ) -> Result<StarSystem, String> {
        let mut s = self.complete(d, i, addr, cell, frame, k, mass, birth)?;
        s.address = addr
            .child_key("sys", &format!("f{frame}b{k}"))
            .map_err(|e| format!("{e:?}"))?;
        s.in_bulge = true;
        Ok(s)
    }

    /// The bulge's stars of a cell.
    pub(crate) fn draw_bulge(
        &self,
        cell: [i64; 3],
        frame: i64,
        class: Option<usize>,
    ) -> Result<Vec<StarSystem>, String> {
        if self.params.key_version < 2 || !self.has_bulge() {
            return Ok(vec![]);
        }
        crate::star_tree::TreeWalk::bulge(self, frame).draw(cell, class)
    }

    /// GH4: the earliest birth any component holds (every remains history
    /// starts there, so the class grid is one for the walk).
    fn remains_first_gyr(&self) -> f64 {
        self.env
            .galaxy
            .assembled
            .as_ref()
            .map_or(0.0, |a| {
                a.components
                    .iter()
                    .filter_map(|c| c.births.first().map(|b| b.t_gyr))
                    .fold(f64::INFINITY, f64::min)
            })
            .min(self.env.disc.t_start_gyr)
            - 0.05
    }

    /// GH4: one component's cumulative births up to `t_now_gyr`.
    pub(crate) fn component_cdf(&self, i: usize, t_now_gyr: f64) -> (Vec<(f64, f64)>, f64) {
        let mut cdf = vec![(self.remains_first_gyr(), 0.0)];
        let mut acc = 0.0;
        if let Some(c) = self
            .env
            .galaxy
            .assembled
            .as_ref()
            .and_then(|a| a.components.get(i))
        {
            for b in c.births.iter().filter(|b| b.t_gyr <= t_now_gyr) {
                acc += b.stars;
                cdf.push((b.t_gyr, acc));
            }
        }
        (cdf, acc)
    }

    /// GH4: the remains' births at a point of the grid: each component's
    /// share of the stars there times its own history.
    pub(crate) fn remains_cdf_at(&self, g: [f64; 3], t0: f64) -> (Vec<(f64, f64)>, f64) {
        let t_now = t0 / 1000.0;
        let w = self.remains_weights_at(g, t0);
        let total: f64 = w.iter().map(|x| x.1).sum();
        let first = self.remains_first_gyr();
        let steps = 200;
        type Cdf = (f64, Vec<(f64, f64)>, f64);
        let cdfs: Vec<Cdf> = w
            .iter()
            .map(|(i, d)| {
                let (c, a) = self.component_cdf(*i, t_now);
                (*d, c, a)
            })
            .collect();
        let mut out = Vec::with_capacity(steps + 1);
        for k in 0..=steps {
            let t = first + (t_now - first) * k as f64 / steps as f64;
            let mut v = 0.0;
            for (d, c, a) in &cdfs {
                if *a > 0.0 && total > 0.0 {
                    let by = birth_share(c, *a, t);
                    v += d / total * by;
                }
            }
            out.push((t, v));
        }
        let acc = out.last().map_or(0.0, |p| p.1);
        (out, acc)
    }

    /// GH4: a halo star of component `comp`: everything but its mass and
    /// birth, with the abundances of the gas it was born from in its own
    /// galaxy and an orbit through the halo (fudge F49).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn complete_remains(
        &self,
        d: &Draws,
        i: &dyn Fn(u64) -> u64,
        addr: &Address,
        cell: [i64; 3],
        frame: i64,
        k: u64,
        comp: usize,
        mass: f64,
        birth: f64,
    ) -> Result<StarSystem, String> {
        let mut s = self.complete(d, i, addr, cell, frame, k, mass, birth)?;
        let a = self
            .env
            .galaxy
            .assembled
            .as_ref()
            .ok_or("no remains here")?;
        let c = a.components.get(comp).ok_or("no such component")?;
        let b = c
            .births
            .iter()
            .min_by(|x, y| (x.t_gyr - birth).abs().total_cmp(&(y.t_gyr - birth).abs()))
            .ok_or("a component with no births")?;
        s.feh = b.feh + b.feh_sd * d.normal(i(13));
        s.alpha_fe = b.alpha_fe;
        s.c_o = 0.55 * m::pow(10.0, b.cn_alpha);
        s.r_fe = b.r_fe;
        s.k40_fe = b.k40_fe;
        // Its orbit: the closed form, with the halo's reach (F49). It turns
        // with the disc as far as its galaxy's orbit did, at half the
        // circular speed at most.
        let t0 = frame as f64 * self.params.frame_myr;
        let [x, y, z] = s.orbit.xyz(t0);
        let r0 = m::hypot(x, y).max(1.0);
        let (v_c, _) = self.env.circular(r0 / 1000.0);
        let v = v_c * KMS_PC_MYR;
        let rayleigh = |u: f64| m::sqrt(-2.0 * m::ln(u.max(1.0e-300)));
        let reach = 500.0 * (c.apo_kpc - c.peri_kpc).max(0.0);
        let amp_r = (reach * rayleigh(d.unit(i(17))) * 0.5).min(0.9 * r0);
        let phase_r = 2.0 * m::PI * d.unit(i(18));
        let r_guide = r0 - amp_r * m::cos(phase_r);
        let omega = 0.5 * c.prograde * v / r_guide.max(1.0);
        let kappa = m::sqrt(2.0) * v / r_guide.max(1.0);
        let r3 = m::sqrt(r0 * r0 + z * z).max(1.0);
        let nu = v / r3;
        let phi_now = m::atan2(y, x);
        let phi0 = if kappa > 0.0 {
            phi_now + 2.0 * omega / kappa * amp_r / r_guide.max(1.0) * m::sin(phase_r)
        } else {
            phi_now
        };
        let amp_z = (1000.0 * c.apo_kpc * 0.5 * d.unit(i(19))).max(z.abs() * (1.0 + 1.0e-12));
        let base = m::acos((z / amp_z).clamp(-1.0, 1.0));
        let phase_z = if d.unit(i(20)) < 0.5 { base } else { -base };
        s.orbit = Orbit {
            r_guide_pc: r_guide,
            amp_r_pc: amp_r,
            amp_z_pc: amp_z,
            phase_r,
            phase_z,
            phi0,
            t0_myr: t0,
            omega,
            kappa,
            nu,
        };
        let (op, rp) = a.shapes_with.as_ref().ok_or("no remains here")?;
        let kind = self.env.remains.kind_at(
            &a.components,
            op,
            rp,
            [x / 1000.0, y / 1000.0, z / 1000.0],
            t0 / 1000.0,
            comp,
        );
        s.address = addr
            .child_key("sys", &format!("f{frame}h{k}"))
            .map_err(|e| format!("{e:?}"))?;
        s.born_in = Some(BornIn {
            galaxy: c.from.clone(),
            fell_gyr: c.fell_gyr,
            torn_gyr: c.torn_gyr,
            merged_gyr: c.merged_gyr,
            galaxy_stars_msun: c.stars_msun,
            remains: kind.into(),
            says: format!("born in {}", crate::origin::born_in_words(c, t0 / 1000.0)),
        });
        Ok(s)
    }

    /// GH4: the remains' stars of a cell (none in a galaxy that tore none
    /// apart).
    pub(crate) fn draw_remains(
        &self,
        cell: [i64; 3],
        frame: i64,
        class: Option<usize>,
    ) -> Result<Vec<StarSystem>, String> {
        if self.params.key_version < 2 || !self.has_remains() {
            return Ok(vec![]);
        }
        crate::star_tree::TreeWalk::remains(self, frame).draw(cell, class)
    }

    /// Every system drawn for a cell and frame, before pins.
    pub(crate) fn draw_all(&self, cell: [i64; 3], frame: i64) -> Result<Vec<StarSystem>, String> {
        if self.params.key_version >= 2 {
            return self.draw_tree(cell, frame, None);
        }
        let p = &self.params;
        let addr = self.cell_address(cell)?;
        let count = Draws::new(
            self.seed,
            addr.clone(),
            Stream::new(&format!("cosmos.stars.count.v{}", p.key_version))
                .map_err(|e| e.to_string())?,
        );
        let n = crate::haloes::poisson(&count, frame as u64, self.expected(cell, frame));
        let d = Draws::new(
            self.seed,
            addr.clone(),
            Stream::new(&format!("cosmos.stars.v{}", p.key_version)).map_err(|e| e.to_string())?,
        );
        let t0 = frame as f64 * p.frame_myr;
        let (cdf, acc) = self.birth_cdf(cell, t0 / 1000.0);
        let mut out = vec![];
        for k in 0..n {
            let i = |j: u64| {
                (frame as u64)
                    .wrapping_mul(1 << 20)
                    .wrapping_add(k * 64 + j)
            };
            let birth = birth_from(&cdf, acc, d.unit(i(3)), t0 / 1000.0);
            let mass = imf_quantile(p, d.unit(i(4)));
            out.push(self.complete(&d, &i, &addr, cell, frame, k, mass, birth)?);
        }
        Ok(out)
    }

    /// The ring's cumulative star formation at a cell's radius up to
    /// `t_now_gyr`, for birth times.
    pub(crate) fn birth_cdf(&self, cell: [i64; 3], t_now_gyr: f64) -> (Vec<(f64, f64)>, f64) {
        let c = self.params.cell_pc;
        let rc = m::hypot((cell[0] as f64 + 0.5) * c, (cell[1] as f64 + 0.5) * c) / 1000.0;
        self.birth_cdf_at(rc, t_now_gyr)
    }

    pub(crate) fn birth_cdf_at(&self, rc: f64, t_now_gyr: f64) -> (Vec<(f64, f64)>, f64) {
        let steps = 200;
        let t_first = self.env.disc.t_start_gyr;
        let mut cdf = Vec::with_capacity(steps + 1);
        let mut acc = 0.0;
        cdf.push((t_first, 0.0));
        for k in 1..=steps {
            let t = t_first + (t_now_gyr - t_first) * k as f64 / steps as f64;
            acc += self.env.surface(rc, t).sfr.max(0.0);
            cdf.push((t, acc));
        }
        (cdf, acc)
    }

    /// Everything about a system but its mass and birth, drawn with the
    /// star's own keys `i(0..=2)` and `i(5..=20)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn complete(
        &self,
        d: &Draws,
        i: &dyn Fn(u64) -> u64,
        addr: &Address,
        cell: [i64; 3],
        frame: i64,
        k: u64,
        mass: f64,
        birth: f64,
    ) -> Result<StarSystem, String> {
        let p = &self.params;
        let t0 = frame as f64 * p.frame_myr;
        let t_now_gyr = t0 / 1000.0;
        let c = p.cell_pc;
        let rc = m::hypot((cell[0] as f64 + 0.5) * c, (cell[1] as f64 + 0.5) * c) / 1000.0;
        let [x, y, z] = self.from_grid(
            [
                (cell[0] as f64 + d.unit(i(0))) * c,
                (cell[1] as f64 + d.unit(i(1))) * c,
                (cell[2] as f64 + d.unit(i(2))) * c,
            ],
            t0,
        );
        let mut companions = vec![];
        if d.unit(i(5)) < p.multiplicity_at(mass) {
            let n_c = if d.unit(i(6)) < p.triple_share { 2 } else { 1 };
            for j in 0..n_c {
                let q = p.q_min + (1.0 - p.q_min) * d.unit(i(7 + j));
                let la = p.log_a_mean + p.log_a_sd * d.normal(i(10 + j));
                companions.push(Companion {
                    mass_msun: mass * q,
                    a_au: m::pow(10.0, la),
                });
            }
            companions.sort_by(|a, b| a.a_au.total_cmp(&b.a_au));
        }
        // The gas's mix at its birth ring and time (`local`'s arithmetic,
        // without the hazards it also works out).
        let sb = self.env.surface(rc, birth);
        let feh = sb.feh + p.feh_scatter * d.normal(i(13));
        let cluster = {
            let u = d.unit(i(16));
            let (a, b) = (1.0 / p.cluster_min, 1.0 / p.cluster_max);
            1.0 / (a - u * (a - b))
        };
        // Orbit through (x, y, z) at t0.
        let r0 = m::hypot(x, y);
        let phi_now = m::atan2(y, x);
        let age = (t_now_gyr - birth).max(0.0);
        let grow = m::pow((age / 10.0).max(1.0e-3), p.sigma_exponent);
        let sr = (p.sigma_r_10 * grow).max(p.sigma_floor) * KMS_PC_MYR;
        let sz = (p.sigma_z_10 * grow).max(p.sigma_floor) * KMS_PC_MYR;
        // Angular and epicyclic frequencies from the galaxy's rotation
        // curve (DM4): κ² = 2 Ω² (1 + d ln v / d ln r).
        let (v_c, slope) = self.env.circular(r0 / 1000.0);
        let omega = v_c * KMS_PC_MYR / r0;
        let kappa = m::sqrt(2.0 * (1.0 + slope).max(0.05)) * omega;
        let nu = m::sqrt(4.0 * m::PI * G_PC3 * p.midplane_density);
        let rayleigh = |u: f64| m::sqrt(-2.0 * m::ln(u.max(1.0e-300)));
        let amp_r = sr / kappa * rayleigh(d.unit(i(17)));
        let phase_r = 2.0 * m::PI * d.unit(i(18));
        let r_guide = r0 - amp_r * m::cos(phase_r);
        let phi0 = phi_now + 2.0 * omega / kappa * amp_r / r_guide * m::sin(phase_r);
        let amp_z = (sz / nu * rayleigh(d.unit(i(19)))).max(z.abs() * (1.0 + 1.0e-12));
        let base = m::acos((z / amp_z).clamp(-1.0, 1.0));
        let phase_z = if d.unit(i(20)) < 0.5 { base } else { -base };
        Ok(StarSystem {
            address: addr
                .child_key("sys", &format!("f{frame}k{k}"))
                .map_err(|e| format!("{e:?}"))?,
            cell,
            frame,
            slot: k,
            birth_gyr: birth,
            mass_msun: mass,
            companions,
            feh,
            alpha_fe: sb.alpha_h - sb.feh,
            c_o: 0.55 * m::pow(10.0, sb.cn_h - sb.alpha_h),
            r_fe: sb.r_h - sb.feh,
            k40_fe: sb.k40_h - sb.feh,
            cluster_mass_msun: cluster,
            orbit: Orbit {
                r_guide_pc: r_guide,
                amp_r_pc: amp_r,
                amp_z_pc: amp_z,
                phase_r,
                phase_z,
                phi0,
                t0_myr: t0,
                omega,
                kappa,
                nu,
            },
            pinned: false,
            born_in: None,
            in_bulge: false,
        })
    }

    /// The systems of a cell in a frame: fill drawn from the envelope, with
    /// the pins that are in the cell at the frame's start claiming their
    /// share first.
    pub fn cell(
        &self,
        cell: [i64; 3],
        frame: i64,
        pins: &[StarSystem],
    ) -> Result<Vec<StarSystem>, String> {
        let fill = self.draw_all(cell, frame)?;
        let rem = self.draw_remains(cell, frame, None)?;
        let bul = self.draw_bulge(cell, frame, None)?;
        Ok(self.with_pins(cell, frame, pins, fill, rem, bul))
    }

    /// Several cells in one frame, each exactly as `cell` gives it. The
    /// cells share one walk of the census tree, so the blocks and splits
    /// above them are worked out once (a voyage reads 27 cells).
    pub fn cells(
        &self,
        cells: &[[i64; 3]],
        frame: i64,
        pins: &[StarSystem],
    ) -> Result<Vec<Vec<StarSystem>>, String> {
        let walk =
            (self.params.key_version >= 2).then(|| crate::star_tree::TreeWalk::new(self, frame));
        let rwalk = (self.params.key_version >= 2 && self.has_remains())
            .then(|| crate::star_tree::TreeWalk::remains(self, frame));
        let bwalk = (self.params.key_version >= 2 && self.has_bulge())
            .then(|| crate::star_tree::TreeWalk::bulge(self, frame));
        cells
            .iter()
            .map(|&c| {
                let fill = match &walk {
                    Some(w) => w.draw(c, None)?,
                    None => self.draw_all(c, frame)?,
                };
                let rem = match &rwalk {
                    Some(w) => w.draw(c, None)?,
                    None => vec![],
                };
                let bul = match &bwalk {
                    Some(w) => w.draw(c, None)?,
                    None => vec![],
                };
                Ok(self.with_pins(c, frame, pins, fill, rem, bul))
            })
            .collect()
    }

    fn with_pins(
        &self,
        cell: [i64; 3],
        frame: i64,
        pins: &[StarSystem],
        mut fill: Vec<StarSystem>,
        mut remains: Vec<StarSystem>,
        mut bulge: Vec<StarSystem>,
    ) -> Vec<StarSystem> {
        let t0 = frame as f64 * self.params.frame_myr;
        let here: Vec<&StarSystem> = pins
            .iter()
            .filter(|s| self.cell_at(s.orbit.xyz(t0), t0) == cell)
            .collect();
        // Pins drawn from this cell and frame keep their slot; a disc pin
        // that moved here takes a disc star's place, a halo pin a halo
        // star's, a bulge pin a bulge star's.
        let own = |kind: Kind| -> Vec<u64> {
            here.iter()
                .filter(|s| s.cell == cell && s.frame == frame && s.kind() == kind)
                .map(|s| s.slot)
                .collect()
        };
        for (list, kind) in [
            (&mut fill, Kind::Disc),
            (&mut remains, Kind::Halo),
            (&mut bulge, Kind::Bulge),
        ] {
            let mine = own(kind);
            list.retain(|s| !mine.contains(&s.slot));
            let of_kind = here.iter().filter(|s| s.kind() == kind).count();
            let keep = list.len().saturating_sub(of_kind - mine.len());
            list.truncate(keep);
        }
        let mut out: Vec<StarSystem> = here.into_iter().cloned().collect();
        out.extend(fill);
        out.extend(remains);
        out.extend(bulge);
        out.sort_by(|a, b| a.address.cmp(&b.address));
        out
    }

    /// The fill system at `address` (`<galaxy>/cell:<x>_<y>_<z>/sys:f<frame>k<slot>`)
    /// as today's rules draw it, or `None` when the cell holds no such slot.
    pub fn draw_one(&self, address: &Address) -> Result<Option<StarSystem>, String> {
        let text = address.to_string();
        let (head, sys) = text
            .rsplit_once("/sys:")
            .ok_or_else(|| format!("{text}: not a star system address"))?;
        let (cell_head, key) = head
            .rsplit_once("/cell:")
            .ok_or_else(|| format!("{text}: no cell"))?;
        if cell_head != self.galaxy.to_string() {
            return Err(format!("{text}: not in galaxy {}", self.galaxy));
        }
        let n: Vec<i64> = key.split('_').filter_map(|x| x.parse().ok()).collect();
        let halo = sys.contains('h');
        let in_bulge = sys.contains('b');
        let sep = if halo {
            'h'
        } else if in_bulge {
            'b'
        } else {
            'k'
        };
        let (f, k) = sys
            .strip_prefix('f')
            .and_then(|x| x.split_once(sep))
            .ok_or_else(|| {
                format!("{text}: system key is f<frame>k<slot>, f<frame>h<slot> or f<frame>b<slot>")
            })?;
        let (frame, slot): (i64, u64) = match (f.parse(), k.parse()) {
            (Ok(f), Ok(k)) => (f, k),
            _ => return Err(format!("{text}: system key is f<frame>k<slot>")),
        };
        if n.len() != 3 {
            return Err(format!("{text}: cell key is <x>_<y>_<z>"));
        }
        let cell = [n[0], n[1], n[2]];
        let all = if halo {
            let classes = self.params.class_edges_lsun.len() as u64 + 1;
            self.draw_remains(cell, frame, Some((slot % classes) as usize))?
        } else if in_bulge {
            let classes = self.params.class_edges_lsun.len() as u64 + 1;
            self.draw_bulge(cell, frame, Some((slot % classes) as usize))?
        } else if self.params.key_version >= 2 {
            // The slot names its class (count tree slots interleave them).
            let classes = self.params.class_edges_lsun.len() as u64 + 1;
            self.draw_tree(cell, frame, Some((slot % classes) as usize))?
        } else {
            self.draw_all(cell, frame)?
        };
        Ok(all.into_iter().find(|s| s.slot == slot))
    }

    /// Look at a fill system: it becomes permanent.
    pub fn promote(&self, s: &StarSystem) -> StarSystem {
        StarSystem {
            pinned: true,
            ..s.clone()
        }
    }

    /// A system at any time: closed form, so the same in any order.
    pub fn at(&self, s: &StarSystem, t_myr: f64) -> SystemAt {
        SystemAt {
            xyz_pc: s.orbit.xyz(t_myr),
            state: self.params.state(s.mass_msun, t_myr / 1000.0 - s.birth_gyr),
        }
    }

    /// What the formation suite inherits (G0's `SystemBirth`).
    pub fn birth(&self, s: &StarSystem) -> crate::SystemBirth {
        let total: f64 = s.mass_msun + s.companions.iter().map(|c| c.mass_msun).sum::<f64>();
        crate::SystemBirth {
            address: s.address.clone(),
            population: if s.born_in.is_some() {
                "stellar halo".into()
            } else if s.in_bulge {
                "bulge".into()
            } else if s.birth_gyr < 3.0 {
                "thick disc".into()
            } else {
                "thin disc".into()
            },
            mass_msun: total,
            birth_gyr: s.birth_gyr,
            feh: s.feh,
            alpha_fe: s.alpha_fe,
            c_o: s.c_o,
            mg_si: 1.0,
            r_process_rel: m::pow(10.0, s.r_fe + s.feh),
            k40_rel: m::pow(10.0, s.k40_fe + s.feh),
            al26_rel: 1.0,
            cluster_mass_msun: s.cluster_mass_msun,
            cluster_fuv_g0: 10.0 * m::pow(s.cluster_mass_msun / 1000.0, 0.5),
        }
    }
}
