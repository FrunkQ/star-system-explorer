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
    pub rotation_kms: f64,
    /// Birth cluster mass function ~ M^-2 between these (solar masses).
    pub cluster_min: f64,
    pub cluster_max: f64,
    /// Cells co-rotate with the galaxy at this angular speed (km/s/kpc;
    /// the circular speed at the Sun's radius), so nearby stars drift
    /// through them at their peculiar speeds, not at 220 km/s.
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
            rotation_kms: 220.0,
            cluster_min: 50.0,
            cluster_max: 1.0e5,
            cell_rotation_kms_kpc: 26.83,
            key_version: 2,
            class_edges_lsun: vec![
                0.1, 1.0, 10.0, 100.0, 300.0, 1.0e3, 3.0e3, 1.0e4, 3.0e4, 1.0e5, 3.0e5,
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

const KMS_PC_MYR: f64 = 1.022_712;
const G_PC3: f64 = 4.498_502e-3;

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

pub(crate) fn imf_quantile(p: &StarParams, u: f64) -> f64 {
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
        let ms_l = sse_atoms::star::luminosity_solar(mass);
        let ms_r = sse_atoms::star::radius_solar(mass);
        let teff = |l: f64, r: f64| 5772.0 * m::pow(l / (r * r), 0.25);
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
            return StarState {
                phase: "giant".into(),
                mass_msun: mass,
                luminosity_lsun: l,
                teff_k: self.giant_teff_k,
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
        self.params.cell_rotation_kms_kpc * KMS_PC_MYR / 1000.0 * t_myr
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
        let omega = p.rotation_kms * KMS_PC_MYR / r0;
        let kappa = m::sqrt(2.0) * omega;
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
        let t0 = frame as f64 * self.params.frame_myr;
        let here: Vec<&StarSystem> = pins
            .iter()
            .filter(|s| self.cell_at(s.orbit.xyz(t0), t0) == cell)
            .collect();
        let mut fill = self.draw_all(cell, frame)?;
        // Pins drawn from this cell and frame keep their slot.
        let own: Vec<u64> = here
            .iter()
            .filter(|s| s.cell == cell && s.frame == frame)
            .map(|s| s.slot)
            .collect();
        fill.retain(|s| !own.contains(&s.slot));
        let foreign = here.len() - own.len();
        let keep = fill.len().saturating_sub(foreign);
        fill.truncate(keep);
        let mut out: Vec<StarSystem> = here.into_iter().cloned().collect();
        out.extend(fill);
        out.sort_by(|a, b| a.address.cmp(&b.address));
        Ok(out)
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
        let (f, k) = sys
            .strip_prefix('f')
            .and_then(|x| x.split_once('k'))
            .ok_or_else(|| format!("{text}: system key is f<frame>k<slot>"))?;
        let (frame, slot): (i64, u64) = match (f.parse(), k.parse()) {
            (Ok(f), Ok(k)) => (f, k),
            _ => return Err(format!("{text}: system key is f<frame>k<slot>")),
        };
        if n.len() != 3 {
            return Err(format!("{text}: cell key is <x>_<y>_<z>"));
        }
        let all = self.draw_all([n[0], n[1], n[2]], frame)?;
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
            population: if s.birth_gyr < 3.0 {
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
