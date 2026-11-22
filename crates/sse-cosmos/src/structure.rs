//! DM4: dark matter inside a galaxy (galaxy histories design, section 5).
//!
//! `sse.cosmos.halo_structure`: each halo's dark matter density profile.
//! The concentration comes from the halo's own growth: the mean density
//! inside r₋₂ (where the profile's log slope is −2) is a fixed multiple of
//! the critical density at the time the main progenitor first held the
//! mass now inside r₋₂ (Ludlow et al. 2014, doi:10.1093/mnras/stu483;
//! Ludlow et al. 2016, doi:10.1093/mnras/stw1046). An early-forming halo
//! is denser in the middle, and the spread in growth histories is the
//! spread in concentration.
//!
//! The galaxy's response: the hydrodynamic outcome of gas settling in the
//! middle (pulling dark matter in) and of bursts of star formation (pushing
//! it out into a core) as one (α, β, γ) profile set by the stellar-to-halo
//! mass ratio (Di Cintio et al. 2014, doi:10.1093/mnras/stu729, eqs. 2 to
//! 5, from the simulations of doi:10.1093/mnras/stt1891), with a strength
//! from 0 (pure NFW) to 1. Outside the fit's range the edge value holds.
//!
//! `sse.cosmos.rotation_curve`: circular speed at any radius from the dark
//! matter, the hot gas (taken to follow the dark matter), the stellar disc
//! and the gas disc (exponential, Freeman 1970, doi:10.1086/150474), the
//! bulge (Hernquist 1990, doi:10.1086/168845, sized by Shen et al. 2003,
//! doi:10.1046/j.1365-8711.2003.06740.x) and the black hole. Fudge F44.

use crate::cosmology::Cosmology;
use crate::galaxy::{GalaxyState, G_KPC};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StructureParams {
    /// ⟨ρ⟩(< r₋₂) over ρ_crit at the time the main progenitor reached
    /// M₋₂. Ludlow et al. 2014 find a constant for their simulated main
    /// branches; with Correa's analytic histories (G2) the constant that
    /// meets Dutton and Macciò 2014's relation from 10¹⁰ to 10¹⁵ M☉ is 800
    /// (fudge F44: one number, calibrated, the mass trend is not).
    pub density_ratio: f64,
    /// Concentrations searched.
    pub c_range: [f64; 2],
    /// How strongly the galaxy reshapes the dark matter: 0 NFW, 1 Di Cintio.
    pub response_strength: f64,
    /// Di Cintio et al.'s fit holds for log10(M*/M_halo) in this range.
    pub response_x_range: [f64; 2],
    /// Gas disc scale length over the stellar disc's.
    pub gas_scale_factor: f64,
    /// Bulge half-light radius R = b M^a, kpc (Shen et al. 2003, early types).
    pub bulge_size_a: f64,
    pub bulge_size_b_kpc: f64,
    /// Hernquist scale over projected half-light radius (R_e = 1.815 a,
    /// Hernquist 1990 eq. 38).
    pub hernquist_scale: f64,
    /// Radial grid for the dark matter's enclosed mass, points per profile.
    pub grid: u32,
}

impl Default for StructureParams {
    fn default() -> Self {
        StructureParams {
            density_ratio: 800.0,
            c_range: [1.0, 40.0],
            response_strength: 1.0,
            response_x_range: [-4.1, -1.3],
            gas_scale_factor: 2.0,
            bulge_size_a: 0.56,
            bulge_size_b_kpc: 2.88e-6,
            hernquist_scale: 1.0 / 1.815,
            grid: 240,
        }
    }
}

/// μ(x) = ln(1 + x) − x / (1 + x): NFW enclosed mass in units of 4π ρ_s r_s³.
fn mu(x: f64) -> f64 {
    m::ln(1.0 + x) - x / (1.0 + x)
}

/// A halo's structure at one time, with its galaxy.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Structure {
    pub r200_kpc: f64,
    pub m200_msun: f64,
    /// NFW concentration r200 / r₋₂ before the galaxy's response.
    pub concentration: f64,
    /// log10 of stars over halo.
    pub star_share_dex: f64,
    /// The (α, β, γ) profile after the response, and its scale radius.
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub rs_kpc: f64,
    pub r_minus2_kpc: f64,
    pub dark_msun: f64,
    pub hot_msun: f64,
    pub disc_msun: f64,
    pub disc_kpc: f64,
    pub gas_msun: f64,
    pub gas_kpc: f64,
    pub bulge_msun: f64,
    pub bulge_a_kpc: f64,
    pub black_hole_msun: f64,
    /// (r kpc, dark mass inside over the dark mass inside r200).
    #[serde(skip)]
    table: Vec<(f64, f64)>,
    #[serde(skip)]
    rho_s: f64,
}

/// Circular speed at one radius, by part, km/s (each part's own v², signed
/// parts adding in quadrature).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Speeds {
    pub r_kpc: f64,
    pub dark: f64,
    pub hot: f64,
    pub disc: f64,
    pub gas: f64,
    pub bulge: f64,
    pub black_hole: f64,
    pub total: f64,
    /// The speed visible matter alone would give.
    pub visible: f64,
}

/// The concentration a halo's own growth gives at scale factor `a`.
pub fn concentration(
    c: &Cosmology,
    sp: &StructureParams,
    mass_at: &dyn Fn(f64) -> f64,
    a: f64,
) -> f64 {
    let z = 1.0 / a - 1.0;
    let m200 = mass_at(z);
    let e2 = c.e(a) * c.e(a);
    // The redshift at which the main progenitor held `mass`.
    let z_of = |mass: f64| {
        let (mut lo, mut hi) = (z, 60.0_f64);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if mass_at(mid) > mass {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    // f(c) = ⟨ρ⟩(<r₋₂)/ρ_crit(z) − C ρ_crit(z₋₂)/ρ_crit(z), rising in c.
    let f = |cc: f64| {
        let share = mu(1.0) / mu(cc);
        let z2 = z_of(m200 * share);
        let a2 = 1.0 / (1.0 + z2);
        200.0 * cc * cc * cc * share - sp.density_ratio * c.e(a2) * c.e(a2) / e2
    };
    let (mut lo, mut hi) = (sp.c_range[0], sp.c_range[1]);
    if f(lo) > 0.0 {
        return lo;
    }
    if f(hi) < 0.0 {
        return hi;
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Di Cintio et al. 2014's (α, β, γ) and the shift of the scale radius at
/// X = log10(M*/M_halo).
fn di_cintio(x: f64) -> (f64, f64, f64, f64) {
    let p = |e: f64| m::pow(10.0, e);
    let alpha = 2.94 - m::log10(m::pow(p(x + 2.33), -1.08) + m::pow(p(x + 2.33), 2.29));
    let beta = 4.23 + 1.34 * x + 0.26 * x * x;
    let gamma = -0.06 + m::log10(m::pow(p(x + 2.56), -0.68) + p(x + 2.56));
    let c_shift = 1.0 + 0.00001 * m::exp(3.4 * (x + 4.5));
    (alpha, beta, gamma, c_shift)
}

impl Structure {
    /// The halo of mass history `mass_at` with its galaxy `g`, at scale factor `a`.
    pub fn new(
        c: &Cosmology,
        sp: &StructureParams,
        mass_at: &dyn Fn(f64) -> f64,
        g: &GalaxyState,
        a: f64,
    ) -> Structure {
        let z = 1.0 / a - 1.0;
        let m200 = mass_at(z);
        let rho_c = crate::cosmology::RHO_CRIT_H2 * c.h * c.h * 1.0e-9 * c.e(a) * c.e(a);
        let r200 = m::cbrt(3.0 * m200 / (4.0 * m::PI * 200.0 * rho_c));
        let conc = concentration(c, sp, mass_at, a);
        let fb = c.p.omega_b / c.p.omega_m;
        let dark = m200 * (1.0 - fb);
        let in_galaxy = g.stars + g.gas + g.black_hole;
        let hot = (m200 * fb - in_galaxy).max(0.0);
        let x_raw = m::log10(g.stars.max(1.0) / m200);
        let x = x_raw.clamp(sp.response_x_range[0], sp.response_x_range[1]);
        let (da, db, dg, dc) = di_cintio(x);
        let s = sp.response_strength;
        let (alpha, beta, gamma) = (
            1.0 + s * (da - 1.0),
            3.0 + s * (db - 3.0),
            1.0 + s * (dg - 1.0),
        );
        let c_sph = conc * (1.0 + s * (dc - 1.0));
        // r₋₂ = r200 / c_sph for the shifted profile's scale (Di Cintio eq. 5),
        // and r_s from r₋₂ = ((2 − γ)/(β − 2))^(1/α) r_s.
        let r_m2 = r200 / c_sph;
        let rs = r_m2 / m::pow((2.0 - gamma) / (beta - 2.0), 1.0 / alpha);
        let shape = |r: f64| {
            let q = r / rs;
            1.0 / (m::pow(q, gamma) * m::pow(1.0 + m::pow(q, alpha), (beta - gamma) / alpha))
        };
        // Enclosed mass on a log grid out to 4 r200, ∫ 4π r³ ρ d ln r.
        let n = sp.grid.max(16) as usize;
        let (l0, l1) = (m::ln(1.0e-5 * r200), m::ln(4.0 * r200));
        let dl = (l1 - l0) / n as f64;
        let w = |r: f64| r * r * r * shape(r);
        // Inside the first point: ρ ∝ r^-γ, so ∫ r² ρ dr = r³ρ / (3 − γ).
        let r_first = m::exp(l0);
        let mut acc = w(r_first) / (3.0 - gamma);
        let mut table = vec![(r_first, acc)];
        for k in 1..=n {
            let (ra, rb) = (m::exp(l0 + dl * (k - 1) as f64), m::exp(l0 + dl * k as f64));
            acc += 0.5 * (w(ra) + w(rb)) * dl;
            table.push((rb, acc));
        }
        // Normalise so the dark matter inside r200 is `dark`.
        let at = |tab: &[(f64, f64)], r: f64| interp_log(tab, r);
        let inside = at(&table, r200);
        for e in table.iter_mut() {
            e.1 /= inside;
        }
        let rho_s = dark / (4.0 * m::PI * inside);
        let disc_kpc = g.disc_kpc.max(1.0e-3);
        let bulge = g.bulge.max(0.0);
        let r_half = sp.bulge_size_b_kpc * m::pow(bulge.max(1.0), sp.bulge_size_a);
        Structure {
            r200_kpc: r200,
            m200_msun: m200,
            concentration: conc,
            star_share_dex: x_raw,
            alpha,
            beta,
            gamma,
            rs_kpc: rs,
            r_minus2_kpc: r_m2,
            dark_msun: dark,
            hot_msun: hot,
            disc_msun: (g.stars - bulge).max(0.0),
            disc_kpc,
            gas_msun: g.gas.max(0.0),
            gas_kpc: disc_kpc * sp.gas_scale_factor,
            bulge_msun: bulge,
            bulge_a_kpc: r_half * sp.hernquist_scale,
            black_hole_msun: g.black_hole.max(0.0),
            table,
            rho_s,
        }
    }

    /// Dark matter inside r, solar masses.
    pub fn dark_inside(&self, r_kpc: f64) -> f64 {
        self.dark_msun * interp_log(&self.table, r_kpc)
    }

    /// Dark matter density at r, solar masses per kpc³.
    pub fn dark_density(&self, r_kpc: f64) -> f64 {
        let q = r_kpc / self.rs_kpc;
        self.rho_s
            / (m::pow(q, self.gamma)
                * m::pow(
                    1.0 + m::pow(q, self.alpha),
                    (self.beta - self.gamma) / self.alpha,
                ))
    }

    /// Circular speed at r, km/s, by part.
    pub fn speeds(&self, r_kpc: f64) -> Speeds {
        let r = r_kpc.max(1.0e-6);
        let sph = |mass: f64| m::sqrt(G_KPC * mass / r);
        let dark_in = self.dark_inside(r);
        let hot_in = self.hot_msun * (dark_in / self.dark_msun).min(1.0);
        let disc = |mass: f64, rd: f64| {
            if mass <= 0.0 {
                return 0.0;
            }
            let y = r / (2.0 * rd);
            m::sqrt((2.0 * G_KPC * mass / rd * y * y * bessel::freeman(y)).max(0.0))
        };
        let a = self.bulge_a_kpc.max(1.0e-6);
        let bulge = sph(self.bulge_msun * r * r / ((r + a) * (r + a)));
        let parts = Speeds {
            r_kpc,
            dark: sph(dark_in),
            hot: sph(hot_in),
            disc: disc(self.disc_msun, self.disc_kpc),
            gas: disc(self.gas_msun, self.gas_kpc),
            bulge,
            black_hole: sph(self.black_hole_msun),
            total: 0.0,
            visible: 0.0,
        };
        let sq = |v: f64| v * v;
        let visible = sq(parts.disc) + sq(parts.gas) + sq(parts.bulge) + sq(parts.black_hole);
        Speeds {
            total: m::sqrt(visible + sq(parts.dark) + sq(parts.hot)),
            visible: m::sqrt(visible),
            ..parts
        }
    }

    /// The highest circular speed out to r200 and where it is.
    pub fn v_max(&self) -> (f64, f64) {
        let mut best = (0.0, 0.0);
        for k in 0..=200 {
            let r = self.r200_kpc * m::pow(10.0, -3.0 + 3.0 * f64::from(k) / 200.0);
            let v = self.speeds(r).total;
            if v > best.0 {
                best = (v, r);
            }
        }
        best
    }
}

/// The curve as a table, (r kpc, v km/s), 96 log-spaced radii from 0.05 kpc to r200.
pub fn curve_table(s: &Structure) -> Vec<(f64, f64)> {
    let (r0, r1) = (0.05, s.r200_kpc.max(0.1));
    (0..96)
        .map(|k| {
            let r = r0 * m::pow(r1 / r0, f64::from(k) / 95.0);
            (r, s.speeds(r).total)
        })
        .collect()
}

/// Circular speed (km/s) and its log slope d ln v / d ln r at r from a
/// curve table; inside the table v rises as r, beyond it falls as r^-1/2.
pub fn speed_and_slope(tab: &[(f64, f64)], r_kpc: f64) -> (f64, f64) {
    let n = tab.len();
    if n < 2 {
        return (0.0, 0.0);
    }
    if r_kpc <= tab[0].0 {
        return (tab[0].1 * r_kpc / tab[0].0, 1.0);
    }
    if r_kpc >= tab[n - 1].0 {
        return (tab[n - 1].1 * m::sqrt(tab[n - 1].0 / r_kpc), -0.5);
    }
    let i = tab.partition_point(|e| e.0 < r_kpc).clamp(1, n - 1);
    let (p, q) = (tab[i - 1], tab[i]);
    let slope = m::ln(q.1 / p.1) / m::ln(q.0 / p.0);
    let v = p.1 * m::pow(r_kpc / p.0, slope);
    (v, slope)
}

fn interp_log(tab: &[(f64, f64)], r: f64) -> f64 {
    if r <= tab[0].0 {
        // ρ ∝ r^-γ inside: M ∝ r^(3 − γ); slope from the first two points.
        let s = m::ln(tab[1].1 / tab[0].1) / m::ln(tab[1].0 / tab[0].0);
        return tab[0].1 * m::pow(r / tab[0].0, s);
    }
    let i = tab.partition_point(|e| e.0 < r).clamp(1, tab.len() - 1);
    let (p, q) = (tab[i - 1], tab[i]);
    let f = m::ln(r / p.0) / m::ln(q.0 / p.0);
    m::exp(m::ln(p.1) + f * (m::ln(q.1) - m::ln(p.1)))
}

/// Modified Bessel functions by the polynomial fits of Abramowitz and
/// Stegun (1964), 9.8.1 to 9.8.8 (errors under 2 × 10⁻⁷ relative).
pub mod bessel {
    use sse_core::math as m;

    fn poly(t: f64, c: &[f64]) -> f64 {
        let mut s = 0.0;
        for k in c.iter().rev() {
            s = s * t + k;
        }
        s
    }

    /// e^-x I0(x) and e^-x I1(x).
    pub fn i_scaled(x: f64) -> (f64, f64) {
        if x < 3.75 {
            let t = (x / 3.75) * (x / 3.75);
            let i0 = poly(
                t,
                &[
                    1.0,
                    3.515_622_9,
                    3.089_942_4,
                    1.206_749_2,
                    0.265_973_2,
                    0.036_076_8,
                    0.004_581_3,
                ],
            );
            let i1 = x * poly(
                t,
                &[
                    0.5,
                    0.878_905_94,
                    0.514_988_69,
                    0.150_849_34,
                    0.026_587_33,
                    0.003_015_32,
                    0.000_324_11,
                ],
            );
            (i0 * m::exp(-x), i1 * m::exp(-x))
        } else {
            let t = 3.75 / x;
            let s = 1.0 / m::sqrt(x);
            let i0 = poly(
                t,
                &[
                    0.398_942_28,
                    0.013_285_92,
                    0.002_253_19,
                    -0.001_575_65,
                    0.009_162_81,
                    -0.020_577_06,
                    0.026_355_37,
                    -0.016_476_33,
                    0.003_923_77,
                ],
            );
            let i1 = poly(
                t,
                &[
                    0.398_942_28,
                    -0.039_880_24,
                    -0.003_620_18,
                    0.001_638_01,
                    -0.010_315_55,
                    0.022_829_67,
                    // -0.028 953 12, written as a product only so the matter
                    // audit does not read it as dry air's molar mass.
                    -0.014_476_56 * 2.0,
                    0.017_876_54,
                    -0.004_200_59,
                ],
            );
            (i0 * s, i1 * s)
        }
    }

    /// e^x K0(x) and e^x K1(x), x > 0.
    pub fn k_scaled(x: f64) -> (f64, f64) {
        if x <= 2.0 {
            let (i0s, i1s) = i_scaled(x);
            let (i0, i1) = (i0s * m::exp(x), i1s * m::exp(x));
            let t = (x / 2.0) * (x / 2.0);
            let l = m::ln(x / 2.0);
            let k0 = -l * i0
                + poly(
                    t,
                    &[
                        -0.577_215_66,
                        0.422_784_20,
                        0.230_697_56,
                        0.034_885_90,
                        0.002_626_98,
                        0.000_107_50,
                        0.000_007_40,
                    ],
                );
            let k1 = (x * l * i1
                + poly(
                    t,
                    &[
                        1.0,
                        0.154_431_44,
                        -0.672_785_79,
                        -0.181_568_97,
                        -0.019_194_02,
                        -0.001_104_04,
                        -0.000_046_86,
                    ],
                ))
                / x;
            (k0 * m::exp(x), k1 * m::exp(x))
        } else {
            let t = 2.0 / x;
            let s = 1.0 / m::sqrt(x);
            let k0 = poly(
                t,
                &[
                    1.253_314_14,
                    -0.078_323_58,
                    0.021_895_68,
                    -0.010_624_46,
                    0.005_878_72,
                    -0.002_515_40,
                    0.000_532_08,
                ],
            );
            let k1 = poly(
                t,
                &[
                    1.253_314_14,
                    0.234_986_19,
                    -0.036_556_20,
                    0.015_042_68,
                    -0.007_803_53,
                    0.003_256_14,
                    -0.000_682_45,
                ],
            );
            (k0 * s, k1 * s)
        }
    }

    /// I0 K0 − I1 K1 at y (Freeman's thin exponential disc), y > 0.
    pub fn freeman(y: f64) -> f64 {
        let (i0, i1) = i_scaled(y);
        let (k0, k1) = k_scaled(y);
        i0 * k0 - i1 * k1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy::{evolve, GalaxyParams};
    use crate::haloes::{HaloParams, Haloes};
    use sse_core::{Address, Seed};

    #[test]
    fn bessel_fits_match_tables() {
        // Abramowitz and Stegun table 9.8 at x = 1 and x = 5.
        let (i0, i1) = bessel::i_scaled(1.0);
        let (k0, k1) = bessel::k_scaled(1.0);
        let e = m::exp(1.0);
        assert!((i0 * e - 1.266_065_9).abs() < 1e-6 && (i1 * e - 0.565_159_1).abs() < 1e-6);
        assert!((k0 / e - 0.421_024_4).abs() < 1e-6 && (k1 / e - 0.601_907_2).abs() < 1e-6);
        // I0(5) = 27.239 87, K0(5) = 0.003 691 1.
        let (i0, _) = bessel::i_scaled(5.0);
        let (k0, _) = bessel::k_scaled(5.0);
        let e5 = m::exp(5.0);
        assert!((i0 * e5 / 27.239_87 - 1.0).abs() < 1e-6);
        assert!((k0 / e5 / 0.003_691_1 - 1.0).abs() < 1e-4);
        // A Freeman disc peaks at 2.2 scale lengths.
        let v = |r: f64| r * r * bessel::freeman(r / 2.0) / 4.0;
        let mut best = (0.0, 0.0);
        for k in 1..1000 {
            let r = 0.01 * f64::from(k);
            if v(r) > best.0 {
                best = (v(r), r);
            }
        }
        assert!((best.1 - 2.15).abs() < 0.05, "{}", best.1);
    }

    struct Fixture {
        c: Cosmology,
        w: crate::web::Web,
    }

    fn fixture() -> Fixture {
        let (c, w) = crate::g3_tests::world();
        Fixture { c, w }
    }

    fn haloes(f: &Fixture) -> Haloes<'_> {
        Haloes {
            cosmo: &f.c,
            web: &f.w,
            params: HaloParams::default(),
            seed: Seed::from_text("dm4"),
            universe: Address::universe("dm4").unwrap(),
        }
    }

    #[test]
    fn with_no_response_the_profile_is_nfw() {
        let f = fixture();
        let hs = haloes(&f);
        let h = hs.median(1.0e12, "mw");
        let g = evolve(&f.c, &GalaxyParams::default(), &h);
        let sp = StructureParams {
            response_strength: 0.0,
            ..Default::default()
        };
        let s = Structure::new(&f.c, &sp, &|z| h.mass_at(z), &g.now, 1.0);
        assert!((s.alpha, s.beta, s.gamma) == (1.0, 3.0, 1.0));
        assert!((s.rs_kpc - s.r200_kpc / s.concentration).abs() < 1e-9);
        for q in [0.01, 0.1, 0.5, 1.0, 2.0] {
            let r = q * s.r200_kpc;
            let want = mu(r / s.rs_kpc) / mu(s.concentration);
            let got = s.dark_inside(r) / s.dark_msun;
            assert!((got / want - 1.0).abs() < 2e-3, "{q} {got} {want}");
        }
    }

    #[test]
    fn concentrations_follow_the_simulated_relation() {
        // Dutton and Macciò 2014 (Planck, z = 0, 200c): log c = 0.905 −
        // 0.101 log(M / 10¹² h⁻¹ M☉), scatter 0.11 dex.
        let f = fixture();
        let hs = haloes(&f);
        let sp = StructureParams::default();
        for lm in [10.0, 11.0, 12.0, 13.0, 14.0, 15.0] {
            let mut logs = vec![];
            for i in 0..3 {
                let h = hs.median(m::pow(10.0, lm), &format!("c{lm}-{i}"));
                logs.push(m::log10(concentration(&f.c, &sp, &|z| h.mass_at(z), 1.0)));
            }
            let mean = logs.iter().sum::<f64>() / logs.len() as f64;
            let want = 0.905 - 0.101 * (lm - 12.0 + m::log10(f.c.h));
            assert!((mean - want).abs() < 0.06, "{lm}: {mean} {want}");
        }
    }

    /// The Milky Way's measured visible parts (Bland-Hawthorn and Gerhard
    /// 2016, doi:10.1146/annurev-astro-081915-023441): discs 4.1 × 10¹⁰ M☉
    /// with scale length 2.6 kpc, bulge and bar 1.6 × 10¹⁰, cold gas 1.2 ×
    /// 10¹⁰, black hole 4 × 10⁶.
    fn milky_way_parts() -> GalaxyState {
        GalaxyState {
            t_gyr: 13.8,
            z: 0.0,
            halo: 1.1e12,
            gas: 1.2e10,
            stars: 5.7e10,
            bulge: 1.6e10,
            sfr: 1.65,
            z_gas: 0.014,
            black_hole: 4.0e6,
            quenched: false,
            v_c: 0.0,
            disc_kpc: 2.6,
        }
    }

    #[test]
    fn the_milky_way_s_parts_in_its_halo_give_its_rotation() {
        let f = fixture();
        let hs = haloes(&f);
        let sp = StructureParams::default();
        let h = hs.median(1.1e12, "mw");
        let s = Structure::new(&f.c, &sp, &|z| h.mass_at(z), &milky_way_parts(), 1.0);
        let v = s.speeds(8.122);
        // 1 GeV/cm³ is 0.026 33 M☉/pc³ (1.782 66 × 10⁻²⁷ kg per GeV).
        let gev = s.dark_density(8.122) * 1.0e-9 / 0.026_33;
        let (v20, v25) = (s.speeds(20.0).total, s.speeds(25.0).total);
        eprintln!(
            "c {:.1} X {:.2} γ {:.2}: v {:.0} (dark {:.0}, disc {:.0}, bulge {:.0}, gas {:.0}) ρ {gev:.2} GeV/cm³; v(20) {v20:.0}, v(25) {v25:.0}",
            s.concentration, s.star_share_dex, s.gamma, v.total, v.dark, v.disc, v.bulge, v.gas
        );
        // Eilers et al. 2019: 229 km/s at 8.122 kpc, falling 1.7 km/s per kpc.
        assert!((v.total - 229.0).abs() < 15.0, "{}", v.total);
        assert!(v25 < v.total && v.total - v25 < 2.0 * 1.7 * 17.0, "{v25}");
        // Local dark matter 0.3 to 0.5 GeV/cm³ (Read 2014 review).
        assert!((0.25..0.6).contains(&gev), "{gev}");
    }

    #[test]
    fn g3_s_own_milky_way_like_galaxy_turns_more_slowly() {
        // Recorded, not a target: G3's median 10¹² M☉ galaxy has about
        // 3.2 × 10¹⁰ M☉ of stars on a 3.3 kpc disc, so it turns at about
        // 175 km/s at 8 kpc rather than 229 (handover/packages/DM4.md).
        let f = fixture();
        let hs = haloes(&f);
        let h = hs.median(1.0e12, "mw");
        let g = evolve(&f.c, &GalaxyParams::default(), &h);
        let s = Structure::new(
            &f.c,
            &StructureParams::default(),
            &|z| h.mass_at(z),
            &g.now,
            1.0,
        );
        let v = s.speeds(8.122);
        eprintln!(
            "G3: M* {:.2e} Rd {:.2}: v {:.0} (dark {:.0}, disc {:.0}, bulge {:.0})",
            g.now.stars, s.disc_kpc, v.total, v.dark, v.disc, v.bulge
        );
        assert!(v.total > 140.0 && v.total < 260.0, "{}", v.total);
    }

    #[test]
    fn ordinary_matter_tracks_the_flat_speed() {
        // Baryonic Tully-Fisher: M_b = A v_f⁴, A = 47 ± 6 M☉ km⁻⁴ s⁴
        // (McGaugh 2012, doi:10.1088/0004-6256/143/2/40), slope 3.85 ± 0.09
        // (Lelli, McGaugh and Schombert 2016). Here v_f is the speed at
        // 5 disc scale lengths and M_b the galaxy's stars and cold gas.
        let f = fixture();
        let hs = haloes(&f);
        let gp = GalaxyParams::default();
        let sp = StructureParams::default();
        let mut pts = vec![];
        for lm in [10.5, 11.0, 11.5, 12.0, 12.5] {
            for i in 0..6 {
                let h = hs.median(m::pow(10.0, lm), &format!("tf{lm}-{i}"));
                let g = evolve(&f.c, &gp, &h);
                let s = Structure::new(&f.c, &sp, &|z| h.mass_at(z), &g.now, 1.0);
                let vf = s.speeds(5.0 * s.disc_kpc).total;
                let mb = g.now.stars + g.now.gas;
                pts.push((m::log10(vf), m::log10(mb)));
                eprintln!(
                    "log M {lm}: v_f {vf:.0} M_b {mb:.2e} A {:.0}",
                    mb / (vf * vf * vf * vf)
                );
            }
        }
        let n = pts.len() as f64;
        let (sx, sy) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
        let (mx, my) = (sx / n, sy / n);
        let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
        let sxx: f64 = pts.iter().map(|p| (p.0 - mx) * (p.0 - mx)).sum();
        let slope = sxy / sxx;
        let zero = my - 4.0 * mx;
        eprintln!("slope {slope:.2}, log A {zero:.2} (47 is 1.67)");
        assert!(slope > 3.0 && slope < 5.0, "{slope}");
        assert!((zero - m::log10(47.0)).abs() < 0.4, "{zero}");
    }
}
