//! F-redox: how oxidised a young world's mantle is, and the gas its magma
//! ocean breathes out. The blocks are Equations in `packs/base-physics/
//! formulas/interior.json` and `atmosphere.json`; these are the same sums
//! for formation's stage 8 (a test holds the two together):
//!
//! - `sse.interior.redox_iw`: the mantle's oxygen fugacity against the
//!   iron-wüstite buffer, from how much of the world's iron stayed in the
//!   rock as FeO rather than sinking as metal (water oxidises metal before
//!   that, carbon smelts FeO back to metal), then the self-oxidation of a
//!   deep magma ocean;
//! - `sse.atmosphere.outgassed_mix`: the hydrogen and carbon a magma ocean
//!   gives off, split into H2, H2O, CO, CO2 and CH4 by gas equilibrium at
//!   that oxygen fugacity.
//!
//! No classes: a big core, carbon or no water makes a reduced world that
//! breathes H2 and CO; a small core and water make an oxidised one that
//! breathes H2O and CO2, by degrees.

use sse_core::math as m;

pub const WW05: &str = "Wade, J. and Wood, B. J. 2005, EPSL 236, 78: core formation and the oxidation state of the Earth; ΔIW = 2 log10(x_FeO / x_Fe) between silicate and metal. doi:10.1016/j.epsl.2005.05.017";
pub const MD03: &str = "McDonough, W. F. 2003, Treatise on Geochemistry 2, 547: bulk Earth 32 wt% iron, the core 85 wt% iron (tables 3 and 4). doi:10.1016/B0-08-043751-6/02015-6";
pub const MS95: &str = "McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: pyrolite, the silicate Earth's oxides. doi:10.1016/0009-2541(94)00140-4";
pub const ARM19: &str = "Armstrong, K., Frost, D. J., McCammon, C. A., Rubie, D. C. and Ballaran, T. B. 2019, Science 365, 903: a deep magma ocean oxidises itself as FeO turns to Fe2O3 and metal under pressure. doi:10.1126/science.aax8376";
pub const SOS20: &str = "Sossi, P. A., Burnham, A. D., Badro, J., Lanzirotti, A., Newville, M. and O'Neill, H. St. C. 2020, Science Advances 6, eabd1387: Earth's magma ocean surface near IW+0.5. doi:10.1126/sciadv.abd1387";
pub const HIR12: &str = "Hirschmann, M. M. 2012, EPSL 341, 48: magma ocean influence on early atmosphere mass and composition. doi:10.1016/j.epsl.2012.06.015";
pub const GAI22: &str = "Gaillard, F., Bernadou, F., Roskosz, M. et al. 2022, EPSL 577, 117255: redox controls during magma ocean degassing. doi:10.1016/j.epsl.2021.117255";
pub const FRO91: &str = "Frost, B. R. 1991, Reviews in Mineralogy 25, 1: introduction to oxygen fugacity and its petrologic importance; iron-wüstite log fO2 = -27489/T + 6.702 (table 1). doi:10.1515/9781501508684-004";
pub const JANAF: &str = "Chase, M. W. 1998, NIST-JANAF Thermochemical Tables, 4th edn, J. Phys. Chem. Ref. Data Monograph 9: Gibbs energies of formation of H2O, CO, CO2 and CH4, fitted as A/T - B from 1000 to 2000 K";

/// The redox block's parameters (its defaults).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Redox {
    /// Iron's share of the rock and metal by mass (bulk Earth 0.32).
    pub iron: f64,
    /// Iron's share of the core by mass (Earth 0.85).
    pub core_iron: f64,
    /// Iron's mole fraction in the metal (Earth's core about 0.79).
    pub metal_iron: f64,
    /// Activity coefficient ratio γ_FeO / γ_Fe (1: ideal mixing).
    pub gamma: f64,
    /// Mean molar mass of the silicate's other oxides, kg/mol (pyrolite).
    pub silicate_molar: f64,
    /// The least FeO mole fraction a mantle keeps.
    pub floor: f64,
    /// Self-oxidation of a deep magma ocean, log10 units.
    pub shift: f64,
}

pub const REDOX: Redox = Redox {
    iron: 0.32,
    core_iron: 0.85,
    metal_iron: 0.79,
    gamma: 1.0,
    silicate_molar: 0.0506,
    floor: 1.0e-3,
    shift: 2.7,
};

/// The mantle's ΔIW (log10 units against iron-wüstite) at the magma ocean's
/// surface, and at core formation: from the core mass fraction `cmf` of the
/// rock and metal, `carbon` free to smelt FeO (share of the rock and metal
/// by mass), and the molar masses of Fe, FeO and C (kg/mol).
pub fn redox_iw(cmf: f64, carbon: f64, m_fe: f64, m_feo: f64, m_c: f64, p: &Redox) -> (f64, f64) {
    let fem = (p.iron - p.core_iron * cmf).max(0.0);
    let n = (fem / m_fe - carbon.max(0.0) / m_c).max(0.0);
    let ns = (1.0 - cmf - n * m_feo).max(0.0) / p.silicate_molar;
    let x = if n + ns <= 0.0 { 1.0 } else { n / (n + ns) };
    let core = 2.0 * m::log10((p.gamma * x).max(p.floor) / p.metal_iron);
    ((core + p.shift).clamp(-8.0, 6.0), core)
}

/// The outgassing block's parameters: equilibrium temperature, and the
/// fitted log10 constants (A/T - B) of iron-wüstite and three reactions.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Outgas {
    pub temperature: f64,
    pub iw_a: f64,
    pub iw_b: f64,
    /// H2 + ½ O2 = H2O.
    pub water_a: f64,
    pub water_b: f64,
    /// CO + ½ O2 = CO2.
    pub co2_a: f64,
    pub co2_b: f64,
    /// CO + 3 H2 = CH4 + H2O.
    pub ch4_a: f64,
    pub ch4_b: f64,
    /// The tables' standard pressure, Pa (1 bar).
    pub standard_pressure: f64,
}

pub const OUTGAS: Outgas = Outgas {
    temperature: 2000.0,
    iw_a: 27489.0,
    iw_b: 6.702,
    water_a: 13040.0,
    water_b: 2.981,
    co2_a: 14674.0,
    co2_b: 4.456,
    ch4_a: 11860.0,
    ch4_b: 13.279,
    standard_pressure: sse_core::sheet::unit::BAR,
};

/// Mole fractions [H2, H2O, CO, CO2, CH4] of what a magma ocean at `diw`
/// gives off from `h` moles of hydrogen (as H2) and `c` moles of carbon,
/// under a total pressure `p_pa`. All zero when it gives off nothing.
pub fn outgassed_mix(diw: f64, h: f64, c: f64, p_pa: f64, o: &Outgas) -> [f64; 5] {
    let t = o.temperature;
    let lo = diw + o.iw_b - o.iw_a / t;
    let r1 = m::pow(10.0, o.water_a / t - o.water_b + lo / 2.0);
    let r2 = m::pow(10.0, o.co2_a / t - o.co2_b + lo / 2.0);
    let hh = h.max(0.0);
    let cc = c.max(0.0);
    let h2 = hh / (1.0 + r1);
    let w = hh - h2;
    let co = cc / (1.0 + r2);
    let co2 = cc - co;
    let k3 = m::pow(10.0, o.ch4_a / t - o.ch4_b);
    let nt = h2 + w + co + co2;
    let pb = p_pa.max(0.0) / o.standard_pressure;
    let q = if nt <= 0.0 || w <= 0.0 {
        0.0
    } else {
        k3 * co * (h2 * h2 * h2) * (pb * pb) / (w * (nt * nt))
    };
    let ch4 = q.min(co.min(h2 / 3.0));
    let n = nt - 2.0 * ch4;
    let inv = if n <= 0.0 { 0.0 } else { 1.0 / n };
    [
        (h2 - 3.0 * ch4) * inv,
        (w + ch4) * inv,
        (co - ch4) * inv,
        co2 * inv,
        ch4 * inv,
    ]
}
