//! Single-star evolution fits of Hurley, Pols and Tout 2000 (HPT00, MNRAS 315,
//! 543), main-sequence part, with the zero-age fits of Tout et al. 1996 (MNRAS
//! 281, 257). Masses in solar masses, times in Myr, luminosity and radius in
//! solar units, metallicity Z as a mass fraction (fits valid 1e-4 to 0.03,
//! masses 0.1 to 100).
//!
//! Ported from the SSE routines `zcnsts`, `zfuncs` and the main-sequence branch
//! of `hrdiag` as distributed in COSMIC 4.3.4 (MIT licence, credited in
//! NOTICE). Array indices follow the Fortran (`a[i]` is `msp(i)`), so each line
//! can be checked against the source and against the paper's appendix. All
//! maths goes through `sse_core::math`.

pub mod coeffs;

use coeffs::*;
use sse_core::math as m;

/// Hurley's reference metallicity: the fits are in `log10(Z / 0.02)`. HPT00
/// fit the Pols et al. 1998 models, in which the Sun is Z = 0.02 (the older
/// Anders and Grevesse 1989 solar mix). This is the fits' own scale, not a
/// measured metal fraction.
pub const Z_REF: f64 = 0.02;
/// The Sun's metal mass fraction as measured (protosolar, Asplund et al. 2009,
/// ARA&A 47, 481). Every Z in SSE4 is on this measured scale; the fits are read
/// at the same [Fe/H], Z_fit = Z x Z_REF / Z_SUN, so a star of solar
/// composition is the fits' Sun. Until 27 Nov 2026 measured Z went straight in,
/// which read the Sun as metal-poor ([Fe/H] -0.15) and 13 percent too bright.
pub const Z_SUN: f64 = 0.0142;

/// The fits' Z for a measured metal mass fraction (the same [Fe/H]).
pub fn fit_z(z: f64) -> f64 {
    z * Z_REF / Z_SUN
}

/// Fortran-style 1-based polynomial in `x`: `c[s] + x (c[s+1] + x (...))`
/// over `n` coefficients starting at Fortran index `s`.
fn poly(c: &[f64], s: usize, n: usize, x: f64) -> f64 {
    let mut v = 0.0;
    for k in (0..n).rev() {
        v = c[s - 1 + k] + x * v;
    }
    v
}

fn pw(x: f64, y: f64) -> f64 {
    m::pow(x, y)
}

/// The metallicity-dependent constants of one Z (`zpars`, `msp`, `gbp`).
#[derive(Clone, Debug)]
pub struct Fits {
    /// Measured metal mass fraction (the scale of every Z in SSE4).
    pub z: f64,
    /// The same composition on the fits' scale (`fit_z`), which the
    /// coefficients below are worked from.
    pub z_fit: f64,
    /// `zpars(1..)`: 1 Mhook, 2 MHeF, 3 MFGB, 6 log D, 8 x for tMS, 11 X, 12 Y.
    pub zpars: [f64; 21],
    a: [f64; 201],
    g: [f64; 201],
}

impl Fits {
    /// The fits for a star of measured metal mass fraction `z`.
    pub fn new(z_measured: f64) -> Self {
        let z = fit_z(z_measured);
        let lzs = m::log10(z / Z_REF);
        let lz = m::log10(z);
        let mut zp = [0.0; 21];
        zp[1] = 1.0185 + lzs * (0.16015 + lzs * 0.0892);
        zp[2] = 1.995 + lzs * (0.25 + lzs * 0.087);
        zp[3] = 16.5 * pw(z, 0.06) / (1.0 + pw(1.0e-4 / z, 1.27));
        zp[4] = (6.11044 + 1.02167 * lzs).max(5.0);
        zp[5] = zp[4] + 1.8;
        zp[6] = 5.37 + lzs * 0.135;
        zp[8] = 0.95_f64.max(
            (0.95 - (10.0 / 3.0) * (z - 0.01))
                .max(0.99_f64.min(0.98 - (100.0 / 7.0) * (z - 0.001))),
        );
        zp[11] = 0.76 - 3.0 * z;
        zp[12] = 0.24 + 2.0 * z;

        let mut a = [0.0; 201];
        // Lzams, Rzams (Tout et al. 1996)
        for i in 0..7 {
            a[1 + i] = poly(&XZ, 1 + 5 * i, 5, lzs);
        }
        for i in 0..5 {
            a[8 + i] = poly(&XZ, 36 + 5 * i, 5, lzs);
        }
        a[13] = XZ[60];
        for i in 0..3 {
            a[14 + i] = poly(&XZ, 62 + 5 * i, 5, lzs);
        }
        // Tbgb
        for i in 0..4 {
            a[17 + i] = poly(&XT, 1 + 4 * i, 4, lzs);
        }
        a[21] = XT[16];
        // Thook
        a[22] = poly(&XT, 18, 4, lzs);
        a[23] = XT[21];
        a[24] = poly(&XT, 23, 4, lzs);
        a[25] = poly(&XT, 27, 4, lzs);
        a[26] = XT[30];
        // Ltms
        a[27] = poly(&XL, 1, 5, lzs);
        a[28] = poly(&XL, 6, 5, lzs);
        a[29] = poly(&XL, 11, 4, lzs);
        a[30] = poly(&XL, 15, 5, lzs);
        a[27] *= a[30];
        a[28] *= a[30];
        a[31] = poly(&XL, 20, 4, lzs);
        a[32] = poly(&XL, 24, 4, lzs);
        // Lalpha
        let m2 = 2.0;
        a[33] = poly(&XL, 28, 4, lzs);
        a[34] = poly(&XL, 32, 4, lzs);
        a[35] = poly(&XL, 36, 4, lzs);
        a[36] = poly(&XL, 40, 4, lzs);
        a[37] = 0.9_f64.max(1.1064 + lzs * (0.415 + 0.18 * lzs));
        a[38] = 1.0_f64.max(1.19 + lzs * (0.377 + 0.176 * lzs));
        if z > 0.01 {
            a[37] = a[37].min(1.0);
            a[38] = a[38].min(1.1);
        }
        a[39] = 0.145_f64.max(0.0977 - lzs * (0.231 + 0.0753 * lzs));
        a[40] = (0.24 + lzs * (0.18 + 0.595 * lzs)).min(0.306 + 0.053 * lzs);
        a[41] = (0.33 + lzs * (0.132 + 0.218 * lzs)).min(0.3625 + 0.062 * lzs);
        a[42] = (a[33] + a[34] * pw(m2, a[36])) / (pw(m2, 0.4) + a[35] * pw(m2, 1.9));
        // Lbeta
        a[43] = poly(&XL, 44, 5, lzs);
        a[44] = poly(&XL, 49, 5, lzs);
        a[45] = poly(&XL, 54, 3, lzs);
        a[46] = 1.4_f64.min(1.5135 + 0.3769 * lzs);
        a[46] = (0.6355 - 0.4192 * lzs).max(1.25_f64.max(a[46]));
        // Lhook
        a[47] = poly(&XL, 57, 4, lzs);
        a[48] = poly(&XL, 61, 4, lzs);
        a[49] = poly(&XL, 65, 4, lzs);
        a[50] = poly(&XL, 69, 4, lzs);
        a[51] = 1.4_f64.min(1.5135 + 0.3769 * lzs);
        a[51] = (0.6355 - 0.4192 * lzs).max(1.25_f64.max(a[51]));
        // Rtms
        a[52] = poly(&XR, 1, 5, lzs);
        a[53] = poly(&XR, 6, 5, lzs);
        a[54] = poly(&XR, 11, 5, lzs);
        a[55] = poly(&XR, 16, 4, lzs);
        a[56] = poly(&XR, 20, 4, lzs);
        a[52] *= a[54];
        a[53] *= a[54];
        a[57] = XR[23];
        a[58] = poly(&XR, 25, 4, lzs);
        a[59] = poly(&XR, 29, 4, lzs);
        a[60] = poly(&XR, 33, 4, lzs);
        a[61] = poly(&XR, 37, 4, lzs);
        a[62] = (0.097 - 0.1072 * (lz + 3.0))
            .max(0.097_f64.max(0.1461_f64.min(0.1461 + 0.1237 * (lz + 2.0))));
        a[62] = pw(10.0, a[62]);
        let m2 = a[62] + 0.1;
        a[63] = (a[52] + a[53] * pw(a[62], a[55])) / (a[54] + pw(a[62], a[56]));
        a[64] = (a[57] * m2 * m2 * m2 + a[58] * pw(m2, a[61]) + a[59] * pw(m2, a[61] + 1.5))
            / (a[60] + pw(m2, 5.0));
        // Ralpha
        a[65] = poly(&XR, 41, 4, lzs);
        a[66] = poly(&XR, 45, 4, lzs);
        a[67] = poly(&XR, 49, 4, lzs);
        a[68] = poly(&XR, 53, 4, lzs);
        a[69] = poly(&XR, 57, 5, lzs);
        a[70] = 0.9_f64.max(1.0_f64.min(1.116 + 0.166 * lzs));
        a[71] = (1.477 + 0.296 * lzs).max(1.6_f64.min(-0.308 - 1.046 * lzs));
        a[71] = 0.8_f64.max((0.8 - 2.0 * lzs).min(a[71]));
        a[72] = poly(&XR, 62, 3, lzs);
        a[73] = 0.065_f64.max(0.0843 - lzs * (0.0475 + 0.0352 * lzs));
        a[74] = 0.0736 + lzs * (0.0749 + 0.04426 * lzs);
        if z < 0.004 {
            a[74] = 0.055_f64.min(a[74]);
        }
        a[75] = 0.091_f64.max(0.121_f64.min(0.136 + 0.0352 * lzs));
        a[76] = (a[65] * pw(a[71], a[67])) / (a[66] + pw(a[71], a[68]));
        if a[70] > a[71] {
            a[70] = a[71];
            a[75] = a[76];
        }
        // Rbeta
        a[77] = poly(&XR, 65, 4, lzs);
        a[78] = poly(&XR, 69, 4, lzs);
        a[79] = poly(&XR, 73, 4, lzs);
        a[80] = poly(&XR, 77, 4, lzs);
        a[81] = XR[80] + lzs * (XR[81] + lzs * lzs * XR[82]);
        if z > 0.01 {
            a[81] = a[81].max(0.95);
        }
        a[82] = 1.4_f64.max(1.6_f64.min(1.6 + lzs * (0.764 + 0.3322 * lzs)));
        // Rgamma
        a[83] = poly(&XR, 84, 4, lzs).max(poly(&XR, 96, 3, lzs));
        a[84] = 0.0_f64.min(poly(&XR, 88, 4, lzs));
        a[84] = a[84].max(poly(&XR, 99, 3, lzs));
        a[85] = poly(&XR, 92, 4, lzs);
        a[85] = 0.0_f64.max(a[85].min(7.454 + 9.046 * lzs));
        a[86] = (XR[101] + lzs * XR[102]).min(2.0_f64.max(-13.3 - 18.6 * lzs));
        a[87] = 1.5_f64.min(0.4_f64.max(2.493 + 1.1475 * lzs));
        a[88] = 1.0_f64.max(1.27_f64.min(0.8109 - 0.6282 * lzs));
        a[88] = a[88].max(0.6355 - 0.4192 * lzs);
        a[89] = 5.855420e-2_f64.max(-0.2711 - lzs * (0.5756 + 0.0838 * lzs));
        // Rhook
        a[90] = poly(&XR, 104, 4, lzs);
        a[91] = poly(&XR, 108, 4, lzs);
        a[92] = poly(&XR, 112, 4, lzs);
        a[93] = poly(&XR, 116, 4, lzs);
        a[94] = 1.25_f64.min(1.1_f64.max(1.9848 + lzs * (1.1386 + 0.3564 * lzs)));
        a[95] = 0.063 + lzs * (0.0481 + 0.00984 * lzs);
        a[96] = 1.3_f64.min(0.45_f64.max(1.2 + 2.45 * lzs));
        // Lneta
        a[97] = if z > 0.0009 { 10.0 } else { 20.0 };

        let mut g = [0.0; 201];
        // Lbgb
        g[1] = poly(&XG, 1, 4, lzs);
        g[2] = poly(&XG, 5, 4, lzs);
        g[3] = poly(&XG, 9, 4, lzs);
        g[4] = poly(&XG, 13, 4, lzs);
        g[5] = poly(&XG, 17, 3, lzs);
        g[6] = poly(&XG, 20, 3, lzs);
        g[3] = pw(g[3], g[6]);
        g[7] = XG[22];
        g[8] = XG[23];
        // Rgb
        g[17] = pw(10.0, -4.6739 - 0.9394 * lz);
        g[17] = g[17].max(-0.04167 + 55.67 * z);
        g[17] = g[17].min(0.4771 - 9329.21 * pw(z, 2.94));
        g[18] = 0.54_f64.min(0.397 + lzs * (0.28826 + 0.5293 * lzs));
        g[19] = pw(
            10.0,
            (-0.1451_f64).max(-2.2794 - lz * (1.5175 + 0.254 * lz)),
        );
        if z > 0.004 {
            g[19] = g[19].max(0.7307 + 14265.1 * pw(z, 3.395));
        }
        g[20] = poly(&XG, 45, 6, lzs);
        g[21] = poly(&XG, 51, 5, lzs);
        g[22] = poly(&XG, 56, 6, lzs);
        g[23] = poly(&XG, 62, 5, lzs);
        Fits {
            z: z_measured,
            z_fit: z,
            zpars: zp,
            a,
            g,
        }
    }

    /// ZAMS luminosity (Tout et al. 1996 eq. 1).
    pub fn l_zams(&self, mass: f64) -> f64 {
        let a = &self.a;
        let mx = m::sqrt(mass);
        (a[1] * pw(mass, 5.0) * mx + a[2] * pw(mass, 11.0))
            / (a[3]
                + pw(mass, 3.0)
                + a[4] * pw(mass, 5.0)
                + a[5] * pw(mass, 7.0)
                + a[6] * pw(mass, 8.0)
                + a[7] * pw(mass, 9.0) * mx)
    }

    /// ZAMS radius (Tout et al. 1996 eq. 2).
    pub fn r_zams(&self, mass: f64) -> f64 {
        let a = &self.a;
        let mx = m::sqrt(mass);
        ((a[8] * mass * mass + a[9] * pw(mass, 6.0)) * mx
            + a[10] * pw(mass, 11.0)
            + (a[11] + a[12] * mx) * pw(mass, 19.0))
            / (a[13]
                + a[14] * mass * mass
                + (a[15] * pw(mass, 8.0) + pw(mass, 18.0) + a[16] * pw(mass, 19.0)) * mx)
    }

    /// Time to the base of the giant branch (or to helium ignition), Myr (HPT00 eq. 4).
    pub fn t_bgb(&self, mass: f64) -> f64 {
        let a = &self.a;
        (a[17] + a[18] * pw(mass, 4.0) + a[19] * pw(mass, 5.5) + pw(mass, 7.0))
            / (a[20] * mass * mass + a[21] * pw(mass, 7.0))
    }

    /// Hook time as a fraction of t_BGB (HPT00 eq. 7).
    pub fn thook_frac(&self, mass: f64) -> f64 {
        let a = &self.a;
        let v = 1.0 - 0.01 * (a[22] / pw(mass, a[23])).max(a[24] + a[25] / pw(mass, a[26]));
        v.max(0.5)
    }

    /// Main-sequence lifetime, Myr (HPT00 eq. 5).
    pub fn t_ms(&self, mass: f64) -> f64 {
        self.zpars[8].max(self.thook_frac(mass)) * self.t_bgb(mass)
    }

    /// Terminal-age main-sequence luminosity (HPT00 eq. 8).
    pub fn l_tms(&self, mass: f64) -> f64 {
        let a = &self.a;
        (a[27] * pw(mass, 3.0) + a[28] * pw(mass, 4.0) + a[29] * pw(mass, a[32] + 1.8))
            / (a[30] + a[31] * pw(mass, 5.0) + pw(mass, a[32]))
    }

    /// Terminal-age main-sequence radius (HPT00 eq. 9, SSE's `rtmssse`).
    pub fn r_tms(&self, mass: f64) -> f64 {
        let a = &self.a;
        let m2 = a[62] + 0.1;
        if mass <= a[62] {
            let rchk = 1.5 * self.r_zams(mass);
            rchk.max((a[52] + a[53] * pw(mass, a[55])) / (a[54] + pw(mass, a[56])))
        } else if mass >= m2 {
            (a[57] * pw(mass, 3.0) + a[58] * pw(mass, a[61]) + a[59] * pw(mass, a[61] + 1.5))
                / (a[60] + pw(mass, 5.0))
        } else {
            a[63] + ((a[64] - a[63]) / 0.1) * (mass - a[62])
        }
    }

    fn l_alpha(&self, mass: f64) -> f64 {
        let a = &self.a;
        let mcut = 2.0;
        if mass >= mcut {
            (a[33] + a[34] * pw(mass, a[36])) / (pw(mass, 0.4) + a[35] * pw(mass, 1.9))
        } else if mass <= 0.5 {
            a[39]
        } else if mass <= 0.7 {
            a[39] + ((0.3 - a[39]) / 0.2) * (mass - 0.5)
        } else if mass <= a[37] {
            0.3 + ((a[40] - 0.3) / (a[37] - 0.7)) * (mass - 0.7)
        } else if mass <= a[38] {
            a[40] + ((a[41] - a[40]) / (a[38] - a[37])) * (mass - a[37])
        } else {
            a[41] + ((a[42] - a[41]) / (mcut - a[38])) * (mass - a[38])
        }
    }

    fn l_beta(&self, mass: f64) -> f64 {
        let a = &self.a;
        let mut b = (a[43] - a[44] * pw(mass, a[45])).max(0.0);
        if mass > a[46] && b > 0.0 {
            let a1 = a[43] - a[44] * pw(a[46], a[45]);
            b = (a1 - 10.0 * a1 * (mass - a[46])).max(0.0);
        }
        b
    }

    fn l_eta(&self, mass: f64) -> f64 {
        let v = if mass <= 1.0 {
            10.0
        } else if mass >= 1.1 {
            20.0
        } else {
            10.0 + 100.0 * (mass - 1.0)
        };
        v.min(self.a[97])
    }

    fn l_hook(&self, mass: f64) -> f64 {
        let a = &self.a;
        let mhook = self.zpars[1];
        if mass <= mhook {
            0.0
        } else if mass >= a[51] {
            (a[47] / pw(mass, a[48])).min(a[49] / pw(mass, a[50]))
        } else {
            let a2 = (a[47] / pw(a[51], a[48])).min(a[49] / pw(a[51], a[50]));
            a2 * pw((mass - mhook) / (a[51] - mhook), 0.4)
        }
    }

    fn r_alpha(&self, mass: f64) -> f64 {
        let a = &self.a;
        if mass <= 0.5 {
            a[73]
        } else if mass <= 0.65 {
            a[73] + ((a[74] - a[73]) / 0.15) * (mass - 0.5)
        } else if mass <= a[70] {
            a[74] + ((a[75] - a[74]) / (a[70] - 0.65)) * (mass - 0.65)
        } else if mass <= a[71] {
            a[75] + ((a[76] - a[75]) / (a[71] - a[70])) * (mass - a[70])
        } else if mass <= a[72] {
            (a[65] * pw(mass, a[67])) / (a[66] + pw(mass, a[68]))
        } else {
            let a5 = (a[65] * pw(a[72], a[67])) / (a[66] + pw(a[72], a[68]));
            a5 + a[69] * (mass - a[72])
        }
    }

    fn r_beta(&self, mass: f64) -> f64 {
        let a = &self.a;
        let (m2, m3) = (2.0, 16.0);
        let b = if mass <= 1.0 {
            1.06
        } else if mass <= a[82] {
            1.06 + ((a[81] - 1.06) / (a[82] - 1.0)) * (mass - 1.0)
        } else if mass <= m2 {
            let b2 = (a[77] * pw(m2, 3.5)) / (a[78] + pw(m2, a[79]));
            a[81] + ((b2 - a[81]) / (m2 - a[82])) * (mass - a[82])
        } else if mass <= m3 {
            (a[77] * pw(mass, 3.5)) / (a[78] + pw(mass, a[79]))
        } else {
            let b3 = (a[77] * pw(m3, 3.5)) / (a[78] + pw(m3, a[79]));
            b3 + a[80] * (mass - m3)
        };
        b - 1.0
    }

    fn r_gamma(&self, mass: f64) -> f64 {
        let a = &self.a;
        let m1 = 1.0;
        if mass > a[88] + 0.1 {
            return 0.0;
        }
        let mut b1 = 0.0_f64.max(a[83] + a[84] * pw(m1 - a[85], a[86]));
        let v = if mass <= m1 {
            a[83] + a[84] * pw((mass - a[85]).abs(), a[86])
        } else if mass <= a[88] {
            b1 + (a[89] - b1) * pw((mass - m1) / (a[88] - m1), a[87])
        } else {
            if a[88] > m1 {
                b1 = a[89];
            }
            b1 - 10.0 * b1 * (mass - a[88])
        };
        v.max(0.0)
    }

    fn r_hook(&self, mass: f64) -> f64 {
        let a = &self.a;
        let mhook = self.zpars[1];
        if mass <= mhook {
            0.0
        } else if mass <= a[94] {
            a[95] * m::sqrt((mass - mhook) / (a[94] - mhook))
        } else if mass <= 2.0 {
            let m2 = 2.0;
            let b2 = (a[90] + a[91] * pw(m2, 3.5)) / (a[92] * pw(m2, 3.0) + pw(m2, a[93])) - 1.0;
            a[95] + (b2 - a[95]) * pw((mass - a[94]) / (m2 - a[94]), a[96])
        } else {
            (a[90] + a[91] * pw(mass, 3.5)) / (a[92] * pw(mass, 3.0) + pw(mass, a[93])) - 1.0
        }
    }

    /// Luminosity at the base of the giant branch (HPT00 eq. 10; low and
    /// intermediate masses, M below MFGB).
    pub fn l_bgb(&self, mass: f64) -> f64 {
        let g = &self.g;
        (g[1] * pw(mass, g[5]) + g[2] * pw(mass, g[8]))
            / (g[3] + g[4] * pw(mass, g[7]) + pw(mass, g[6]))
    }

    /// Giant-branch radius at luminosity `lum` (HPT00 eq. 46).
    pub fn r_gb(&self, mass: f64, lum: f64) -> f64 {
        let g = &self.g;
        let a1 = (g[20] / pw(mass, g[21])).min(g[22] / pw(mass, g[23]));
        a1 * (pw(lum, g[18]) + g[17] * pw(lum, g[19]))
    }

    /// Luminosity and radius on the main sequence at age `t` Myr (`hrdiag`,
    /// HPT00 eqs. 12 and 13 with the hook terms). `None` past the main sequence.
    pub fn ms_state(&self, mass: f64, t: f64) -> Option<(f64, f64)> {
        let tbgb = self.t_bgb(mass);
        let tm = self.t_ms(mass);
        if !(0.0..tm).contains(&t) {
            return None;
        }
        let taumin = 5.0e-8;
        let tau = t / tm;
        let thook = self.thook_frac(mass) * tbgb;
        let zeta = 0.01;
        let tau1 = (t / thook).min(1.0);
        let tau2 = ((t - (1.0 - zeta) * thook) / (zeta * thook)).clamp(0.0, 1.0);
        let (lz, lt) = (self.l_zams(mass), self.l_tms(mass));
        let dell = self.l_hook(mass);
        let dtau = tau1 * tau1 - tau2 * tau2;
        let alpha = self.l_alpha(mass);
        let beta = self.l_beta(mass);
        let eta = self.l_eta(mass);
        let lx = m::log10(lt / lz);
        let xx = if tau > taumin {
            alpha * tau + beta * pw(tau, eta) + (lx - alpha - beta) * tau * tau - dell * dtau
        } else {
            alpha * tau + (lx - alpha) * tau * tau - dell * dtau
        };
        let lum = lz * pw(10.0, xx);

        let (rz, rt) = (self.r_zams(mass), self.r_tms(mass));
        let delr = self.r_hook(mass);
        let dtau = tau1 * tau1 * tau1 - tau2 * tau2 * tau2;
        let alpha = self.r_alpha(mass);
        let beta = self.r_beta(mass);
        let gamma = self.r_gamma(mass);
        let rx = m::log10(rt / rz);
        let xx = if tau > taumin {
            alpha * tau
                + beta * pw(tau, 10.0)
                + gamma * pw(tau, 40.0)
                + (rx - alpha - beta - gamma) * tau * tau * tau
                - delr * dtau
        } else {
            alpha * tau + (rx - alpha) * tau * tau * tau - delr * dtau
        };
        let mut r = rz * pw(10.0, xx);
        if mass < self.zpars[1] - 0.3 {
            // Degenerate low-mass floor (HPT00 eq. 24, after Tout et al. 1997), X = 0.76 - 3Z.
            let rdgen = 0.0258 * pw(1.0 + self.zpars[11], 5.0 / 3.0) * pw(mass, -1.0 / 3.0);
            r = r.max(rdgen);
        }
        Some((lum, r))
    }
}
