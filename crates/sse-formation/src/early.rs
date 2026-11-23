//! Stage 8, each world's early life, and the hand-off to the surface, climate
//! and life suites (design section 3, row 8; F6).
//!
//! Input is what the growth stages leave (`Protoplanet`): mass, orbit, when it
//! finished growing, and its provenance by element group. Output is a
//! `WorldStart`: core and mantle, the fate of any primordial hydrogen, the
//! magma ocean and its end, the volatile inventory, and a list of hand-off
//! facts, each kind published by this stage only.

use crate::disc::{Group, GROUPS};
use crate::hurley::Fits;
use crate::lifeline::LifeLine;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Address;

pub const M_EARTH_KG: f64 = sse_core::sheet::M_EARTH;
pub const R_EARTH_M: f64 = sse_core::sheet::R_EARTH_MEAN;
const MSUN_IN_EARTH: f64 = sse_core::sheet::GM_SUN / sse_core::sheet::GM_EARTH;
const G: f64 = sse_core::sheet::G;
const AU_M: f64 = sse_core::sheet::AU;
const L_SUN_W: f64 = sse_core::sheet::L_SUN;
const MYR_S: f64 = 1e6 * sse_core::sheet::YR_JULIAN;
/// Earth's ocean mass, kg.
pub const EARTH_OCEAN_KG: f64 = sse_core::sheet::M_OCEAN_EARTH;

pub const REF_ZENG16: &str = "Zeng, L., Sasselov, D. D. and Jacobsen, S. B. 2016, ApJ 819, 127: rocky radius (1.07 - 0.21 CMF) M^(1/3.7)";
pub const REF_LF14: &str =
    "Lopez, E. D. and Fortney, J. J. 2014, ApJ 792, 1, eq. 3: radius of a hydrogen envelope";
pub const REF_OW17: &str = "Owen, J. E. and Wu, Y. 2017, ApJ 847, 29: energy-limited photoevaporation carves the radius valley";
pub const REF_JACKSON: &str = "Jackson, A. P., Davis, T. A. and Wheatley, P. J. 2012, MNRAS 422, 2024: X-ray to bolometric saturated at 10^-3.1 (late K) to 10^-4.3 (early F) for 10^7.6 to 10^8.3 yr; SSE4 uses 10^-3.5 and 100 Myr for every star, as XUV, falling as t^-1.5 after. arXiv:1111.0031";
pub const REF_HAMANO: &str = "Hamano, K., Abe, Y. and Genda, H. 2013, Nature 497, 607: magma oceans solidify in a few Myr beyond the runaway limit, about 100 Myr inside it, losing their water";
pub const REF_KOPPARAPU: &str = "Kopparapu, R. K. et al. 2014, ApJL 787, L29 (arXiv:1404.5292), table 1: runaway greenhouse for a Sun-like star at 0.99, 1.107 and 1.188 times Earth's insolation for 0.1, 1 and 5 Earth masses (corrects the 2013 value of about 1.06, ApJ 765, 131)";
pub const REF_ELKINS: &str = "Elkins-Tanton, L. T. 2008, EPSL 271, 181: magma ocean outgassing; most water and CO2 go to the early atmosphere, a fraction stays in the mantle";

/// What growth leaves behind (F2 to F4 fill this; F6 reads it).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Protoplanet {
    pub address: Address,
    /// Earth masses of solids.
    pub mass: f64,
    pub a_au: f64,
    pub e: f64,
    /// When it stopped growing, Myr from the system's birth.
    pub formed_myr: f64,
    /// Solid mass by group (`GROUPS` order), Earth masses: the provenance ledger's sums.
    pub by_group: [f64; 6],
    /// Gas captured from the disc, Earth masses.
    pub gas: f64,
}

/// Fate of the primordial hydrogen air.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Envelope {
    None,
    Stripped,
    Kept,
    /// Enough gas that it is a giant: the envelope is the planet.
    Giant,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldStart {
    pub address: Address,
    pub mass_earth: f64,
    pub core_mass_fraction: f64,
    /// Water mass fraction of the solid body.
    pub water_fraction: f64,
    /// Radius of the solid body, Earth radii.
    pub solid_radius: f64,
    /// Radius including any kept hydrogen envelope at 5 Gyr, Earth radii.
    pub radius: f64,
    pub envelope: Envelope,
    /// Hydrogen envelope as a fraction of total mass after loss.
    pub envelope_fraction: f64,
    /// Magma ocean solidified, Myr from birth; whether it lost its water doing so.
    pub magma_ocean_end_myr: f64,
    pub desiccated: bool,
    /// Surface water after outgassing, in Earth oceans.
    pub surface_water_oceans: f64,
    /// Outgassed CO2, bar on this world.
    pub co2_bar: f64,
    /// Insolation when it formed, Earth units.
    pub early_insolation: f64,
    pub facts: Vec<HandOff>,
}

/// One hand-off fact: a quantity kind, an SI value, and the suite that reads it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HandOff {
    pub kind: String,
    pub value: f64,
    pub unit: String,
    pub read_by: String,
}

/// The kinds this stage publishes, each with its unit and reader. Nothing else
/// publishes them; the base-physics flows take some as inputs and derive the rest.
pub const HAND_OFF_KINDS: [(&str, &str, &str); 12] = [
    (
        "body.mass",
        "kg",
        "base-physics flows, surface (S), life (L)",
    ),
    ("body.radius", "m", "base-physics flows, surface (S)"),
    ("orbit.semi_major_axis", "m", "base-physics flows, climate"),
    ("orbit.eccentricity", "one", "base-physics flows, climate"),
    (
        "interior.core_mass_fraction",
        "one",
        "surface (S) interior model",
    ),
    (
        "interior.water_mass_fraction",
        "one",
        "surface (S), life (L)",
    ),
    (
        "atmosphere.h2_envelope_fraction",
        "one",
        "climate, surface (S)",
    ),
    (
        "early.magma_ocean_end",
        "s",
        "surface (S) first crust, life (L) earliest habitable date",
    ),
    (
        "early.surface_water",
        "kg",
        "climate, surface (S) oceans, life (L)",
    ),
    (
        "early.co2_partial_pressure",
        "pa",
        "climate, surface (S) carbonate cycle",
    ),
    ("early.desiccated", "one", "climate, life (L)"),
    ("early.insolation", "w_m2", "climate"),
];

/// Bolometric luminosity of the host in solar units at age `t` Myr.
fn host_l(fits: &Fits, host: &LifeLine, t: f64) -> f64 {
    host.state_at(fits, t).luminosity.max(1.0e-6)
}

/// Rocky radius (Zeng 2016), the `sse.interior.rocky_radius` atom's relation,
/// used outside its 1 to 8 Earth-mass validity range as a labelled extrapolation.
pub fn rocky_radius(mass_earth: f64, cmf: f64) -> f64 {
    (1.07 - 0.21 * cmf) * m::pow(mass_earth, 1.0 / 3.7)
}

/// Lopez and Fortney 2014 envelope thickness, Earth radii (solar metallicity,
/// enhanced opacity), at `age_gyr` and insolation `flux` (Earth units).
pub fn envelope_thickness(core_earth: f64, fenv: f64, flux: f64, age_gyr: f64) -> f64 {
    2.06 * m::pow(core_earth, -0.21)
        * m::pow(fenv / 0.05, 0.59)
        * m::pow(flux, 0.044)
        * m::pow(age_gyr / 5.0, -0.18)
}

/// XUV energy (J/m^2) received at `a_au` over the first `t_myr` Myr: saturated at
/// 10^-3.5 of the bolometric for 100 Myr, then falling as t^-1.5 (Jackson 2012).
fn xuv_fluence(fits: &Fits, host: &LifeLine, a_au: f64, t0: f64, t_myr: f64) -> f64 {
    let a = a_au * AU_M;
    let mut e = 0.0;
    // Fixed 200-step midpoint sum in log time from t0 to t_myr.
    let (lo, hi) = (m::ln(t0.max(1.0)), m::ln(t_myr));
    let n = 200;
    for k in 0..n {
        let x0 = lo + (hi - lo) * k as f64 / n as f64;
        let x1 = lo + (hi - lo) * (k + 1) as f64 / n as f64;
        let t = m::exp(0.5 * (x0 + x1));
        let dt = (m::exp(x1) - m::exp(x0)) * MYR_S;
        let sat = m::pow(10.0, -3.5);
        let frac = if t < 100.0 {
            sat
        } else {
            sat * m::pow(t / 100.0, -1.5)
        };
        e += frac * host_l(fits, host, t) * L_SUN_W / (4.0 * m::PI * a * a) * dt;
    }
    e
}

/// Gas above this fraction of the total makes a giant (the envelope is the planet).
pub const GIANT_GAS_FRACTION: f64 = 0.5;
/// Photoevaporation efficiency (Owen and Wu 2017 use about 0.1).
pub const EFFICIENCY: f64 = 0.1;
/// Fraction of water and CO2 that stays in a solidifying mantle (Elkins-Tanton 2008: a few to tens of percent).
pub const MANTLE_RETAINED: f64 = 0.3;
/// Fraction of a rocky world's carbon taken into the core (Dasgupta 2013, Rev.
/// Mineral. Geochem. 75, 183: most of Earth's carbon may be in the core; sketch).
pub const CARBON_TO_CORE: f64 = 0.9;

/// Stage 8 for one world around `host` (its life line and fits).
pub fn early_life(p: &Protoplanet, fits: &Fits, host: &LifeLine) -> WorldStart {
    let g = |grp: Group| p.by_group[GROUPS.iter().position(|x| *x == grp).expect("group")];
    let solids: f64 = p.by_group.iter().sum();
    let rockmetal = g(Group::Metal) + g(Group::Rock);
    let water = g(Group::Water);
    // Core: the metal, less what water oxidises (iron into the mantle as FeO):
    // first order, a water-rich body loses up to half its core (sketch).
    let ox = (water / (water + rockmetal)).min(0.5);
    let cmf = if rockmetal > 0.0 {
        g(Group::Metal) / rockmetal * (1.0 - ox)
    } else {
        0.0
    };
    let wf = if solids > 0.0 { water / solids } else { 0.0 };
    // Solid radius: rock and metal by Zeng, a water mantle adds about half its mass fraction (Zeng et al. 2019: 50 percent water, about 25 percent larger).
    let solid_radius = rocky_radius(solids.max(1e-6), cmf) * (1.0 + 0.5 * wf);

    let l_form = host_l(fits, host, p.formed_myr.max(1.0));
    let flux = l_form / (p.a_au * p.a_au);
    let total = solids + p.gas;
    let (envelope, fenv) = if p.gas <= 0.0 {
        (Envelope::None, 0.0)
    } else if p.gas / total > GIANT_GAS_FRACTION {
        (Envelope::Giant, p.gas / total)
    } else {
        // Energy-limited loss over 1 Gyr: the envelope goes if the absorbed XUV
        // energy can lift it out of the potential well.
        let fenv = p.gas / total;
        let r = (solid_radius + envelope_thickness(solids, fenv, flux, 0.1)) * R_EARTH_M;
        let fluence = xuv_fluence(fits, host, p.a_au, p.formed_myr, 1000.0);
        let absorbed = EFFICIENCY * fluence * m::PI * r * r;
        let binding = G * solids * M_EARTH_KG * p.gas * M_EARTH_KG / r;
        if absorbed > binding {
            (Envelope::Stripped, 0.0)
        } else {
            // Partial loss: what the energy could lift, a first-order share.
            (Envelope::Kept, fenv * (1.0 - absorbed / binding))
        }
    };
    let radius = match envelope {
        Envelope::Kept => solid_radius + envelope_thickness(solids, fenv, flux, 5.0),
        Envelope::Giant => {
            // Giants: about a Jupiter radius over a wide mass range (sketch until F3's giant block).
            11.2
        }
        _ => solid_radius,
    };
    // Magma ocean: Hamano 2013 type I (a few Myr) beyond the runaway limit, type
    // II (about 100 Myr, water lost to space) inside it. Bodies below 0.1 Earth
    // masses keep no global magma ocean beyond formation.
    // Runaway limit by planet mass, Kopparapu et al. 2014 table 1 (Sun-like star),
    // straight between its three masses in log mass, held flat outside them.
    let runaway = {
        let lm = m::log10(solids.clamp(0.1, 5.0));
        if lm < 0.0 {
            1.107 + (1.107 - 0.99) * lm
        } else {
            1.107 + (1.188 - 1.107) * lm / m::log10(5.0)
        }
    };
    let desiccated = solids >= 0.1
        && flux > runaway
        && envelope != Envelope::Kept
        && envelope != Envelope::Giant;
    let mo_years = if solids < 0.1 {
        0.0
    } else if desiccated {
        100.0
    } else {
        4.0
    };
    let magma_ocean_end = p.formed_myr + mo_years;
    let outgassed = 1.0 - MANTLE_RETAINED;
    let surface_water_kg = if desiccated {
        0.0
    } else {
        water * M_EARTH_KG * outgassed
    };
    // CO2 from the carbon carriers: most carbon follows iron into the core
    // (CARBON_TO_CORE), the rest outgasses as CO2 like the water; column mass
    // times gravity is pressure.
    let carbon = g(Group::Carbon) + g(Group::Co2) * (12.0 / 44.0) + g(Group::Co) * (12.0 / 28.0);
    let co2_kg = carbon * M_EARTH_KG * (1.0 - CARBON_TO_CORE) * outgassed * (44.0 / 12.0);
    let r_m = solid_radius * R_EARTH_M;
    let grav = G * solids * M_EARTH_KG / (r_m * r_m);
    let co2_pa = co2_kg * grav / (4.0 * m::PI * r_m * r_m);

    let h = |kind: &str, value: f64| {
        let (_, unit, read_by) = HAND_OFF_KINDS
            .iter()
            .find(|k| k.0 == kind)
            .expect("declared kind");
        HandOff {
            kind: kind.into(),
            value,
            unit: unit.to_string(),
            read_by: read_by.to_string(),
        }
    };
    let facts = vec![
        h("body.mass", total * M_EARTH_KG),
        h("body.radius", radius * R_EARTH_M),
        h("orbit.semi_major_axis", p.a_au * AU_M),
        h("orbit.eccentricity", p.e),
        h("interior.core_mass_fraction", cmf),
        h("interior.water_mass_fraction", wf),
        h("atmosphere.h2_envelope_fraction", fenv),
        h("early.magma_ocean_end", magma_ocean_end * MYR_S),
        h("early.surface_water", surface_water_kg),
        h("early.co2_partial_pressure", co2_pa),
        h("early.desiccated", if desiccated { 1.0 } else { 0.0 }),
        h("early.insolation", flux * sse_core::sheet::S_SUN),
    ];
    WorldStart {
        address: p.address.clone(),
        mass_earth: total,
        core_mass_fraction: cmf,
        water_fraction: wf,
        solid_radius,
        radius,
        envelope,
        envelope_fraction: fenv,
        magma_ocean_end_myr: magma_ocean_end,
        desiccated,
        surface_water_oceans: surface_water_kg / EARTH_OCEAN_KG,
        co2_bar: co2_pa / 1.0e5,
        early_insolation: flux,
        facts,
    }
}

/// Earth masses per solar mass (for callers converting disc budgets).
pub fn earth_masses(msun: f64) -> f64 {
    msun * MSUN_IN_EARTH
}
