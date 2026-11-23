//! `chem` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

/// Molar gas constant, J mol^-1 K^-1 (CODATA 2018, exact from k and N_A).
pub const R_GAS: f64 = sse_core::sheet::R_GAS;
/// The temperature standard values are tabulated at, K.
pub const T_REF_K: f64 = 298.15;

const THAUER: &str = "Thauer, R. K., Jungermann, K. and Decker, K. 1977, Bacteriological Reviews 41, 100: energy conservation in chemotrophic anaerobic bacteria (free energies of formation, appendix). doi:10.1128/br.41.1.100-180.1977";
const AMEND: &str = "Amend, J. P. and Shock, E. L. 2001, FEMS Microbiology Reviews 25, 175: energetics of overall metabolic reactions of thermophilic and hyperthermophilic Archaea and Bacteria. doi:10.1111/j.1574-6976.2001.tb00576.x";
const LAROWE: &str = "LaRowe, D. E. and Amend, J. P. 2015, Frontiers in Microbiology 6, 718: power limits for microbial life. doi:10.3389/fmicb.2015.00718";

/// J mol^-1.
pub const MOLAR_ENERGY: D = D([2, 1, -2, 0, 0, -1, 0]);

/// Standard free energy at temperature `t_k` from the tabulated values at
/// 298.15 K, holding the reaction enthalpy constant (Gibbs-Helmholtz
/// integrated with constant ΔH°; the van 't Hoff approximation).
pub fn standard_gibbs_at(dg0_298: f64, dh0: f64, t_k: f64) -> f64 {
    dh0 - t_k * (dh0 - dg0_298) / T_REF_K
}

/// Free energy of reaction under actual activities: ΔG = ΔG° + R T ln Q.
pub fn reaction_gibbs(dg0: f64, t_k: f64, ln_q: f64) -> f64 {
    dg0 + R_GAS * t_k * ln_q
}

/// ln Q from activities and stoichiometric coefficients (negative for
/// reactants, positive for products).
pub fn ln_quotient(terms: &[(f64, f64)]) -> Result<f64, String> {
    let mut s = 0.0;
    for (nu, a) in terms {
        if *a <= 0.0 || !a.is_finite() {
            return Err(format!("activity must be positive, got {a}"));
        }
        s += nu * m::ln(*a);
    }
    Ok(s)
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![
        Def {
            id: "sse.chem.standard_gibbs_at_temperature",
            version: "1.0.0",
            inputs: vec![
                port("standard_gibbs_298", "chem.standard_gibbs_298", MOLAR_ENERGY, None),
                port("standard_enthalpy", "chem.standard_enthalpy", MOLAR_ENERGY, None),
                port("temperature", "chem.temperature", D::TEMPERATURE, Some((200.0, 450.0))),
            ],
            outputs: vec![port("standard_gibbs", "chem.standard_gibbs", MOLAR_ENERGY, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[AMEND, THAUER],
            curious: "Tables give a reaction's energy at 25 °C. A vent at 90 °C or an ice-moon ocean near 0 °C changes it, sometimes enough to decide whether anything can live on it.",
            science: "Integrated Gibbs-Helmholtz equation with the reaction enthalpy held at its 298.15 K value. Good to a few kJ per mole across liquid-water temperatures; beyond about 150 °C the heat-capacity terms of the full revised HKF model matter (Amend and Shock 2001 tabulate those).",
            formula: "ΔG°(T) = ΔH° − T (ΔH° − ΔG°₂₉₈) / 298.15 K",
            f: |i, _| {
                let t = positive(i[2], "temperature")?;
                Ok(vec![standard_gibbs_at(i[0], i[1], t)])
            },
        }
        .build(),
        Def {
            id: "sse.chem.reaction_gibbs",
            version: "1.0.0",
            inputs: vec![
                port("standard_gibbs", "chem.standard_gibbs", MOLAR_ENERGY, None),
                port("temperature", "chem.temperature", D::TEMPERATURE, Some((200.0, 450.0))),
                port("ln_quotient", "chem.ln_reaction_quotient", D::NONE, None),
            ],
            outputs: vec![port("gibbs", "chem.reaction_gibbs", MOLAR_ENERGY, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[THAUER, LAROWE],
            curious: "Whether a reaction can feed life depends on how much of each ingredient is actually there. Plenty of fuel and little waste makes it pay; a build-up of waste can shut it off.",
            science: "Free energy of reaction at the local activities. Negative means energy is released. Q is the product of activities raised to their stoichiometric coefficients (products positive, reactants negative); at pH 7 the hydrogen ion activity is 10^-7.",
            formula: "ΔG = ΔG° + R T ln Q",
            f: |i, _| {
                let t = positive(i[1], "temperature")?;
                Ok(vec![reaction_gibbs(i[0], t, i[2])])
            },
        }
        .build(),
    ]
}
