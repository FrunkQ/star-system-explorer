//! C: cosmology (galaxy design section 2, stage G1).
//!
//! Flat Lambda-CDM with radiation. Everything is closed form or a fixed
//! quadrature, so every device gets the same numbers:
//! - expansion E(a) = H(a)/H0 = sqrt(Or a^-4 + Om a^-3 + OL);
//! - cosmic time t(a) by fixed Gauss-Legendre quadrature (with radiation),
//!   and the matter-plus-Lambda closed form t(a) = 2/(3 H0 sqrt(OL)) asinh(sqrt(OL/Om) a^1.5)
//!   for the atom (both agree to 0.1 percent below z = 10);
//! - linear growth D(a): Carroll, Press and Turner 1992 eq. 29, normalised to D(1) = 1,
//!   with the exact Heath 1977 integral kept as the check;
//! - matter power spectrum P(k) proportional to k^ns T(k)^2 with the Eisenstein and Hu 1998
//!   no-wiggle transfer function, normalised to sigma8;
//! - sigma(R) with a top-hat window, tabulated once per parameter set as sigma(M).

use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Seconds in a Julian year.
pub const YEAR_S: f64 = sse_core::sheet::YR_JULIAN;
/// Megaparsec in metres (IAU 2015: 1 pc = 648000/pi au).
pub const MPC_M: f64 = 1e6 * sse_core::sheet::PC;
/// Critical density today per h^2, solar masses per cubic Mpc (3 H^2 / 8 pi G with H = 100 km/s/Mpc).
/// (H = 1e5 m/s per Mpc, so H^2 / G in solar masses per Mpc^3 is (1e5)^2 Mpc / GM_sun.)
pub const RHO_CRIT_H2: f64 = 3.0 * (1e5 * 1e5) * MPC_M / (8.0 * m::PI * sse_core::sheet::GM_SUN);

/// The universe's parameters. The measured ones are the master data
/// sheet's Cosmos group (DM1), read through the constants handle; only
/// `now_gyr`, a model setting, comes from `provides.other.cosmology`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Params {
    /// Hubble constant, km/s/Mpc.
    pub h0: f64,
    pub omega_m: f64,
    pub omega_b: f64,
    pub sigma8: f64,
    pub n_s: f64,
    /// CMB temperature, K.
    pub t_cmb: f64,
    /// Effective number of neutrino species (radiation density).
    pub n_eff: f64,
    /// "Now", Gyr after the origin.
    pub now_gyr: f64,
    /// DM2: dark energy's pressure over density, w = w0 + wa (1 − a)
    /// (Chevallier and Polarski 2001; Linder 2003). −1 and 0 is a
    /// cosmological constant, and then every number is as before.
    #[serde(default = "minus_one", skip_serializing_if = "is_minus_one")]
    pub w0: f64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub wa: f64,
}

fn minus_one() -> f64 {
    -1.0
}
fn is_minus_one(x: &f64) -> bool {
    *x == -1.0
}
fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

impl Params {
    /// The measured parameters from the Cosmos group `c`, with `now_gyr`.
    pub fn from_cosmos(c: &sse_core::constants::Cosmos, now_gyr: f64) -> Self {
        Params {
            h0: c.h0,
            omega_m: c.omega_m,
            omega_b: c.omega_b,
            sigma8: c.sigma8,
            n_s: c.n_s,
            t_cmb: c.t_cmb,
            n_eff: c.n_eff,
            now_gyr,
            w0: c.w0,
            wa: c.wa,
        }
    }
}

impl Default for Params {
    /// The real universe (the sheet's values as compiled), now at 13.8 Gyr.
    fn default() -> Self {
        Self::from_cosmos(&sse_core::constants::Cosmos::REAL, 13.8)
    }
}

/// Gauss-Legendre 16-point nodes and weights on [-1, 1] (Abramowitz and Stegun 1964, table 25.4).
const GL_X: [f64; 8] = [
    0.095_012_509_837_637_44,
    0.281_603_550_779_258_9,
    0.458_016_777_657_227_4,
    0.617_876_244_402_643_7,
    0.755_404_408_355_003,
    0.865_631_202_387_831_8,
    0.944_575_023_073_232_6,
    0.989_400_934_991_649_9,
];
const GL_W: [f64; 8] = [
    0.189_450_610_455_068_5,
    0.182_603_415_044_923_6,
    0.169_156_519_395_002_5,
    0.149_595_988_816_576_7,
    0.124_628_971_255_533_9,
    0.095_158_511_682_492_8,
    0.062_253_523_938_647_9,
    0.027_152_459_411_754_1,
];

/// Integral of `f` over [a, b] by `panels` fixed 16-point Gauss-Legendre panels.
pub fn quad(f: &dyn Fn(f64) -> f64, a: f64, b: f64, panels: u32) -> f64 {
    let h = (b - a) / f64::from(panels);
    let mut s = 0.0;
    for p in 0..panels {
        let c = a + h * (f64::from(p) + 0.5);
        let r = 0.5 * h;
        for k in 0..8 {
            s += GL_W[k] * (f(c - r * GL_X[k]) + f(c + r * GL_X[k])) * r;
        }
    }
    s
}

/// erfc by Abramowitz and Stegun 1964 eq. 7.1.26 (absolute error below 1.5e-7).
pub fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * z);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let e = poly * m::exp(-z * z);
    if x >= 0.0 {
        e
    } else {
        2.0 - e
    }
}

/// A cosmology with its tables.
#[derive(Clone, PartialEq, Debug)]
pub struct Cosmology {
    pub p: Params,
    pub h: f64,
    pub omega_r: f64,
    pub omega_l: f64,
    /// 1/H0 in Gyr.
    pub hubble_time_gyr: f64,
    /// sigma(M) table: (log10 M in solar masses, sigma at z = 0).
    pub sigma_table: Vec<(f64, f64)>,
    /// Power-spectrum amplitude fixed by sigma8.
    pub amplitude: f64,
    /// DM2: with w other than −1, the growth factor stepped from its
    /// equation, (ln a, D unnormalised) on a fixed grid; empty for Λ.
    pub growth_table: Vec<(f64, f64)>,
}

/// The growth table's grid: ln a from this to 0.5, in this many steps.
const GROWTH_LN_A0: f64 = -9.210_340_371_976_184; // ln 1e-4
const GROWTH_STEPS: usize = 2000;

impl Cosmology {
    pub fn new(p: Params) -> Self {
        let h = p.h0 / 100.0;
        // Photons: Omega_g h^2 = 2.473e-5 (T/2.7255)^4; neutrinos add 7/8 (4/11)^(4/3) per species.
        let og = 2.473e-5 * m::pow(p.t_cmb / 2.7255, 4.0) / (h * h);
        let omega_r = og * (1.0 + 0.227_107_317_7 * p.n_eff);
        let omega_l = 1.0 - p.omega_m - omega_r;
        let hubble_time_gyr = MPC_M / (p.h0 * 1000.0) / YEAR_S / 1.0e9;
        let mut c = Cosmology {
            p,
            h,
            omega_r,
            omega_l,
            hubble_time_gyr,
            sigma_table: vec![],
            amplitude: 1.0,
            growth_table: vec![],
        };
        if !c.is_lambda() {
            c.growth_table = c.growth_steps();
        }
        let r8 = 8.0 / h;
        c.amplitude = (c.p.sigma8 * c.p.sigma8) / c.sigma2_r(r8);
        c.sigma_table = (0..=280)
            .map(|k| {
                let lm = 3.0 + 0.05 * f64::from(k);
                (lm, m::sqrt(c.sigma2_r(c.radius_of_mass(m::pow(10.0, lm)))))
            })
            .collect();
        c
    }

    /// A cosmological constant (w = −1 at every time)?
    pub fn is_lambda(&self) -> bool {
        self.p.w0 == -1.0 && self.p.wa == 0.0
    }

    /// Dark energy's density at a over today's: a^(−3(1 + w0 + wa)) e^(−3 wa (1 − a))
    /// for w = w0 + wa (1 − a); exactly 1 for a cosmological constant.
    pub fn dark_energy(&self, a: f64) -> f64 {
        if self.is_lambda() {
            return 1.0;
        }
        let (w0, wa) = (self.p.w0, self.p.wa);
        m::pow(a, -3.0 * (1.0 + w0 + wa)) * m::exp(-3.0 * wa * (1.0 - a))
    }

    /// w at scale factor a.
    pub fn w(&self, a: f64) -> f64 {
        self.p.w0 + self.p.wa * (1.0 - a)
    }

    /// E(a) = H(a) / H0.
    pub fn e(&self, a: f64) -> f64 {
        if self.is_lambda() {
            return m::sqrt(
                self.omega_r / (a * a * a * a) + self.p.omega_m / (a * a * a) + self.omega_l,
            );
        }
        m::sqrt(
            self.omega_r / (a * a * a * a)
                + self.p.omega_m / (a * a * a)
                + self.omega_l * self.dark_energy(a),
        )
    }

    /// The linear growth equation in ln a, for matter and any smooth dark
    /// energy: D'' + (2 + d ln H / d ln a) D' − (3/2) Ωm(a) D = 0, from the
    /// matter era's growing mode D = a, stepped by fixed RK4 (radiation
    /// left out, as in `growth_exact`). (ln a, D) on the grid.
    fn growth_steps(&self) -> Vec<(f64, f64)> {
        let h2 = |a: f64| self.p.omega_m / (a * a * a) + self.omega_l * self.dark_energy(a);
        // d ln H / d ln a = (1/2) d ln H² / d ln a, with ρ_DE' = −3(1 + w) ρ_DE.
        let dlnh = |a: f64| {
            let de = self.omega_l * self.dark_energy(a);
            let mm = self.p.omega_m / (a * a * a);
            0.5 * (-3.0 * mm - 3.0 * (1.0 + self.w(a)) * de) / (mm + de)
        };
        let om = |a: f64| self.p.omega_m / (a * a * a) / h2(a);
        let f = |x: f64, d: f64, v: f64| {
            let a = m::exp(x);
            (v, -(2.0 + dlnh(a)) * v + 1.5 * om(a) * d)
        };
        let h = (0.5 - GROWTH_LN_A0) / GROWTH_STEPS as f64;
        let mut x = GROWTH_LN_A0;
        let a0 = m::exp(x);
        let (mut d, mut v) = (a0, a0);
        let mut out = Vec::with_capacity(GROWTH_STEPS + 1);
        out.push((x, d));
        for _ in 0..GROWTH_STEPS {
            let (k1d, k1v) = f(x, d, v);
            let (k2d, k2v) = f(x + 0.5 * h, d + 0.5 * h * k1d, v + 0.5 * h * k1v);
            let (k3d, k3v) = f(x + 0.5 * h, d + 0.5 * h * k2d, v + 0.5 * h * k2v);
            let (k4d, k4v) = f(x + h, d + h * k3d, v + h * k3v);
            d += h / 6.0 * (k1d + 2.0 * k2d + 2.0 * k3d + k4d);
            v += h / 6.0 * (k1v + 2.0 * k2v + 2.0 * k3v + k4v);
            x += h;
            out.push((x, d));
        }
        out
    }

    /// The stepped growth at a (unnormalised), linear in ln a between grid points.
    fn growth_stepped(&self, table: &[(f64, f64)], a: f64) -> f64 {
        let x = m::ln(a.max(1.0e-12));
        if x <= table[0].0 {
            return table[0].1 * a / m::exp(table[0].0);
        }
        let h = table[1].0 - table[0].0;
        let i = (((x - table[0].0) / h) as usize).min(table.len() - 2);
        let (x0, d0) = table[i];
        let (_, d1) = table[i + 1];
        d0 + (d1 - d0) * (x - x0) / h
    }

    /// Cosmic time at scale factor a, Gyr, with radiation (fixed quadrature in s = sqrt(a)).
    pub fn time_of_scale(&self, a: f64) -> f64 {
        // dt = da / (a H) = 2 s ds / (s^2 H); with a = s^2.
        let f = |s: f64| {
            let a = s * s;
            2.0 * s / (a * self.e(a))
        };
        // The integrand tends to 2 s^3 / sqrt(Or) near 0: smooth.
        let s = m::sqrt(a);
        quad(&|x| if x <= 0.0 { 0.0 } else { f(x) }, 0.0, s, 64) * self.hubble_time_gyr
    }

    /// Scale factor at cosmic time t (Gyr): fixed 80-step bisection in log a.
    pub fn scale_of_time(&self, t_gyr: f64) -> f64 {
        let (mut lo, mut hi) = (-12.0_f64, 3.0_f64);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if self.time_of_scale(m::pow(10.0, mid)) < t_gyr {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        m::pow(10.0, 0.5 * (lo + hi))
    }

    /// Comoving distance light covers from scale factor `a` to now, Mpc:
    /// c / H0 times the integral of da / (a² E(a)) from a to 1 (Hogg 1999,
    /// arXiv:astro-ph/9905116, eq. 15), by fixed quadrature in ln a.
    pub fn comoving_mpc(&self, a: f64) -> f64 {
        if a >= 1.0 {
            return 0.0;
        }
        let c_mpc_gyr = sse_core::quantity::consts::C * YEAR_S * 1.0e9 / MPC_M;
        let la = m::ln(a.max(1.0e-12));
        // da / (a² E) = d(ln a) / (a E).
        let f = |x: f64| {
            let aa = m::exp(x);
            1.0 / (aa * self.e(aa))
        };
        quad(&f, la, 0.0, 64) * c_mpc_gyr * self.hubble_time_gyr
    }

    /// The scale factor at which light now arriving from comoving distance
    /// `d_mpc` set out (fixed 60-step bisection in ln a).
    pub fn scale_at_comoving(&self, d_mpc: f64) -> f64 {
        if d_mpc <= 0.0 {
            return 1.0;
        }
        let (mut lo, mut hi) = (-12.0_f64, 0.0_f64);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if self.comoving_mpc(m::exp(mid)) > d_mpc {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        m::exp(0.5 * (lo + hi))
    }

    pub fn age_gyr(&self) -> f64 {
        self.time_of_scale(1.0)
    }

    /// Omega_m(a) and Omega_L(a).
    fn omegas(&self, a: f64) -> (f64, f64) {
        let e2 = self.e(a) * self.e(a);
        (
            self.p.omega_m / (a * a * a) / e2,
            self.omega_l * self.dark_energy(a) / e2,
        )
    }

    /// The growth equation stepped even for Λ (the check against the
    /// closed forms).
    pub fn growth_by_steps(&self, a: f64) -> f64 {
        let t = self.growth_steps();
        self.growth_stepped(&t, a) / self.growth_stepped(&t, 1.0)
    }

    /// Linear growth factor, Carroll, Press and Turner 1992, D(1) = 1; with
    /// w other than −1, stepped from its equation (DM2).
    pub fn growth(&self, a: f64) -> f64 {
        if !self.growth_table.is_empty() {
            let t = &self.growth_table;
            return self.growth_stepped(t, a) / self.growth_stepped(t, 1.0);
        }
        let g = |om: f64, ol: f64| {
            2.5 * om / (m::pow(om, 4.0 / 7.0) - ol + (1.0 + 0.5 * om) * (1.0 + ol / 70.0))
        };
        let (om, ol) = self.omegas(a);
        let (o1, l1) = self.omegas(1.0);
        a * g(om, ol) / g(o1, l1)
    }

    /// Exact growing mode for matter plus Lambda (Heath 1977): D proportional to
    /// E(a) times the integral of da / (a E)^3; radiation left out. The check on `growth`.
    pub fn growth_exact(&self, a: f64) -> f64 {
        let em = |x: f64| m::sqrt(self.p.omega_m / (x * x * x) + self.omega_l + self.omega_r * 0.0);
        let d = |aa: f64| {
            let i = quad(
                &|x: f64| {
                    if x <= 0.0 {
                        0.0
                    } else {
                        1.0 / m::pow(x * em(x), 3.0)
                    }
                },
                0.0,
                aa,
                64,
            );
            em(aa) * i
        };
        d(a) / d(1.0)
    }

    /// Eisenstein and Hu 1998 no-wiggle transfer function, k in 1/Mpc (eqs. 26 to 31).
    pub fn transfer(&self, k: f64) -> f64 {
        let h = self.h;
        let wm = self.p.omega_m * h * h;
        let wb = self.p.omega_b * h * h;
        let fb = wb / wm;
        let theta = self.p.t_cmb / 2.7;
        let s = 44.5 * m::ln(9.83 / wm) / m::sqrt(1.0 + 10.0 * m::pow(wb, 0.75));
        let alpha = 1.0 - 0.328 * m::ln(431.0 * wm) * fb + 0.38 * m::ln(22.3 * wm) * fb * fb;
        let ks = 0.43 * k * s;
        let gamma = self.p.omega_m * h * (alpha + (1.0 - alpha) / (1.0 + ks * ks * ks * ks));
        let q = k * theta * theta / (gamma * h);
        let l0 = m::ln(2.0 * core::f64::consts::E + 1.8 * q);
        let c0 = 14.2 + 731.0 / (1.0 + 62.5 * q);
        l0 / (l0 + c0 * q * q)
    }

    /// Linear power spectrum today, Mpc^3 (amplitude set by sigma8).
    pub fn power(&self, k: f64) -> f64 {
        let t = self.transfer(k);
        self.amplitude * m::pow(k, self.p.n_s) * t * t
    }

    /// sigma^2 of the linear field today in a top-hat sphere of radius R (Mpc):
    /// 1/(2 pi^2) integral k^3 P(k) W^2(kR) dln k, 4000 trapezoid steps over 1e-5 to 1e3 / Mpc.
    pub fn sigma2_r(&self, r: f64) -> f64 {
        let (l0, l1) = (m::ln(1.0e-5), m::ln(1.0e3));
        let n = 4000;
        let dl = (l1 - l0) / f64::from(n);
        let mut s = 0.0;
        for i in 0..=n {
            let k = m::exp(l0 + dl * f64::from(i));
            let x = k * r;
            let w = if x < 1.0e-3 {
                1.0 - x * x / 10.0
            } else {
                3.0 * (m::sin(x) - x * m::cos(x)) / (x * x * x)
            };
            let wt = if i == 0 || i == n { 0.5 } else { 1.0 };
            s += wt * k * k * k * self.power(k) * w * w;
        }
        s * dl / (2.0 * m::PI * m::PI)
    }

    /// Mean matter density today, solar masses per cubic Mpc (comoving).
    pub fn rho_m(&self) -> f64 {
        RHO_CRIT_H2 * self.h * self.h * self.p.omega_m
    }

    /// Comoving radius (Mpc) holding mass M (solar masses) at the mean density.
    pub fn radius_of_mass(&self, mass: f64) -> f64 {
        m::cbrt(3.0 * mass / (4.0 * m::PI * self.rho_m()))
    }

    /// sigma(M) today, from the table (linear in log M; 1e3 to 1e17 solar masses).
    pub fn sigma_m(&self, mass: f64) -> f64 {
        let lm = m::log10(mass).clamp(3.0, 17.0);
        let t = &self.sigma_table;
        let i = (((lm - 3.0) / 0.05) as usize).min(t.len() - 2);
        let (a, b) = (t[i], t[i + 1]);
        a.1 + (b.1 - a.1) * (lm - a.0) / (b.0 - a.0)
    }

    /// Redshift at which a nu-sigma peak of mass M collapses: D(a) sigma(M) nu = delta_c.
    pub fn collapse_redshift(&self, mass: f64, nu: f64) -> f64 {
        let target = DELTA_C / (nu * self.sigma_m(mass));
        // D falls with z: bisection on log(1+z).
        let (mut lo, mut hi) = (0.0_f64, 3.0_f64);
        if self.growth(1.0) < target {
            return -1.0; // not yet collapsed today
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if self.growth(1.0 / m::pow(10.0, mid)) > target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        m::pow(10.0, 0.5 * (lo + hi)) - 1.0
    }
}

/// Linear collapse threshold for a top-hat (Gunn and Gott 1972).
pub const DELTA_C: f64 = 1.686;

/// Closed form t(a) for flat matter plus Lambda (the atom), Gyr.
pub fn time_of_scale_closed(a: f64, h0: f64, omega_m: f64, omega_l: f64) -> f64 {
    let th = MPC_M / (h0 * 1000.0) / YEAR_S / 1.0e9;
    2.0 / (3.0 * m::sqrt(omega_l)) * m::asinh(m::sqrt(omega_l / omega_m) * m::pow(a, 1.5)) * th
}

/// Inverse of the closed form (the matching sinh form).
pub fn scale_of_time_closed(t_gyr: f64, h0: f64, omega_m: f64, omega_l: f64) -> f64 {
    let th = MPC_M / (h0 * 1000.0) / YEAR_S / 1.0e9;
    let x = 1.5 * m::sqrt(omega_l) * t_gyr / th;
    m::pow(m::sinh(x) * m::sqrt(omega_m / omega_l), 2.0 / 3.0)
}

// ---- atoms (H6.2): the closed forms as cited operators for flows ----

use sse_atoms::common::{dims, param, port, positive, Def};
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const CPT: &str = "Carroll, S. M., Press, W. H. and Turner, E. L. 1992, ARA&A 30, 499. doi:10.1146/annurev.aa.30.090192.002435";
const PLANCK: &str = "Planck Collaboration 2020, Planck 2018 results VI, A&A 641, A6. doi:10.1051/0004-6361/201833910";
const PEEBLES: &str = "Peebles, P. J. E. 1993, Principles of Physical Cosmology, Princeton University Press, eq. 13.20";

fn cosmo_params() -> Vec<sse_flow::Param> {
    vec![
        param(
            "h0",
            D::NONE,
            67.66,
            (20.0, 200.0),
            "Hubble constant in km/s/Mpc",
        ),
        param(
            "omega_m",
            D::NONE,
            0.3111,
            (0.01, 1.0),
            "matter density parameter today",
        ),
        param(
            "omega_l",
            D::NONE,
            0.6889,
            (0.0, 0.99),
            "dark-energy density parameter today (flat when Omega_m + Omega_L = 1)",
        ),
    ]
}

/// Cosmology atoms: time of scale, scale of time, growth, Hubble rate.
pub fn atoms() -> Vec<Box<dyn Atom>> {
    let a = || {
        port(
            "scale_factor",
            "cosmo.scale_factor",
            D::NONE,
            Some((1.0e-6, 100.0)),
        )
    };
    let t = || port("time", "cosmo.time", D::TIME, None);
    vec![
        Def {
            id: "sse.cosmo.time_of_scale",
            version: "1.0.0",
            inputs: vec![a()],
            outputs: vec![t()],
            params: cosmo_params(),
            cost: CostClass::Trivial,
            references: &[PEEBLES, PLANCK],
            curious: "The universe's size and its age are tied together: knowing how stretched space is tells you how long it has been expanding.",
            science: "Cosmic time in a flat universe of matter and dark energy, exact in closed form; radiation, ignored here, matters only in the first hundred thousand years.",
            formula: "t(a) = 2 / (3 H0 √Ω_Λ) · asinh(√(Ω_Λ/Ω_m) · a^1.5)",
            f: |i, p| {
                let a = positive(i[0], "scale factor")?;
                Ok(vec![time_of_scale_closed(a, p[0], p[1], p[2]) * 1.0e9 * YEAR_S])
            },
        }
        .build(),
        Def {
            id: "sse.cosmo.scale_of_time",
            version: "1.0.0",
            inputs: vec![t()],
            outputs: vec![a()],
            params: cosmo_params(),
            cost: CostClass::Trivial,
            references: &[PEEBLES, PLANCK],
            curious: "Run the clock forward and space stretches: this gives how big the universe was at any moment.",
            science: "Inverse of the closed-form time for flat matter plus dark energy.",
            formula: "a(t) = (√(Ω_m/Ω_Λ) · sinh(1.5 √Ω_Λ H0 t))^(2/3)",
            f: |i, p| {
                let t = positive(i[0], "time")?;
                Ok(vec![scale_of_time_closed(t / (1.0e9 * YEAR_S), p[0], p[1], p[2])])
            },
        }
        .build(),
        Def {
            id: "sse.cosmo.growth",
            version: "1.0.0",
            inputs: vec![a()],
            outputs: vec![port("growth", "cosmo.growth", D::NONE, None)],
            params: cosmo_params(),
            cost: CostClass::Trivial,
            references: &[CPT],
            curious: "Small lumps in the early universe grow as gravity pulls matter in; dark energy slows that growth down.",
            science: "Linear growth factor of density fluctuations, normalised to 1 today; accurate to about 1 percent against the exact integral.",
            formula: "D(a) = a g(a) / g(1), g = 2.5 Ω_m / (Ω_m^(4/7) − Ω_Λ + (1 + Ω_m/2)(1 + Ω_Λ/70))",
            f: |i, p| {
                let a = positive(i[0], "scale factor")?;
                let gf = |aa: f64| {
                    let e2 = p[1] / (aa * aa * aa) + p[2] + (1.0 - p[1] - p[2]) / (aa * aa);
                    let (om, ol) = (p[1] / (aa * aa * aa) / e2, p[2] / e2);
                    aa * 2.5 * om / (m::pow(om, 4.0 / 7.0) - ol + (1.0 + 0.5 * om) * (1.0 + ol / 70.0))
                };
                Ok(vec![gf(a) / gf(1.0)])
            },
        }
        .build(),
        Def {
            id: "sse.cosmo.hubble_rate",
            version: "1.0.0",
            inputs: vec![a()],
            outputs: vec![port("hubble_rate", "cosmo.hubble_rate", dims::RATE, None)],
            params: cosmo_params(),
            cost: CostClass::Trivial,
            references: &[PEEBLES, PLANCK],
            curious: "How fast space stretches has changed: it slowed for billions of years, and now dark energy is speeding it up again.",
            science: "The Friedmann equation for matter, dark energy and curvature (radiation ignored).",
            formula: "H(a) = H0 √(Ω_m a^-3 + (1 − Ω_m − Ω_Λ) a^-2 + Ω_Λ)",
            f: |i, p| {
                let a = positive(i[0], "scale factor")?;
                let h0 = p[0] * 1000.0 / MPC_M;
                Ok(vec![h0 * m::sqrt(p[1] / (a * a * a) + (1.0 - p[1] - p[2]) / (a * a) + p[2])])
            },
        }
        .build(),
    ]
}
