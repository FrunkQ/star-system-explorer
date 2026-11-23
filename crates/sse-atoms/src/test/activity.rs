//! `activity` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const W11: &str = "Wright, N. J., Drake, J. J., Mamajek, E. E. and Henry, G. W. 2011, The stellar-activity-rotation relationship and the evolution of stellar dynamos, ApJ 743, 48: log τ = 1.16 − 1.49 log(M/M☉) − 0.54 log²(M/M☉) days (0.09 to 1.36 M☉); saturated log(L_X/L_bol) = −3.13 below Ro = 0.13; slope β = −2.70 above it (unbiased sample); Sun log(L_X/L_bol) = −6.24. https://arxiv.org/abs/1109.4634";
const NOYES84: &str = "Noyes, R. W., Hartmann, L. W., Baliunas, S. L., Duncan, D. K. and Vaughan, A. H. 1984, Rotation, convection, and magnetic activity in lower main-sequence stars, ApJ 279, 763: activity depends on the Rossby number. doi:10.1086/161945";
const MITTAG23: &str = "Mittag, M., Schmitt, J. H. M. M. and Schröder, K.-P. 2023, Revisiting the cycle-rotation connection for late-type stars: short (inactive) branch log P_cyc = 1.488 + 1.324 log P_rot and long (active) branch log P_cyc = 2.22 + 1.28 log P_rot, both periods in days; scatter 0.24 dex and about 40 %; the Sun lies between the branches (6.1 yr predicted against 11). https://arxiv.org/abs/2306.05866";
const VAS24: &str = "Vasilyev, V. et al. 2024, Sun-like stars produce superflares roughly once per century, Science: (8.63 ± 0.20)×10⁻³ per year above 10³⁴ erg for 5000 to 6500 K stars with rotation over 20 days; energy spread dN/dE ∝ E^(−1.97 ± 0.30), consistent with the Sun's own flares extrapolated. https://arxiv.org/abs/2412.12265";
const SHIB13: &str = "Shibayama, T. et al. 2013, Superflares on solar-type stars observed with Kepler I, ApJS 209, 5: dN/dE ∝ E^(−2.0) for slow rotators, −2.2 for all G dwarfs; fast rotators flare far more often. https://arxiv.org/abs/1308.1480";

/// The Sun's X-ray fraction (Judge et al. 2003, as used by Wright et al. 2011).
pub const SUN_LOG_RX: f64 = -6.24;
/// Reference flare energy, J (10³⁴ erg).
pub const E_REF_J: f64 = 1e27;
/// Flares above E_REF per second per watt of X-ray luminosity, times E_REF:
/// k = N(>E_REF) E_REF / L_X, fixed by Vasilyev et al. 2024 (8.63e-3 per
/// year above 10³⁴ erg) on a star with the Sun's L_X (10^-6.24 L☉).
pub const FLARE_K: f64 = 8.63e-3 / JULIAN_YEAR * E_REF_J / (5.754_399e-7 * L_SUN);

const M_SUN: f64 = GM_SUN / G;

/// Convective turnover time, s, from mass in kg (Wright et al. 2011 eq. 11).
/// Outside 0.09 to 1.36 M☉ the fit is extrapolated; the caller says so.
pub fn convective_turnover(mass_kg: f64) -> f64 {
    let x = m::log10(mass_kg / M_SUN);
    m::pow(10.0, 1.16 - 1.49 * x - 0.54 * x * x) * DAY
}

/// L_X / L_bol from the Rossby number, with saturation.
pub fn xray_fraction(ro: f64, log_rx_sat: f64, ro_sat: f64, beta: f64) -> f64 {
    let sat = m::pow(10.0, log_rx_sat);
    if ro <= ro_sat {
        sat
    } else {
        sat * m::pow(ro / ro_sat, beta)
    }
}

/// Activity cycle period, s, from the rotation period, s: log P_cyc = a + n log P_rot (days).
pub fn cycle_period(p_rot_s: f64, a: f64, n: f64) -> f64 {
    m::pow(10.0, a + n * m::log10(p_rot_s / DAY)) * DAY
}

/// Flares per second above energy `e` J for X-ray luminosity `lx` W:
/// N(>E) = k (L_X / E_ref) (E / E_ref)^(1 − α).
pub fn flare_rate_above(lx: f64, e: f64, k: f64, alpha: f64) -> f64 {
    k * lx / E_REF_J * m::pow(e / E_REF_J, 1.0 - alpha)
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let rate = crate::common::dims::RATE;
    let msun_range = Some((0.09 * M_SUN, 1.36 * M_SUN));
    vec![
        Def {
            id: "sse.activity.convective_turnover_time",
            version: "1.0.0",
            inputs: vec![port("mass", "star.mass", D::MASS, msun_range)],
            outputs: vec![port("tau", "star.convective_turnover_time", D::TIME, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[W11, NOYES84],
            curious: "Cool stars boil like a pan of soup under their surfaces. The smaller the star, the deeper and slower the boiling: about two weeks for the Sun, months for a small red dwarf.",
            science: "Empirical turnover time fitted so that X-ray activity against spin over turnover time is one relation for all cool stars. Valid for 0.09 to 1.36 solar masses; stars without a convective outer layer have none.",
            formula: "log τ[d] = 1.16 − 1.49 log(M/M☉) − 0.54 log²(M/M☉)",
            f: |i, _| Ok(vec![convective_turnover(positive(i[0], "mass")?)]),
        }
        .build(),
        Def {
            id: "sse.activity.rossby_number",
            version: "1.0.0",
            inputs: vec![
                port("period", "body.rotation_period", D::TIME, None),
                port("tau", "star.convective_turnover_time", D::TIME, None),
            ],
            outputs: vec![port("ro", "star.rossby_number", D::NONE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[NOYES84, W11],
            curious: "A star's magnetism comes from spin twisting its boiling layer. What matters is how many turns it makes in one boil: fewer than one, and the star is quiet like the Sun; many, and it is wild.",
            science: "Rossby number Ro = P_rot / τ_c.",
            formula: "Ro = P_rot / τ",
            f: |i, _| Ok(vec![positive(i[0], "rotation period")? / positive(i[1], "turnover time")?]),
        }
        .build(),
        Def {
            id: "sse.activity.xray_fraction",
            version: "1.0.0",
            inputs: vec![port("ro", "star.rossby_number", D::NONE, None)],
            outputs: vec![port("rx", "star.xray_fraction", D::NONE, None)],
            params: vec![
                param("log_rx_sat", D::NONE, -3.13, (-6.0, -1.0), "saturated log(L_X/L_bol) (Wright 2011: −3.13 ± 0.08)"),
                param("ro_sat", D::NONE, 0.13, (0.01, 1.0), "Rossby number below which activity saturates (0.13 ± 0.02)"),
                param("beta", D::NONE, -2.70, (-5.0, 0.0), "slope above saturation (−2.70 ± 0.13 unbiased; −2.18 from the full-sample fit)"),
            ],
            cost: CostClass::Trivial,
            references: &[W11],
            curious: "The Sun gives out less than a millionth of its power as X-rays. A young fast-spinning star gives out a thousandth, and no amount of extra spin pushes it higher: it is saturated.",
            science: "The activity-rotation relation: flat below the saturation Rossby number, a power law above. The Sun's Rossby number 1.8 gives log(L_X/L_bol) = −6.21 against −6.24 observed.",
            formula: "L_X/L_bol = R_sat for Ro ≤ Ro_sat; R_sat (Ro/Ro_sat)^β above",
            f: |i, p| Ok(vec![xray_fraction(positive(i[0], "Rossby number")?, p[0], p[1], p[2])]),
        }
        .build(),
        Def {
            id: "sse.activity.cycle_period_inactive",
            version: "1.0.0",
            inputs: vec![port("period", "body.rotation_period", D::TIME, None)],
            outputs: vec![port("cycle", "star.activity_cycle_period", D::TIME, None)],
            params: vec![
                param("a", D::NONE, 1.488, (-2.0, 6.0), "intercept, log days"),
                param("n", D::NONE, 1.324, (0.0, 4.0), "slope"),
            ],
            cost: CostClass::Trivial,
            references: &[MITTAG23],
            curious: "Quiet stars wax and wane in spots over years, like the Sun's eleven-year cycle. Slower spinners have longer cycles.",
            science: "The short-cycle (inactive) branch of the cycle-rotation diagram. Scatter 0.24 dex; the Sun itself sits between the branches, and this relation gives it 6.1 years against the observed 11.",
            formula: "log P_cyc[d] = a + n log P_rot[d]",
            f: |i, p| Ok(vec![cycle_period(positive(i[0], "rotation period")?, p[0], p[1])]),
        }
        .build(),
        Def {
            id: "sse.activity.cycle_period_active",
            version: "1.0.0",
            inputs: vec![port("period", "body.rotation_period", D::TIME, None)],
            outputs: vec![port("cycle", "star.activity_cycle_period", D::TIME, None)],
            params: vec![
                param("a", D::NONE, 2.22, (-2.0, 6.0), "intercept, log days"),
                param("n", D::NONE, 1.28, (0.0, 4.0), "slope"),
            ],
            cost: CostClass::Trivial,
            references: &[MITTAG23],
            curious: "Very active stars have cycles too, but longer for the same spin, as if a second deeper dynamo were running.",
            science: "The long-cycle (active) branch. Scatter about 40 %.",
            formula: "log P_cyc[d] = a + n log P_rot[d]",
            f: |i, p| Ok(vec![cycle_period(positive(i[0], "rotation period")?, p[0], p[1])]),
        }
        .build(),
        Def {
            id: "sse.activity.flare_rate_above",
            version: "1.0.0",
            inputs: vec![
                port("lx", "star.xray_luminosity", D::POWER, None),
                port("energy", "flare.energy", D::POWER.mul(D::TIME), None),
            ],
            outputs: vec![port("rate", "flare.rate_above", rate, None)],
            params: vec![
                param("k", D::NONE, FLARE_K, (0.0, 1.0), "flares above 10³⁴ erg per second, times 10³⁴ erg, per watt of X-rays (from the Sun-like superflare rate)"),
                param("alpha", D::NONE, 2.0, (1.1, 3.0), "index of the energy spread dN/dE ∝ E^−α (Shibayama 2013: 2.0; Vasilyev 2024: 1.97 ± 0.30)"),
            ],
            cost: CostClass::Trivial,
            references: &[VAS24, SHIB13],
            curious: "Small flares happen all the time and great ones rarely: ten times the energy, a tenth as often. A star like the Sun has a superflare about once a century; a young or red dwarf star, many a year.",
            science: "A power-law spread of flare energies whose rate scales with the star's X-ray luminosity (both are fed by the same magnetic energy). Normalised to Sun-like stars' superflare rate.",
            formula: "N(>E) = k (L_X / E₀) (E / E₀)^(1−α),  E₀ = 10³⁴ erg",
            f: |i, p| Ok(vec![flare_rate_above(i[0].max(0.0), positive(i[1], "energy")?, p[0], p[1])]),
        }
        .build(),
    ]
}
