//! `belts` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::C;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

/// Areal mass of a shield, kg/m².
pub const COLUMN: D = D([-2, 1, 0, 0, 0, 0, 0]);
/// Mass, kg; charge, C; permittivity, F/m.
pub const MASS: D = D([0, 1, 0, 0, 0, 0, 0]);
pub const CHARGE: D = D([0, 0, 1, 1, 0, 0, 0]);
pub const PERMITTIVITY: D = D([-3, -1, 4, 2, 0, 0, 0]);

const MAUK10: &str = "Mauk, B. H. and Fox, N. J. 2010, Electron radiation belts of the solar system, J. Geophys. Res. 115, A12220: the relativistic differential Kennel-Petschek limit, eq. 6 and appendix A; the most intense belts of Earth, Jupiter and Uranus sit near it with an E^-1 shape below about 1 MeV; anisotropy S = 0.3; Neptune 30 times below it at 1 MeV, Saturn over 20 below. doi:10.1029/2010JA015660";
const SUMMERS09: &str = "Summers, D., Tang, R. and Thorne, R. M. 2009, Limit on stably trapped particle fluxes in planetary magnetospheres, J. Geophys. Res. 114, A10210: the relativistic growth rate used for the limit. doi:10.1029/2009JA014428";
const KOCH59: &str = "Koch, H. W. and Motz, J. W. 1959, Bremsstrahlung cross-section formulas and related data, Rev. Mod. Phys. 31, 920: the share of an electron's energy radiated as it stops in a material of atomic number Z, Y = k Z T / (1 + k Z T) with k about 6e-4 per MeV. doi:10.1103/RevModPhys.31.920";
const HUBBELL: &str = "Hubbell, J. H. and Seltzer, S. M. 2004, Tables of X-ray mass attenuation coefficients and mass energy-absorption coefficients (NIST Standard Reference Database 126): aluminium and liquid water. https://physics.nist.gov/PhysRefData/XrayMassCoef/";
const KP66: &str = "Kennel, C. F. and Petschek, H. E. 1966, Limit on stably trapped particle fluxes, J. Geophys. Res. 71, 1: waves grow by about e^3 (5 per cent reflection) at the limit. doi:10.1029/JZ071i001p00001";
const KATZ52: &str = "Katz, L. and Penfold, A. S. 1952, Range-energy relations for electrons and the determination of beta-ray end-point energies by absorption, Rev. Mod. Phys. 24, 28: R = 412 E^(1.265 - 0.0954 ln E) mg/cm² for 0.01 to 2.5 MeV, R = 530 E - 106 above. doi:10.1103/RevModPhys.24.28";

/// The constants the growth rate needs, from the sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Electron {
    pub mass_kg: f64,
    pub charge_c: f64,
    pub eps0: f64,
}

/// Mauk and Fox eq. 7 with kT = 0: I = C E^−index / (1 + (E/E₀)^break_index)
/// sin^(2S) α, E in keV, I per cm² s sr keV.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub index: f64,
    pub break_kev: f64,
    pub break_index: f64,
    pub anisotropy: f64,
}

impl Shape {
    /// Equatorially mirroring intensity per unit C, per cm² s sr keV.
    pub fn at(&self, e_kev: f64) -> f64 {
        m::pow(e_kev, -self.index) / (1.0 + m::pow(e_kev / self.break_kev, self.break_index))
    }
}

/// Growth of a parallel whistler at ω = x Ω for a spectrum with C = 1, the
/// strictly parallel resonant energy (keV) and the group speed (m/s).
fn growth(b_t: f64, wpe_over_w: f64, x: f64, s: &Shape, q: &Electron) -> (f64, f64, f64) {
    let me = q.mass_kg;
    let mc = me * C;
    let w_e = q.charge_c * b_t / me;
    let wpe2 = wpe_over_w * wpe_over_w * w_e * w_e;
    let w = x * w_e;
    let n2 = 1.0 + wpe2 / (w * (w_e - w));
    let n = m::sqrt(n2);
    let k = n * w / C;
    let kev = 1e3 * q.charge_c;
    // f(p⊥, p∥): intensity per m² s sr J over p², with the pitch-angle factor.
    let f = |pp: f64, pl: f64| -> f64 {
        let p2 = pp * pp + pl * pl;
        let e = me * C * C * (m::sqrt(1.0 + p2 / (mc * mc)) - 1.0) / kev;
        let i = s.at(e) * 1e4 / kev;
        i * m::pow(pp * pp / p2, s.anisotropy) / p2
    };
    let res = |pp: f64| -> (f64, f64, f64) {
        let a = 1.0 + pp * pp / (mc * mc);
        let g = (-1.0 + n * m::sqrt((n2 - 1.0) * a * x * x + 1.0)) / ((n2 - 1.0) * x);
        let pr = me / k * (g * w - w_e);
        let dl = 1.0 - w * pr / (me * C * C * k * g);
        (g, pr, dl)
    };
    let (mut j1, mut j2) = (0.0, 0.0);
    let steps = 600;
    let mut prev: Option<(f64, f64, f64)> = None;
    for i in 0..steps {
        let pp = mc * m::pow(10.0, -5.0 + 8.0 * i as f64 / (steps - 1) as f64);
        let (g, pr, dl) = res(pp);
        let h = pp * 1e-4;
        let hl = pr.abs().max(mc * 1e-6) * 1e-4;
        let dfp = (f(pp + h, pr) - f(pp - h, pr)) / (2.0 * h);
        let dfl = (f(pp, pr + hl) - f(pp, pr - hl)) / (2.0 * hl);
        let i1 = pp * pp / dl * dfp;
        let i2 = pp * pp / (dl * g) * (pp * dfl - pr * dfp);
        if let Some((p0, a1, a2)) = prev {
            let dp = pp - p0;
            j1 += 0.5 * (i1 + a1) * dp;
            j2 += 0.5 * (i2 + a2) * dp;
        }
        prev = Some((pp, i1, i2));
    }
    let a_rel = k / (me * (w - w_e)) * j2 / j1;
    let a_c = w / (w_e - w);
    // ω_pe² η_rel with N₀ cancelled: π e² (ω − Ω) J₁ / (ε₀ k).
    let wpe2_eta = m::PI * q.charge_c * q.charge_c * (w - w_e) * j1 / (q.eps0 * k);
    let gamma = m::PI * wpe2_eta * (a_rel - a_c) / (2.0 * w + wpe2 * w_e / ((w - w_e) * (w - w_e)));
    let vg = 2.0 * C * C * k * w_e * (1.0 - x) * (1.0 - x)
        / (wpe2 + 2.0 * w_e * w * (1.0 - x) * (1.0 - x));
    let (_, pr0, _) = res(0.0);
    let er = me * C * C * (m::sqrt(1.0 + pr0 * pr0 / (mc * mc)) - 1.0) / kev;
    (gamma, er, vg)
}

/// The limiting normalisation C_K (Mauk and Fox eq. 6) at strictly parallel
/// resonant energy `er_kev`, for equatorial field `b_t`, wave path `path_m`
/// (D R_p) and cold plasma frequency `wpe_over_w` times the gyrofrequency.
/// Infinite where the waves do not grow.
pub fn kp_normalisation(
    b_t: f64,
    path_m: f64,
    er_kev: f64,
    wpe_over_w: f64,
    gain_ln: f64,
    s: &Shape,
    q: &Electron,
) -> f64 {
    let (mut lo, mut hi) = (1e-9_f64, 0.999_f64);
    for _ in 0..50 {
        let mid = m::sqrt(lo * hi);
        let (_, er, _) = growth(b_t, wpe_over_w, mid, s, q);
        if er > er_kev {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (g, _, vg) = growth(b_t, wpe_over_w, m::sqrt(lo * hi), s, q);
    if g.is_nan() || g <= 0.0 {
        return f64::INFINITY;
    }
    gain_ln * vg / (path_m * g)
}

/// The cap on a belt: the lowest C_K over resonant energies 0.1 to 1 MeV, in
/// a dense plasma (`wpe_over_w` large), where the limit no longer depends on
/// the cold density. In that limit C_K scales exactly as B / (D R_p).
pub fn kp_cap(
    b_t: f64,
    path_m: f64,
    wpe_over_w: f64,
    gain_ln: f64,
    s: &Shape,
    q: &Electron,
) -> f64 {
    [100.0, 200.0, 500.0, 1000.0]
        .iter()
        .map(|e| kp_normalisation(b_t, path_m, *e, wpe_over_w, gain_ln, s, q))
        .fold(f64::INFINITY, f64::min)
}

/// Continuous-slowing-down range of an electron of energy `e_mev`, kg/m²
/// (Katz and Penfold 1952; nearly the same in any light material).
pub fn electron_range(e_mev: f64) -> f64 {
    if e_mev <= 0.0 {
        return 0.0;
    }
    let mg_cm2 = if e_mev <= 2.5 {
        412.0 * m::pow(e_mev, 1.265 - 0.0954 * m::ln(e_mev))
    } else {
        530.0 * e_mev - 106.0
    };
    mg_cm2 * 1e-2
}

/// The energy, MeV, an electron needs to pass a shield of `column` kg/m².
pub fn penetration_energy(column: f64) -> f64 {
    if column <= 0.0 {
        return 0.0;
    }
    let (mut lo, mut hi) = (1e-3_f64, 1e3_f64);
    for _ in 0..60 {
        let mid = m::sqrt(lo * hi);
        if electron_range(mid) > column {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    m::sqrt(lo * hi)
}

/// Share of a stopping electron's energy, `e_mev`, radiated as
/// bremsstrahlung in a material of atomic number `z` (Koch and Motz 1959).
pub fn radiation_yield(e_mev: f64, z: f64, k_per_mev: f64) -> f64 {
    let x = k_per_mev * z * e_mev.max(0.0);
    x / (1.0 + x)
}

/// Photon coefficients for a shield and for tissue behind it, cm²/g, at
/// photon energies `energy_mev` (ascending).
#[derive(Clone, Debug, PartialEq)]
pub struct Photons {
    pub energy_mev: Vec<f64>,
    /// The shield's attenuation μ/ρ.
    pub shield_mu_rho_cm2_g: Vec<f64>,
    /// Tissue's energy absorption μ_en/ρ.
    pub tissue_mu_en_rho_cm2_g: Vec<f64>,
}

impl Photons {
    fn at(xs: &[f64], ys: &[f64], x: f64) -> f64 {
        let n = xs.len().min(ys.len());
        if n == 0 {
            return 0.0;
        }
        if x <= xs[0] {
            return ys[0];
        }
        if x >= xs[n - 1] {
            return ys[n - 1];
        }
        let i = xs[..n].partition_point(|v| *v <= x).max(1);
        let t = m::ln(x / xs[i - 1]) / m::ln(xs[i] / xs[i - 1]);
        m::exp(m::ln(ys[i - 1]) + t * m::ln(ys[i] / ys[i - 1]))
    }
    pub fn shield_mu(&self, k_mev: f64) -> f64 {
        Self::at(&self.energy_mev, &self.shield_mu_rho_cm2_g, k_mev)
    }
    pub fn tissue_mu_en(&self, k_mev: f64) -> f64 {
        Self::at(&self.energy_mev, &self.tissue_mu_en_rho_cm2_g, k_mev)
    }
}

/// How bremsstrahlung is made and carried through a shield.
#[derive(Clone, Debug, PartialEq)]
pub struct Brems<'a> {
    /// Atomic number of the shield and Koch and Motz's k, per MeV.
    pub z: f64,
    pub k_per_mev: f64,
    /// Share of the photons sent inward, toward the dose point.
    pub inward_share: f64,
    pub photons: &'a Photons,
    /// Joules in one MeV (from the electron's charge on the data sheet).
    pub j_per_mev: f64,
}

/// Dose rate, Gy/s, behind a shield of `column` kg/m² from the
/// bremsstrahlung of an omnidirectional electron spectrum `j(e_kev)`, per
/// cm² s keV. Each electron radiates Y(E) E as it stops (a share X/R of it
/// when it crosses the shield), spread evenly over photon energy (Kramers'
/// thick-target spectrum), from halfway along its path; photons cross the
/// rest of the shield with linear build-up and deposit μ_en/ρ of tissue.
pub fn bremsstrahlung_dose(column: f64, j: &dyn Fn(f64) -> f64, b: &Brems) -> f64 {
    let x = column * 0.1;
    if x <= 0.0 {
        return 0.0;
    }
    let (ne, nk) = (120, 24);
    let (lo, hi) = (10.0_f64, 1e5_f64);
    let ln_e = m::ln(hi / lo);
    let mut dose = 0.0;
    for i in 0..ne {
        let e_kev = lo * m::exp(ln_e * (i as f64 + 0.5) / ne as f64);
        let e_mev = e_kev * 1e-3;
        let per = j(e_kev) * e_kev * ln_e / ne as f64;
        if per <= 0.0 || e_mev <= 0.01 {
            continue;
        }
        let range = electron_range(e_mev) * 0.1;
        let made = radiation_yield(e_mev, b.z, b.k_per_mev)
            * e_mev
            * if range > x { x / range } else { 1.0 };
        let rest = x - 0.5 * range.min(x);
        let ln_k = m::ln(e_mev / 0.01);
        let mut s = 0.0;
        for n in 0..nk {
            let k = 0.01 * m::exp(ln_k * (n as f64 + 0.5) / nk as f64);
            let mt = b.photons.shield_mu(k) * rest;
            s += k * ln_k / nk as f64 * m::exp(-mt) * (1.0 + mt) * b.photons.tissue_mu_en(k);
        }
        dose += per * b.inward_share * made / e_mev * s;
    }
    dose * b.j_per_mev * 1e3
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![
        Def {
            id: "sse.belts.radiation_yield",
            version: "1.0.0",
            inputs: vec![port("energy", "belts.electron_energy_mev", D::NONE, None)],
            outputs: vec![port("yield", "belts.radiation_yield", D::NONE, None)],
            params: vec![
                param("z", D::NONE, 13.0, (1.0, 92.0), "atomic number of the shield (aluminium 13)"),
                param("k_per_mev", D::NONE, 6e-4, (3e-4, 1.2e-3), "Koch and Motz's constant, per MeV"),
            ],
            cost: CostClass::Trivial,
            references: &[KOCH59, HUBBELL],
            curious: "An electron stopped in metal shakes out X-rays as it slows. Behind a thick wall those X-rays, not the electrons, give most of the dose, so a heavier wall helps less than you would hope.",
            science: "The radiation yield of a stopping electron, used with Kramers' flat thick-target spectrum and the NIST photon coefficients for the shield and tissue to give the bremsstrahlung dose behind a shield.",
            formula: "Y = k Z T / (1 + k Z T)",
            f: |i, p| Ok(vec![radiation_yield(i[0], p[0], p[1])]),
        }
        .build(),
    ]
}
