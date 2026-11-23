//! Stage 3, the disc and its material bands (design section 3, row 3).
//!
//! A viscous disc on the Lynden-Bell and Pringle similarity solution
//! (Hartmann et al. 1998, gamma = 1): it spreads, feeds the star and thins,
//! until photoevaporation clears it. Midplane temperature combines starlight
//! on a flared disc (Chiang and Goldreich 1997) and accretion heating under an optically thick column
//! (Nakamoto and Nakagawa 1994). Condensation lines (rock, soot, water, CO2,
//! CO) are where that temperature crosses each condensation temperature, so
//! they sweep inward as the disc cools. Between them lie the material bands.
//!
//! Units: AU, solar masses, Myr, kelvin. Budgets are integers of `MASS_UNIT`
//! solar masses so they close exactly (H6 no double counting).

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

pub const REF_HARTMANN: &str = "Hartmann, L. et al. 1998, ApJ 495, 385: similarity-solution discs, accretion about 1e-8 solar masses a year at 1 Myr";
pub const REF_CG97: &str = "Chiang, E. I. and Goldreich, P. 1997, ApJ 490, 368, eq. 12a: passive flared disc interior, T = 150 K (r / 1 AU)^-3/7 (L / L_sun)^2/7 (M / M_sun)^-1/7";
pub const REF_NAKAMOTO: &str = "Nakamoto, T. and Nakagawa, Y. 1994, ApJ 421, 640: midplane temperature of an accreting, irradiated disc";
pub const REF_OBERG: &str = "Oberg, K. I., Murray-Clay, R. and Bergin, E. A. 2011, ApJL 743, L16: H2O 135 K, CO2 47 K, CO 20 K";
pub const REF_LODDERS: &str = "Lodders, K. 2003, ApJ 591, 1220: condensation temperatures (forsterite and iron about 1350 to 1400 K) and protosolar abundances";
pub const REF_LI_SOOT: &str =
    "Li, J. et al. 2021, Science Advances 7, eabd3632: the soot line near 500 K";
pub const REF_ANDREWS: &str = "Andrews, S. M. et al. 2010, ApJ 723, 1241 (characteristic radii 14 to 200 AU); Andrews et al. 2013, ApJ 771, 129 (disc mass scales with stellar mass)";
pub const REF_LIFETIME: &str = "Haisch, Lada and Lada 2001, ApJL 553, L153; Mamajek 2009, AIP Conf. Proc. 1158, 3: half of discs gone by about 3 Myr, e-folding about 2.5 Myr";
pub const REF_OWEN: &str = "Owen, J. E., Clarke, C. J. and Ercolano, B. 2012, MNRAS 422, 1880: X-ray photoevaporation, about 1e-9 to 1e-8 solar masses a year";
pub const REF_AL94: &str = "Artymowicz, P. and Lubow, S. H. 1994, ApJ 421, 651: circumstellar discs truncated near a third of the binary separation, circumbinary inner edge near 2 to 3 separations";

/// One budget unit: 1e-12 solar masses (about 2e18 kg; a u64 holds 1.8e7 solar masses).
pub const MASS_UNIT: f64 = 1.0e-12;

const G: f64 = sse_core::sheet::G;
const SIGMA_SB: f64 = sse_core::sheet::SIGMA_SB;
const K_B: f64 = sse_core::sheet::K_B;
const M_H: f64 = sse_core::sheet::atomic_weight::H * 1e-3 / sse_core::sheet::N_A;
const AU_M: f64 = sse_core::sheet::AU;
const MSUN_KG: f64 = sse_core::sheet::M_SUN;
const YR_S: f64 = sse_core::sheet::YR_JULIAN;
const MYR_S: f64 = 1e6 * sse_core::sheet::YR_JULIAN;
/// Mean molecular weight of disc gas.
const MU: f64 = 2.34;

/// Condensation lines, hottest first: (key, name, temperature K, the group that freezes out there).
pub const LINES: [(&str, &str, f64, Group); 5] = [
    ("rock", "rock line", 1400.0, Group::Rock),
    ("soot", "soot line", 500.0, Group::Carbon),
    ("water", "water ice line", 135.0, Group::Water),
    ("co2", "CO2 ice line", 47.0, Group::Co2),
    ("co", "CO ice line", 20.0, Group::Co),
];

/// Element groups the budgets track (design principle 4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Group {
    /// Iron and nickel.
    Metal,
    /// Silicates and oxides (Mg, Si, O and the rest).
    Rock,
    /// Refractory carbon and organics.
    Carbon,
    Water,
    Co2,
    Co,
}

pub const GROUPS: [Group; 6] = [
    Group::Metal,
    Group::Rock,
    Group::Carbon,
    Group::Water,
    Group::Co2,
    Group::Co,
];

impl Group {
    /// Condensation temperature (metal condenses with rock).
    pub fn condenses_below(self) -> f64 {
        match self {
            Group::Metal | Group::Rock => 1400.0,
            Group::Carbon => 500.0,
            Group::Water => 135.0,
            Group::Co2 => 47.0,
            Group::Co => 20.0,
        }
    }
}

/// Mass fraction of each group in the disc gas for the Sun's composition
/// (protosolar Z = 0.0142 as in Asplund et al. 2009; Lodders 2003 gives Z0 = 0.0149; split by carrier as in Oberg et al.
/// 2011): metal 0.0018, rock 0.0035, carbon 0.0010, water 0.0045, CO2 0.0008,
/// CO 0.0022. The remainder of Z (nitrogen, noble gases, sulphur in gas) is not
/// tracked as solids.
pub const SOLAR_MIX: [f64; 6] = [0.0018, 0.0035, 0.0010, 0.0045, 0.0008, 0.0022];

/// The system's mix: metals scale with [Fe/H]; oxygen-carriers (rock, water)
/// with [alpha/Fe] too; carbon carriers with C/O against the solar 0.55, and
/// water loses the oxygen that extra carbon locks into CO (Oberg 2011 figure 2,
/// first order).
pub fn mix(feh: f64, alpha_fe: f64, c_o: f64) -> [f64; 6] {
    let z = m::pow(10.0, feh);
    let a = m::pow(10.0, alpha_fe);
    let c = c_o / 0.55;
    let water_left = (1.0 - 0.6 * (c_o - 0.55)).clamp(0.0, 1.5);
    let s = SOLAR_MIX;
    [
        s[0] * z,
        s[1] * z * a,
        s[2] * z * c,
        s[3] * z * a * water_left,
        s[4] * z * c,
        s[5] * z * c,
    ]
}

/// Disc parameters drawn or fixed at birth.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DiscParams {
    /// Solar masses at t = 0.
    pub mass0: f64,
    /// Characteristic radius at t = 0, AU.
    pub rc0: f64,
    /// Shakura-Sunyaev viscosity.
    pub alpha: f64,
    /// Mass of the star(s) it orbits, and their luminosity (solar units).
    pub host_mass: f64,
    pub host_luminosity: f64,
    /// Rosseland mean opacity for the accretion term, cm^2/g, at solar dust.
    pub kappa: f64,
    /// Mass fractions by group (`GROUPS` order).
    pub mix: [f64; 6],
    /// Photoevaporation that ends the disc: internal (X-ray) plus external (FUV).
    pub mdot_pe: f64,
    /// Inner edge (circumbinary discs), AU; 0 for a disc round one star.
    pub inner_edge: f64,
    /// Truncation by a companion, AU (None if untruncated).
    pub truncation: Option<f64>,
    pub circumbinary: bool,
}

/// What changes with time, at one moment.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DiscSlice {
    pub t_myr: f64,
    /// Gas mass left, solar masses.
    pub mass: f64,
    /// Accretion onto the star, solar masses a year.
    pub mdot: f64,
    pub rc: f64,
    /// Condensation-line radii, AU, `LINES` order.
    pub lines: [f64; 5],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Band {
    pub key: String,
    /// Edges at t = 0, AU (inner edge of the disc, then the condensation lines).
    pub inner_au: f64,
    pub outer_au: f64,
    /// Budget units of gas, and of each group as solid and as vapour.
    pub gas: u64,
    pub solid: [u64; 6],
    pub vapour: [u64; 6],
    /// Midplane temperature at the band's middle at t = 0.
    pub temperature: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub gas_initial: u64,
    pub gas_accreted: u64,
    pub gas_dispersed: u64,
    pub groups_initial: [u64; 6],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Disc {
    pub params: DiscParams,
    /// Viscous time at rc0, Myr.
    pub t_visc: f64,
    /// When photoevaporation clears the gas, Myr.
    pub t_disperse: f64,
    pub bands: Vec<Band>,
    pub ledger: Ledger,
}

/// Passive irradiated temperature of a flared disc's interior (CG97).
pub fn irradiation_temperature(p: &DiscParams, r_au: f64) -> f64 {
    150.0
        * m::pow(r_au, -3.0 / 7.0)
        * m::pow(p.host_luminosity, 2.0 / 7.0)
        * m::pow(p.host_mass, -1.0 / 7.0)
}

/// Midplane temperature from starlight and accretion heating.
pub fn midplane_temperature(p: &DiscParams, sigma_gcm2: f64, mdot_msun_yr: f64, r_au: f64) -> f64 {
    let t_irr = irradiation_temperature(p, r_au);
    let r = r_au * AU_M;
    let mdot = mdot_msun_yr * MSUN_KG / YR_S;
    let teff4 = 3.0 * G * p.host_mass * MSUN_KG * mdot / (8.0 * m::PI * SIGMA_SB * r * r * r);
    let dust = p.mix[0] + p.mix[1];
    let kappa = p.kappa * dust / (SOLAR_MIX[0] + SOLAR_MIX[1]);
    let tau = 0.5 * kappa * sigma_gcm2;
    let tacc4 = 0.75 * tau * teff4;
    m::pow(t_irr * t_irr * t_irr * t_irr + tacc4, 0.25)
}

impl Disc {
    /// Viscous time at rc0 (Myr): rc0^2 / (3 nu), nu = alpha c_s^2 / Omega with
    /// the irradiation temperature there.
    pub fn viscous_time(p: &DiscParams) -> f64 {
        let t = irradiation_temperature(p, p.rc0);
        let cs2 = K_B * t / (MU * M_H);
        let r = p.rc0 * AU_M;
        let omega = m::sqrt(G * p.host_mass * MSUN_KG / (r * r * r));
        let nu = p.alpha * cs2 / omega;
        r * r / (3.0 * nu) / MYR_S
    }

    pub fn new(p: DiscParams) -> Self {
        let t_visc = Self::viscous_time(&p);
        let mdot0 = p.mass0 / (2.0 * t_visc * 1.0e6);
        // Clears when accretion falls to the photoevaporation rate: Mdot0 T^-3/2 = Mdot_pe.
        let tt = m::pow((mdot0 / p.mdot_pe).max(1.0), 2.0 / 3.0);
        let t_disperse = ((tt - 1.0) * t_visc).max(0.1);
        let mut d = Disc {
            params: p,
            t_visc,
            t_disperse,
            bands: vec![],
            ledger: Ledger {
                gas_initial: 0,
                gas_accreted: 0,
                gas_dispersed: 0,
                groups_initial: [0; 6],
            },
        };
        d.claim_bands();
        d
    }

    fn tt(&self, t: f64) -> f64 {
        1.0 + t / self.t_visc
    }
    pub fn mass_at(&self, t: f64) -> f64 {
        if t >= self.t_disperse {
            0.0
        } else {
            self.params.mass0 / m::sqrt(self.tt(t))
        }
    }
    pub fn rc_at(&self, t: f64) -> f64 {
        self.params.rc0 * self.tt(t.min(self.t_disperse))
    }
    pub fn mdot_at(&self, t: f64) -> f64 {
        if t >= self.t_disperse {
            return 0.0;
        }
        let tt = self.tt(t);
        self.params.mass0 / (2.0 * self.t_visc * 1.0e6) / (tt * m::sqrt(tt))
    }
    /// Gas surface density, g/cm^2 (zero inside a circumbinary cavity).
    pub fn sigma_at(&self, r: f64, t: f64) -> f64 {
        if r < self.params.inner_edge {
            return 0.0;
        }
        let (mm, rc) = (self.mass_at(t), self.rc_at(t));
        let s = mm / (2.0 * m::PI * rc * r) * m::exp(-r / rc);
        s * MSUN_KG * 1000.0 / (AU_M * AU_M * 1.0e4)
    }
    pub fn temperature_at(&self, r: f64, t: f64) -> f64 {
        midplane_temperature(&self.params, self.sigma_at(r, t), self.mdot_at(t), r)
    }
    /// Radius where the midplane temperature falls to `temp`, AU: bisection in
    /// log radius over 0.005 to 10,000 AU, fixed 80 steps.
    pub fn line_at(&self, temp: f64, t: f64) -> f64 {
        let (mut lo, mut hi) = (m::ln(0.005), m::ln(10_000.0));
        let f = |x: f64| self.temperature_at(m::exp(x), t) - temp;
        if f(lo) < 0.0 {
            return m::exp(lo);
        }
        if f(hi) > 0.0 {
            return m::exp(hi);
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if f(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        m::exp(0.5 * (lo + hi))
    }

    pub fn slice(&self, t: f64) -> DiscSlice {
        let tq = t.min(self.t_disperse * (1.0 - 1e-12));
        DiscSlice {
            t_myr: t,
            mass: self.mass_at(t),
            mdot: self.mdot_at(t),
            rc: self.rc_at(t),
            lines: LINES.map(|(_, _, temp, _)| self.line_at(temp, tq)),
        }
    }

    /// Fraction of the gas inside `r` at t = 0: 1 - exp(-r / rc0) for gamma = 1,
    /// renormalised for a circumbinary cavity.
    fn cum(&self, r: f64) -> f64 {
        let rc = self.params.rc0;
        let a = m::exp(-self.params.inner_edge / rc);
        if r <= self.params.inner_edge {
            return 0.0;
        }
        (a - m::exp(-r / rc)) / a
    }

    /// Claim the budgets into bands at t = 0, exactly: each band takes the floor
    /// of its share, the outermost band takes what is left.
    fn claim_bands(&mut self) {
        let p = &self.params;
        // The disc's own gas mass is its budget (it never held more than mass0).
        let gas_total = m::round(p.mass0 / MASS_UNIT) as u64;
        let groups: [u64; 6] =
            std::array::from_fn(|g| m::round(p.mass0 * p.mix[g] / MASS_UNIT) as u64);
        let lines0: Vec<f64> = LINES
            .iter()
            .map(|(_, _, temp, _)| self.line_at(*temp, 0.0))
            .collect();
        let mut edges = vec![p.inner_edge];
        edges.extend(lines0.iter().map(|x| x.max(p.inner_edge)));
        let keys = ["vapour", "rock", "carbon", "water-ice", "co2-ice", "co-ice"];
        let n = keys.len();
        let mut bands = vec![];
        let (mut gas_left, mut g_left) = (gas_total, groups);
        for (k, key) in keys.iter().enumerate() {
            let inner = edges[k];
            let outer = if k + 1 < n {
                edges[k + 1]
            } else {
                f64::INFINITY
            };
            let share = if outer.is_finite() {
                self.cum(outer) - self.cum(inner)
            } else {
                1.0
            };
            let take = |total: u64, left: u64| -> u64 {
                if k + 1 == n {
                    left
                } else {
                    (m::floor(total as f64 * share.max(0.0)) as u64).min(left)
                }
            };
            let gas = take(gas_total, gas_left);
            gas_left -= gas;
            let mid = if outer.is_finite() {
                m::sqrt(inner.max(0.05 * outer) * outer)
            } else {
                2.0 * inner
            };
            let temperature = self.temperature_at(mid, 0.0);
            let (mut solid, mut vapour) = ([0u64; 6], [0u64; 6]);
            for (g, grp) in GROUPS.iter().enumerate() {
                let x = take(groups[g], g_left[g]);
                g_left[g] -= x;
                // Condensed in this band if the band lies beyond the group's line.
                if temperature < grp.condenses_below() {
                    solid[g] = x;
                } else {
                    vapour[g] = x;
                }
            }
            bands.push(Band {
                key: key.to_string(),
                inner_au: inner,
                outer_au: if outer.is_finite() { outer } else { f64::MAX },
                gas,
                solid,
                vapour,
                temperature,
            });
        }
        // Gas: what the star swallowed before the disc cleared, the rest blown away.
        let accreted =
            m::round((p.mass0 - self.mass_at(self.t_disperse * (1.0 - 1e-12))) / MASS_UNIT) as u64;
        let accreted = accreted.min(gas_total);
        self.ledger = Ledger {
            gas_initial: gas_total,
            gas_accreted: accreted,
            gas_dispersed: gas_total - accreted,
            groups_initial: groups,
        };
        self.bands = bands;
    }
}

/// Draws and rules that set a system's disc (stage 3 inputs).
#[allow(clippy::too_many_arguments)]
pub fn disc_params(
    seed: &Seed,
    system: &Address,
    host_mass: f64,
    host_luminosity: f64,
    feh: f64,
    alpha_fe: f64,
    c_o: f64,
    fuv_g0: f64,
    companion: Option<(f64, f64)>,
) -> DiscParams {
    let d = Draws::new(
        *seed,
        system.clone(),
        Stream::new("formation.disc.v1").expect("valid"),
    );
    let z = |i: u64| sse_resolve::dist::std_normal_quantile(d.unit(i).clamp(1e-12, 1.0 - 1e-12));
    // Young (class 0/I) discs: about 3 percent of the star, 0.4 dex scatter,
    // capped near gravitational instability (0.15).
    let frac = (0.03 * m::pow(10.0, 0.4 * z(0))).min(0.15);
    let mut rc0 = 30.0 * m::pow(10.0, 0.3 * z(1));
    let mut mass_scale = host_mass;
    let (mut inner_edge, mut truncation, mut circumbinary) = (0.0, None, false);
    if let Some((a, e)) = companion {
        let peri = a * (1.0 - e);
        if a < 2.0 {
            // Close pair: one disc round both stars, cavity of about 2.5 separations.
            circumbinary = true;
            inner_edge = 2.5 * a;
        } else {
            let tr = peri / 3.0;
            truncation = Some(tr);
            rc0 = rc0.min(tr / 3.0);
            mass_scale = host_mass;
        }
    }
    // Photoevaporation: X-ray about 1e-9 for a solar-mass star (Owen 2012), plus
    // external FUV, about 1e-9 at 1000 G0 for a typical disc (sketch, after
    // Haworth et al. 2018's FRIED grid), growing with disc size.
    let mdot_int = 1.0e-9 * host_mass;
    let mdot_ext = 1.0e-9 * (fuv_g0 / 1000.0) * (rc0 / 30.0);
    DiscParams {
        mass0: frac * mass_scale,
        rc0,
        alpha: 0.01,
        host_mass,
        host_luminosity,
        kappa: 3.0,
        mix: mix(feh, alpha_fe, c_o),
        mdot_pe: mdot_int + mdot_ext,
        inner_edge,
        truncation,
        circumbinary,
    }
}
