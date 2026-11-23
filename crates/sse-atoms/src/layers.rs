//! Layers within worlds: brines, ice shells, hydrostatic pressure (P4 atoms).

use crate::common::*;
use sse_core::Dimension as D;
use sse_flow::Atom;

/// Molar mass of NaCl, kg/mol.
pub const NACL_MOLAR_MASS: f64 = 0.058_44;
/// Water's cryoscopic constant, K kg/mol.
pub const WATER_KF: f64 = 1.86;
/// Ice Ih conductivity coefficient: k = ICE_K / T, W/m.
pub const ICE_K: f64 = 651.0;

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let _temp = |name: &str, kind: &str| port(name, kind, D::TEMPERATURE, None);
    vec![]
}
