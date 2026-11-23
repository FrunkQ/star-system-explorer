//! Layers within worlds: brines, ice shells, hydrostatic pressure (P4 atoms).

/// Molar mass of NaCl, kg/mol, from the sheet's atomic weights.
pub const NACL_MOLAR_MASS: f64 =
    (sse_core::sheet::atomic_weight::NA + sse_core::sheet::atomic_weight::CL) / 1000.0;
/// Water's cryoscopic constant, K kg/mol: R T_f² / L_f (L_f per kg), from the
/// sheet's gas constant, ice point and latent heat of fusion (about 1.86).
pub const WATER_KF: f64 = sse_core::sheet::R_GAS * sse_core::sheet::T0C * sse_core::sheet::T0C
    / sse_core::sheet::latent_fusion::H2O;
/// Ice Ih conductivity coefficient: k = ICE_K / T, W/m (the sheet's fit).
pub const ICE_K: f64 = sse_core::sheet::conductivity_solid::H2O[0];
