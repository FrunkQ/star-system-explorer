//! Layers within worlds: brines, ice shells, hydrostatic pressure (P4 atoms).

/// Molar mass of NaCl, kg/mol, from the sheet's atomic weights.
pub const NACL_MOLAR_MASS: f64 =
    (sse_core::sheet::atomic_weight::NA + sse_core::sheet::atomic_weight::CL) / 1000.0;
/// Water's cryoscopic constant, K kg/mol.
pub const WATER_KF: f64 = 1.86;
/// Ice Ih conductivity coefficient: k = ICE_K / T, W/m (the sheet's fit).
pub const ICE_K: f64 = sse_core::sheet::conductivity_solid::H2O[0];
