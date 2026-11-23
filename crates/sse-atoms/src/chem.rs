//! Reaction energetics (track E, stage E0): the free energy a reaction
//! releases under local conditions. The life suite's gradient catalogue (E1)
//! is built on these two blocks.

use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::Atom;

/// Molar gas constant, J mol^-1 K^-1: the data sheet row (N_A k_B, exact).
pub const R_GAS: f64 = sse_core::sheet::R_GAS;
/// The temperature standard values are tabulated at, K.
pub const T_REF_K: f64 = 298.15;

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
    vec![]
}
