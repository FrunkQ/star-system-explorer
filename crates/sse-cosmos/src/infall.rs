//! GH2 (galaxy histories design 7.3): what happens to a halo after it falls
//! into a bigger one. Each progenitor of the host's assembly tree (GH1) is
//! one test body on an orbit in the host's smooth, slowly growing potential
//! (DM4's structure: dark halo, hot gas, disc, bulge, black hole), and four
//! processes act on it at every step, each with its own strength from its
//! governing ratio. There are no gates: fate is a label read afterwards.
//!
//! - **Sinking**: Chandrasekhar's dynamical friction (Chandrasekhar 1943,
//!   doi:10.1086/144517), Coulomb logarithm ln(1 + M/m).
//! - **Tidal stripping**: the satellite loses the mass outside its tidal
//!   radius on its own dynamical time (the SatGen method, Jiang et al. 2021,
//!   doi:10.1093/mnras/staa4034; stripping from the outside in, Peñarrubia
//!   et al. 2008, doi:10.1086/523686). Its stars start to leave only when
//!   about nine tenths of its dark matter has gone (Peñarrubia et al. 2008),
//!   fudge F46 for the shape after that.
//! - **Gas stripping**: the host's hot gas pushes on the satellite's gas as
//!   it moves (ram pressure ρ v²); once that beats the satellite's grip on
//!   its gas, 2πG Σ_tot Σ_gas at its disc scale (Gunn and Gott 1972,
//!   doi:10.1086/151605), its star formation stops. Small ones stop fast,
//!   big ones keep going (Fillingham et al. 2015, doi:10.1093/mnras/stv2058).
//! - **Shock heating**: each pass through the host's disc heats the
//!   satellite impulsively (Garrison-Kimmel et al. 2017,
//!   doi:10.1093/mnras/stx1710); heat speeds stripping, and a satellite
//!   heated past its own binding is torn apart.
//!
//! The orbit is a fixed-step kick-drift-kick leapfrog in the kernel's pinned
//! maths, written fresh from the SatGen paper's description, not from its
//! code. It is not a particle simulation: bodies do not pull on each other.
//!
//! The three cuts of design section 10, each kept inside its condition:
//! 1. A body is stepped only from its infall.
//! 2. Its step is sized from its own orbit near closest approach, worked
//!    out once at infall from its initial conditions and the host at that
//!    time, so it is a function of the body alone: the same body takes the
//!    same path at Chart and at Survey and on every machine.
//! 3. The host's pull, density, spread of speeds and hot gas are read from
//!    one radial table per stored time slice ([`Host`]), built once and used
//!    at every level, and checked against the direct formulas in the tests.
//!    The drag's speed factor is read from one fixed table too.

use crate::cosmology::Cosmology;
use crate::galaxy::{Galaxy, GalaxyParams, GalaxyState, G_KPC};
use crate::haloes::Halo;
use crate::structure::Structure;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Kilometres per second in kiloparsecs per gigayear (the factor `origin`
/// already uses).
pub fn kms() -> f64 {
    crate::origin::kms_kpc_gyr()
}

/// The processes' settings (in `SkyParams.satellites`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct OrbitParams {
    /// Infall speed over the host's virial speed at infall, drawn uniformly
    /// (Jiang, Cole, Sawala and Frenk 2015, MNRAS 448, 1674: the
    /// distribution peaks near 1.2 with most between 0.9 and 1.4).
    pub speed_range: [f64; 2],
    /// Radial share of the infall velocity (v_r / v), drawn uniformly
    /// (same source: infall orbits are mostly radial, few are circular).
    pub radial_range: [f64; 2],
    /// Steps taken over the time a body spends near closest approach
    /// (pericentre distance over speed there). About 25 keeps leapfrog's
    /// energy error under a thousandth over a Hubble time (tested).
    pub steps_near_peri: f64,
    /// The smallest and largest step allowed, Gyr.
    pub step_range_gyr: [f64; 2],
    /// Host time slices, Gyr apart.
    pub slice_gyr: f64,
    /// Radial table: points from `table_kpc[0]` to `table_kpc[1]`.
    pub table_points: u32,
    pub table_kpc: [f64; 2],
    /// Tidal stripping rate: the mass outside the tidal radius goes on this
    /// many of the satellite's own dynamical times (SatGen's α = 0.55, a
    /// fitted number, fudge F46).
    pub strip_alpha: f64,
    /// Stars start to leave once the bound dark matter falls below this
    /// share of its infall mass (Peñarrubia et al. 2008: about a tenth).
    pub stars_leave_below: f64,
    /// Torn apart: bound mass under this share of the infall mass, or heat
    /// past the satellite's own binding energy.
    pub disrupt_below: f64,
    /// Reached the centre: closer than this share of the host's r200.
    pub merge_radius_frac: f64,
    /// Disc passes count for shock heating inside this many disc scale
    /// lengths.
    pub disc_reach: f64,
    /// Remains older than this are phase-mixed ("mixed remains"), Gyr.
    pub mix_gyr: f64,
    /// Remains on an orbit with closest over farthest approach under this
    /// are shells (radial orbits), above it streams.
    pub shell_below: f64,
}

impl Default for OrbitParams {
    fn default() -> Self {
        OrbitParams {
            speed_range: [0.9, 1.4],
            radial_range: [0.3, 1.0],
            steps_near_peri: 25.0,
            step_range_gyr: [1.0e-4, 0.01],
            slice_gyr: 0.25,
            table_points: 96,
            table_kpc: [0.05, 2000.0],
            strip_alpha: 0.55,
            stars_leave_below: 0.1,
            disrupt_below: 1.0e-3,
            merge_radius_frac: 0.01,
            disc_reach: 5.0,
            mix_gyr: 5.0,
            shell_below: 0.1,
        }
    }
}

/// One host time slice: quantities on the fixed log-radius grid.
#[derive(Clone, Debug)]
struct Slice {
    t_gyr: f64,
    r200_kpc: f64,
    /// Inward pull g(r) = v_c²/r, kpc/Gyr².
    pull: Vec<f64>,
    /// Potential, (kpc/Gyr)², zero at the table's outer edge less GM/r beyond.
    phi: Vec<f64>,
    /// Dark matter and hot gas density, M☉/kpc³ (the drag's medium).
    rho: Vec<f64>,
    /// Hot gas density, M☉/kpc³ (ram pressure).
    hot: Vec<f64>,
    /// One-dimensional speed spread, kpc/Gyr.
    sigma: Vec<f64>,
    /// Enclosed mass, M☉, and its log slope.
    mass: Vec<f64>,
    slope: Vec<f64>,
    /// Disc mass and scale length for the disc's surface density.
    disc_msun: f64,
    disc_kpc: f64,
}

/// The host's potential through time.
#[derive(Clone, Debug)]
pub struct Host {
    slices: Vec<Slice>,
    l0: f64,
    dl: f64,
    n: usize,
    /// The grid's radii, kpc.
    radii: Vec<f64>,
    /// G in kpc³ M☉⁻¹ Gyr⁻².
    g: f64,
}

/// What one step reads off the host at a radius.
#[derive(Clone, Copy, Debug)]
pub struct Here {
    pub pull: f64,
    pub phi: f64,
    pub rho: f64,
    pub hot: f64,
    pub sigma: f64,
    pub mass: f64,
    pub slope: f64,
    pub r200: f64,
    pub disc_msun: f64,
    pub disc_kpc: f64,
}

fn state_at(g: &Galaxy, t: f64) -> &GalaxyState {
    let i = g.history.partition_point(|s| s.t_gyr <= t);
    if i == 0 {
        g.history.first().unwrap_or(&g.now)
    } else {
        &g.history[i - 1]
    }
}

impl Host {
    /// Slices of `halo` with its galaxy `galaxy` from `t_from` to `t_to`.
    pub fn new(
        c: &Cosmology,
        gp: &GalaxyParams,
        halo: &Halo,
        galaxy: &Galaxy,
        t_from: f64,
        t_to: f64,
        op: &OrbitParams,
    ) -> Host {
        let n = op.table_points.max(16) as usize;
        let (l0, l1) = (m::ln(op.table_kpc[0]), m::ln(op.table_kpc[1]));
        let dl = (l1 - l0) / (n - 1) as f64;
        let g = G_KPC * kms() * kms();
        let k = m::ceil((t_to - t_from).max(0.0) / op.slice_gyr) as usize;
        let mut slices = Vec::with_capacity(k + 1);
        for j in 0..=k {
            let t = (t_from + op.slice_gyr * j as f64).min(t_to);
            let a = c.scale_of_time(t.max(1.0e-3)).min(1.0);
            let s = Structure::new(
                c,
                &gp.structure,
                &|z| halo.mass_at(z),
                state_at(galaxy, t),
                a,
            );
            slices.push(Self::slice(&s, t, l0, dl, n, g));
            if t >= t_to {
                break;
            }
        }
        Host {
            slices,
            l0,
            dl,
            n,
            radii: (0..n).map(|i| m::exp(l0 + dl * i as f64)).collect(),
            g,
        }
    }

    fn slice(s: &Structure, t: f64, l0: f64, dl: f64, n: usize, g: f64) -> Slice {
        let r = |i: usize| m::exp(l0 + dl * i as f64);
        let mut pull = Vec::with_capacity(n);
        let mut rho = Vec::with_capacity(n);
        let mut hot = Vec::with_capacity(n);
        let mut sigma = Vec::with_capacity(n);
        let mut mass = Vec::with_capacity(n);
        let hot_share = if s.dark_msun > 0.0 {
            s.hot_msun / s.dark_msun
        } else {
            0.0
        };
        for i in 0..n {
            let ri = r(i);
            let v = s.speeds(ri).total * kms();
            pull.push(v * v / ri);
            let d = s.dark_density(ri);
            rho.push(d * (1.0 + hot_share));
            hot.push(d * hot_share);
            // An isothermal sphere's spread at the local circular speed
            // (Binney and Tremaine 2008, eq. 4.103): σ = v_c / √2.
            sigma.push(v / std::f64::consts::SQRT_2);
            mass.push(v * v * ri / g);
        }
        let slope: Vec<f64> = (0..n)
            .map(|i| {
                let (a, b) = (i.saturating_sub(1), (i + 1).min(n - 1));
                (m::ln(mass[b]) - m::ln(mass[a])) / (dl * (b - a) as f64)
            })
            .collect();
        // Φ(r) = −GM/r_out − ∫_r^r_out g dr, trapezium in ln r (dr = r d ln r).
        let mut phi = vec![0.0; n];
        phi[n - 1] = -g * mass[n - 1] / r(n - 1);
        for i in (0..n - 1).rev() {
            phi[i] = phi[i + 1] - 0.5 * (pull[i] * r(i) + pull[i + 1] * r(i + 1)) * dl;
        }
        Slice {
            t_gyr: t,
            r200_kpc: s.r200_kpc,
            pull,
            phi,
            rho,
            hot,
            sigma,
            mass,
            slope,
            disc_msun: s.disc_msun + s.gas_msun,
            disc_kpc: s.disc_kpc,
        }
    }

    /// G in the orbit's units, kpc³ M☉⁻¹ Gyr⁻².
    pub fn g(&self) -> f64 {
        self.g
    }

    /// The host at radius `r` (kpc) and time `t` (Gyr), linear in ln r
    /// within a slice and in time between slices.
    pub fn at(&self, t: f64, r: f64) -> Here {
        let ss = &self.slices;
        let j = ss
            .partition_point(|s| s.t_gyr <= t)
            .clamp(1, ss.len().max(2) - 1);
        let (a, b) = if ss.len() == 1 {
            (&ss[0], &ss[0])
        } else {
            (&ss[j - 1], &ss[j])
        };
        let w = if b.t_gyr > a.t_gyr {
            ((t - a.t_gyr) / (b.t_gyr - a.t_gyr)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let x = ((m::ln(r.max(1.0e-9)) - self.l0) / self.dl).clamp(0.0, (self.n - 1) as f64);
        let i = (m::floor(x) as usize).min(self.n - 2);
        let f = x - i as f64;
        let lin = |v: &[f64]| v[i] + f * (v[i + 1] - v[i]);
        let mix = |p: f64, q: f64| p + w * (q - p);
        // Φ by cubic Hermite in ln r with dΦ/d ln r = g r at the grid points,
        // and the pull from the same cubic, so the force is exactly the
        // potential's slope and a body's energy holds in a still host.
        let (r_i, r_j) = (self.radii[i], self.radii[i + 1]);
        let (f2, f3) = (f * f, f * f * f);
        let hermite = |sl: &Slice| -> (f64, f64) {
            let (p0, p1) = (sl.phi[i], sl.phi[i + 1]);
            let (d0, d1) = (sl.pull[i] * r_i * self.dl, sl.pull[i + 1] * r_j * self.dl);
            let phi = (2.0 * f3 - 3.0 * f2 + 1.0) * p0
                + (f3 - 2.0 * f2 + f) * d0
                + (-2.0 * f3 + 3.0 * f2) * p1
                + (f3 - f2) * d1;
            let dphi = ((6.0 * f2 - 6.0 * f) * p0
                + (3.0 * f2 - 4.0 * f + 1.0) * d0
                + (-6.0 * f2 + 6.0 * f) * p1
                + (3.0 * f2 - 2.0 * f) * d1)
                / self.dl;
            (phi, dphi)
        };
        let (pa, ga) = hermite(a);
        let (pb, gb) = hermite(b);
        let inner = r < self.radii[0];
        let r_here = if inner {
            self.radii[0]
        } else {
            r.min(self.radii[self.n - 1])
        };
        // Inside the table's first point the pull falls with the enclosed
        // mass, taken as uniform there (ρ constant): g ∝ r.
        let shrink = if inner { r / self.radii[0] } else { 1.0 };
        Here {
            pull: mix(ga, gb) / r_here * shrink,
            phi: mix(pa, pb),
            rho: mix(lin(&a.rho), lin(&b.rho)),
            hot: mix(lin(&a.hot), lin(&b.hot)),
            sigma: mix(lin(&a.sigma), lin(&b.sigma)),
            mass: mix(lin(&a.mass), lin(&b.mass)) * shrink * shrink * shrink,
            slope: mix(lin(&a.slope), lin(&b.slope)),
            r200: mix(a.r200_kpc, b.r200_kpc),
            disc_msun: mix(a.disc_msun, b.disc_msun),
            disc_kpc: mix(a.disc_kpc, b.disc_kpc),
        }
    }
}

/// The drag's speed factor F(X) = erf(X) − 2X/√π e^(−X²), X = v/(√2 σ),
/// read from one fixed table (cut 3), 0 ≤ X ≤ 8; F = 1 beyond.
struct DragTable(Vec<f64>);

const DRAG_X_MAX: f64 = 8.0;
const DRAG_N: usize = 512;

impl DragTable {
    fn new() -> DragTable {
        DragTable(
            (0..=DRAG_N)
                .map(|i| {
                    let x = DRAG_X_MAX * i as f64 / DRAG_N as f64;
                    m::erf(x) - 2.0 * x / m::sqrt(m::PI) * m::exp(-x * x)
                })
                .collect(),
        )
    }
    fn at(&self, x: f64) -> f64 {
        if x >= DRAG_X_MAX {
            return 1.0;
        }
        let u = x.max(0.0) / DRAG_X_MAX * DRAG_N as f64;
        let i = (m::floor(u) as usize).min(DRAG_N - 1);
        let f = u - i as f64;
        self.0[i] + f * (self.0[i + 1] - self.0[i])
    }
}

/// The satellite's own galaxy at infall, for gas stripping: stars, gas and
/// disc scale length. Without it, gas stripping is not followed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grip {
    pub stars_msun: f64,
    pub gas_msun: f64,
    pub disc_kpc: f64,
    /// Gas disc scale over the stellar disc's.
    pub gas_scale: f64,
}

/// A body at infall.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub address: String,
    pub t_infall_gyr: f64,
    pub mass_msun: f64,
    /// Its own NFW concentration at infall (Dutton and Macciò 2014).
    pub concentration: f64,
    /// Its r200 at infall, kpc.
    pub r200_kpc: f64,
    pub pos_kpc: [f64; 3],
    pub vel_kpc_gyr: [f64; 3],
    pub grip: Option<Grip>,
}

/// Dutton and Macciò 2014 (doi:10.1093/mnras/stu742), eqs. 10 to 12: the
/// median NFW concentration c200 at redshift z, mass in M☉ (h = 0.7 in the
/// pivot, as their fit is written per h⁻¹ M☉).
pub fn concentration_dm14(mass_msun: f64, z: f64, h: f64) -> f64 {
    let b = -0.101 + 0.026 * z;
    let a = 0.520 + (0.905 - 0.520) * m::exp(-0.617 * m::pow(z, 1.21));
    m::pow(10.0, a + b * m::log10(mass_msun * h / 1.0e12))
}

fn nfw_mu(x: f64) -> f64 {
    m::ln(1.0 + x) - x / (1.0 + x)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Place a body at infall: on the host's r200 at that time, in the
/// direction (`u_cos`, `u_phi`), moving inward at `u_speed` of the host's
/// virial speed with `u_radial` choosing the radial share and `u_turn` the
/// direction of the sideways part. All `u` are unit draws.
#[allow(clippy::too_many_arguments)]
pub fn infall(
    c: &Cosmology,
    gp: &GalaxyParams,
    op: &OrbitParams,
    host_mass_at_infall: f64,
    address: &str,
    mass_msun: f64,
    z_infall: f64,
    t_infall_gyr: f64,
    u: [f64; 5],
    grip: Option<Grip>,
) -> Body {
    let a = 1.0 / (1.0 + z_infall);
    let (r200, v200) = gp.virial(c, host_mass_at_infall, a);
    let (sr200, _) = gp.virial(c, mass_msun, a);
    let cos_t = 2.0 * u[0] - 1.0;
    let sin_t = m::sqrt((1.0 - cos_t * cos_t).max(0.0));
    let phi = 2.0 * m::PI * u[1];
    let n = [sin_t * m::cos(phi), sin_t * m::sin(phi), cos_t];
    let pos = [n[0] * r200, n[1] * r200, n[2] * r200];
    let v = v200 * kms() * (op.speed_range[0] + (op.speed_range[1] - op.speed_range[0]) * u[2]);
    let rad = op.radial_range[0] + (op.radial_range[1] - op.radial_range[0]) * u[3];
    let side = m::sqrt((1.0 - rad * rad).max(0.0));
    // Two directions at right angles to n.
    let e1 = if n[2].abs() < 0.9 {
        let k = m::sqrt(n[0] * n[0] + n[1] * n[1]);
        [-n[1] / k, n[0] / k, 0.0]
    } else {
        let k = m::sqrt(n[1] * n[1] + n[2] * n[2]);
        [0.0, -n[2] / k, n[1] / k]
    };
    let e2 = [
        n[1] * e1[2] - n[2] * e1[1],
        n[2] * e1[0] - n[0] * e1[2],
        n[0] * e1[1] - n[1] * e1[0],
    ];
    let turn = 2.0 * m::PI * u[4];
    let (ct, st) = (m::cos(turn), m::sin(turn));
    let vel = [
        v * (-rad * n[0] + side * (ct * e1[0] + st * e2[0])),
        v * (-rad * n[1] + side * (ct * e1[1] + st * e2[1])),
        v * (-rad * n[2] + side * (ct * e1[2] + st * e2[2])),
    ];
    Body {
        address: address.to_string(),
        t_infall_gyr,
        mass_msun,
        concentration: concentration_dm14(mass_msun, z_infall, c.h),
        r200_kpc: sr200,
        pos_kpc: pos,
        vel_kpc_gyr: vel,
        grip,
    }
}

/// What became of a body, read at a time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fate {
    pub address: String,
    pub t_gyr: f64,
    pub pos_kpc: [f64; 3],
    pub vel_kpc_gyr: [f64; 3],
    pub r_kpc: f64,
    /// Bound dark matter over its infall mass.
    pub dark_kept: f64,
    /// Stars kept over its stars at infall.
    pub stars_kept: f64,
    /// Heat from disc passes over its binding energy.
    pub heat: f64,
    /// When its star formation stopped, if it has.
    pub quench_gyr: Option<f64>,
    /// When it reached the centre, if it has.
    pub merged_gyr: Option<f64>,
    /// When it was torn apart, if it has been.
    pub disrupted_gyr: Option<f64>,
    pub pericentres: u32,
    pub peri_kpc: f64,
    pub apo_kpc: f64,
    pub disc_passes: u32,
    pub step_gyr: f64,
    pub steps: u64,
    /// Read off the results (design 7.3): `satellite`, `being torn apart`,
    /// `stream`, `shell`, `mixed remains`, `merged`. A thing can be two.
    pub labels: Vec<String>,
    /// Where it was at each host slice from infall on: (Gyr, kpc).
    pub track: Vec<(f64, [f64; 3])>,
    /// Its stars kept (over those at infall) at each point of `track`.
    pub kept: Vec<f64>,
    /// Each closest approach and each pass through the disc while bound
    /// (GH3 reads the host's responses off these).
    pub passes: Vec<Pass>,
}

/// One close moment of an orbit: a closest approach or a pass through the
/// host's disc plane inside the disc's reach.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pass {
    pub t_gyr: f64,
    /// `true` for a crossing of the disc plane, `false` for a closest approach.
    pub disc: bool,
    pub r_kpc: f64,
    /// Distance from the axis in the disc plane.
    pub big_r_kpc: f64,
    pub speed_kms: f64,
    /// Speed across the disc plane.
    pub vz_kms: f64,
    /// Angular momentum along the disc's spin (+z) over its size: +1 with
    /// the disc's turning, −1 against it.
    pub prograde: f64,
    /// Bound mass then.
    pub bound_msun: f64,
    /// The host's circular speed and mass inside r then.
    pub v_c_kms: f64,
    pub host_inside_msun: f64,
    pub disc_msun: f64,
    pub disc_kpc: f64,
}

/// The step a body takes, from its own orbit near closest approach in the
/// host at infall (cut 2): a function of the body alone.
pub fn step_for(host: &Host, b: &Body, op: &OrbitParams) -> f64 {
    let t = b.t_infall_gyr;
    let r0 = m::sqrt(dot(b.pos_kpc, b.pos_kpc));
    let v2 = dot(b.vel_kpc_gyr, b.vel_kpc_gyr);
    let e = 0.5 * v2 + host.at(t, r0).phi;
    let lx = b.pos_kpc[1] * b.vel_kpc_gyr[2] - b.pos_kpc[2] * b.vel_kpc_gyr[1];
    let ly = b.pos_kpc[2] * b.vel_kpc_gyr[0] - b.pos_kpc[0] * b.vel_kpc_gyr[2];
    let lz = b.pos_kpc[0] * b.vel_kpc_gyr[1] - b.pos_kpc[1] * b.vel_kpc_gyr[0];
    let l2 = lx * lx + ly * ly + lz * lz;
    // Closest approach: 2(E − Φ(r)) r² = L², bisected in ln r below r0.
    let f = |r: f64| 2.0 * (e - host.at(t, r).phi) * r * r - l2;
    let (mut lo, mut hi) = (m::ln(1.0e-3), m::ln(r0));
    if f(m::exp(lo)) > 0.0 {
        hi = lo;
    } else {
        for _ in 0..50 {
            let mid = 0.5 * (lo + hi);
            if f(m::exp(mid)) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
    }
    let rp = m::exp(hi);
    let vp = m::sqrt((2.0 * (e - host.at(t, rp).phi)).max(1.0e-12));
    (rp / vp / op.steps_near_peri).clamp(op.step_range_gyr[0], op.step_range_gyr[1])
}

/// Follow a body from its infall to `t_to` under the four processes.
pub fn follow(host: &Host, b: &Body, t_to: f64, op: &OrbitParams) -> Fate {
    follow_with(host, b, t_to, op, true)
}

/// As [`follow`]; `processes` false gives the bare orbit (tests).
pub fn follow_with(host: &Host, b: &Body, t_to: f64, op: &OrbitParams, processes: bool) -> Fate {
    let drag = DragTable::new();
    let g = host.g();
    let dt = step_for(host, b, op);
    let mut x = b.pos_kpc;
    let mut v = b.vel_kpc_gyr;
    let mut t = b.t_infall_gyr;
    let m0 = b.mass_msun;
    let mut mass = m0;
    let rs = b.r200_kpc / b.concentration.max(1.0e-3);
    let mu_c = nfw_mu(b.concentration);
    let mut rt = b.r200_kpc;
    let mut heat = 0.0;
    let mut quench = None;
    let mut merged = None;
    let mut disrupted = None;
    let (mut peri_n, mut passes_n) = (0u32, 0u32);
    let r_start = m::sqrt(dot(x, x));
    let (mut peri, mut apo) = (r_start, r_start);
    let mut track = vec![(t, x)];
    let mut passes: Vec<Pass> = vec![];
    let kms_k = kms();
    let pass_at = |t: f64, disc: bool, x: [f64; 3], v: [f64; 3], mass: f64, h: &Here| {
        let r = m::sqrt(dot(x, x));
        let s = m::sqrt(dot(v, v));
        let lz = x[0] * v[1] - x[1] * v[0];
        let l = m::sqrt({
            let lx = x[1] * v[2] - x[2] * v[1];
            let ly = x[2] * v[0] - x[0] * v[2];
            lx * lx + ly * ly + lz * lz
        });
        Pass {
            t_gyr: t,
            disc,
            r_kpc: r,
            big_r_kpc: m::sqrt(x[0] * x[0] + x[1] * x[1]),
            speed_kms: s / kms_k,
            vz_kms: v[2].abs() / kms_k,
            prograde: if l > 0.0 { lz / l } else { 0.0 },
            bound_msun: mass,
            v_c_kms: m::sqrt(g * h.mass.max(0.0) / r.max(1.0e-6)) / kms_k,
            host_inside_msun: h.mass,
            disc_msun: h.disc_msun,
            disc_kpc: h.disc_kpc,
        }
    };
    let kept_of = |mass: f64, gone: bool| {
        if gone {
            0.0
        } else {
            (mass / m0 / op.stars_leave_below).min(1.0)
        }
    };
    let mut kept = vec![1.0];
    let mut next_slice = t + op.slice_gyr;
    let mut steps = 0u64;
    // The grip on its gas at its disc scale (Gunn and Gott): 2πG Σ_tot Σ_gas.
    let grip = b.grip.map(|gr| {
        let rd = gr.disc_kpc.max(1.0e-3);
        let rg = rd * gr.gas_scale.max(1.0e-3);
        let e1 = m::exp(-1.0);
        let s_tot = gr.stars_msun / (2.0 * m::PI * rd * rd) * e1
            + gr.gas_msun / (2.0 * m::PI * rg * rg) * m::exp(-rd / rg);
        let s_gas = gr.gas_msun / (2.0 * m::PI * rg * rg) * m::exp(-rd / rg);
        2.0 * m::PI * g * s_tot * s_gas
    });
    // Acceleration: the host's pull, and the drag while bound.
    let accel = |x: [f64; 3], v: [f64; 3], t: f64, mass: f64, bound: bool| -> ([f64; 3], Here) {
        let r = m::sqrt(dot(x, x)).max(1.0e-6);
        let h = host.at(t, r);
        let mut a = [-h.pull * x[0] / r, -h.pull * x[1] / r, -h.pull * x[2] / r];
        if processes && bound {
            let s = m::sqrt(dot(v, v)).max(1.0e-9);
            let xx = s / (std::f64::consts::SQRT_2 * h.sigma.max(1.0e-9));
            let ln_l = m::ln(1.0 + h.mass / mass.max(1.0));
            let k = 4.0 * m::PI * g * g * mass * h.rho * ln_l * drag.at(xx) / (s * s * s);
            a[0] -= k * v[0];
            a[1] -= k * v[1];
            a[2] -= k * v[2];
        }
        (a, h)
    };
    let mut bound = true;
    let (mut a, _) = accel(x, v, t, mass, bound);
    let mut r_prev = r_start;
    let mut dr_prev = -1.0;
    while t < t_to && merged.is_none() {
        let h = (t_to - t).min(dt);
        // Kick, drift, kick.
        for k in 0..3 {
            v[k] += 0.5 * h * a[k];
        }
        let z_before = x[2];
        for k in 0..3 {
            x[k] += h * v[k];
        }
        t += h;
        steps += 1;
        // The host here is read once a step, at the new position (the drag
        // uses the half-kicked speed, as kick-drift-kick does).
        let (a_new, here) = accel(x, v, t, mass, bound);
        a = a_new;
        for k in 0..3 {
            v[k] += 0.5 * h * a[k];
        }
        let r = m::sqrt(dot(x, x));
        // Closest and farthest approach.
        let dr = r - r_prev;
        if dr_prev < 0.0 && dr >= 0.0 {
            peri_n += 1;
            peri = peri.min(r_prev);
            if processes && bound {
                passes.push(pass_at(t, false, x, v, mass, &here));
            }
        }
        if dr_prev > 0.0 && dr <= 0.0 {
            apo = apo.max(r_prev);
        }
        r_prev = r;
        if dr != 0.0 {
            dr_prev = dr;
        }
        if r < op.merge_radius_frac * here.r200 {
            merged = Some(t);
            peri = peri.min(r);
        }
        if processes && bound {
            // Tidal radius: r_t = r (m(<r_t) / ((2 − dlnM/dlnr) M(<r)))^(1/3),
            // one fixed-point step a step from the last one (it moves slowly
            // next to the step, so it follows the root closely).
            let denom = ((2.0 - here.slope).max(1.0e-3) * here.mass).max(1.0);
            let inside = (m0 * nfw_mu(rt / rs) / mu_c).min(mass);
            rt = r * m::cbrt(inside / denom);
            let inside = (m0 * nfw_mu(rt / rs) / mu_c).min(mass);
            if mass > inside {
                let t_dyn = 2.0 * m::PI * m::sqrt(r * r * r / (g * here.mass.max(1.0)));
                let rate = op.strip_alpha * (1.0 + heat) / t_dyn;
                // First order in the step: rate × h is small.
                mass -= (mass - inside) * (rate * h).min(1.0);
            }
            // A pass through the disc (impulsive heating, design 7.3).
            let big_r = m::sqrt(x[0] * x[0] + x[1] * x[1]);
            if z_before * x[2] < 0.0 && big_r < op.disc_reach * here.disc_kpc {
                passes_n += 1;
                passes.push(pass_at(t, true, x, v, mass, &here));
                let sigma_d = here.disc_msun / (2.0 * m::PI * here.disc_kpc * here.disc_kpc)
                    * m::exp(-big_r / here.disc_kpc);
                let g_m = 2.0 * m::PI * g * sigma_d;
                // Half the bound mass sits inside about this radius.
                let rh = (0.5 * rt).min(rs * 2.163);
                let vz = v[2].abs().max(1.0e-6);
                let de = 2.0 * g_m * g_m * rh * rh / (3.0 * vz * vz);
                let bind = 0.5 * g * mass / rh.max(1.0e-6);
                heat += de / bind.max(1.0e-30);
            }
            if let (Some(grip), None) = (grip, quench) {
                if here.hot * dot(v, v) > grip {
                    quench = Some(t);
                }
            }
            if mass < op.disrupt_below * m0 || heat >= 1.0 {
                disrupted = Some(t);
                bound = false;
                quench = quench.or(Some(t));
            }
        }
        if t >= next_slice {
            track.push((t, x));
            kept.push(kept_of(mass, disrupted.is_some()));
            next_slice += op.slice_gyr;
        }
    }
    if track.last().is_some_and(|l| l.0 < t) {
        track.push((t, x));
        kept.push(kept_of(mass, disrupted.is_some()));
    }
    let dark_kept = mass / m0;
    let stars_kept = if disrupted.is_some() {
        0.0
    } else if dark_kept >= op.stars_leave_below {
        1.0
    } else {
        // F46: below the threshold, the stars follow the dark matter down.
        dark_kept / op.stars_leave_below
    };
    let r = m::sqrt(dot(x, x));
    let mut labels = vec![];
    if merged.is_some() {
        labels.push("merged".to_string());
    } else if let Some(td) = disrupted {
        let ratio = if apo > 0.0 { peri / apo } else { 1.0 };
        if t - td > op.mix_gyr {
            labels.push("mixed remains".into());
        } else if ratio < op.shell_below {
            labels.push("shell".into());
        } else {
            labels.push("stream".into());
        }
    } else {
        labels.push(if stars_kept < 1.0 {
            "being torn apart".into()
        } else {
            "satellite".into()
        });
        if stars_kept < 1.0 {
            labels.push("stream".into());
        }
    }
    Fate {
        address: b.address.clone(),
        t_gyr: t,
        pos_kpc: x,
        vel_kpc_gyr: v,
        r_kpc: r,
        dark_kept,
        stars_kept,
        heat,
        quench_gyr: quench,
        merged_gyr: merged,
        disrupted_gyr: disrupted,
        pericentres: peri_n,
        peri_kpc: peri,
        apo_kpc: apo,
        disc_passes: passes_n,
        step_gyr: dt,
        steps,
        labels,
        track,
        kept,
        passes,
    }
}

/// The energy per unit mass of a body (tests and the drift bound).
pub fn energy(host: &Host, t: f64, x: [f64; 3], v: [f64; 3]) -> f64 {
    0.5 * dot(v, v) + host.at(t, m::sqrt(dot(x, x))).phi
}

#[cfg(test)]
#[path = "gh2_tests.rs"]
mod gh2_tests;
