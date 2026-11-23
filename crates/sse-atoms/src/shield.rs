//! Cosmic-ray shielding and dose (magnetic fields design 6.3, stage MF7
//! part 1). A dipole turns away charged particles below a cut-off rigidity
//! that depends on magnetic latitude and distance (Størmer); air then
//! absorbs what gets through. Each world's dose is computed from its own
//! field and air column, never from a "shielded" switch: shielding is an
//! effect, and a thick air with no field can shield better than a field
//! with no air.

use crate::light::MU0;
use sse_core::math as m;
use sse_core::quantity::consts::C;
use sse_core::Dimension as D;

/// Volt (rigidity is momentum per charge, in volts).
pub const VOLT: D = D([2, 1, -3, -1, 0, 0, 0]);
/// Mass per area (an air column), kg/m².
pub const COLUMN: D = D([-2, 1, 0, 0, 0, 0, 0]);
/// Dose rate, Sv/s (J/kg/s).
pub const DOSE_RATE: D = D([2, 0, -3, 0, 0, 0, 0]);

/// Størmer vertical cut-off rigidity, V, at distance r (m) from a dipole of
/// moment M (A m²), at magnetic latitude λ (radians).
pub fn cutoff_rigidity(moment: f64, r: f64, lat: f64) -> f64 {
    if moment <= 0.0 || r <= 0.0 {
        return 0.0;
    }
    let c = m::cos(lat).abs();
    MU0 * moment * C / (16.0 * m::PI * r * r) * (c * c * c * c)
}

/// Share of the free-space cosmic-ray dose carried by particles above
/// rigidity R (V): (1 + R/R₀)^−γ, a high-rigidity power law with the
/// solar-modulation turnover at R₀.
pub fn space_share(rigidity: f64, r0_gv: f64, gamma: f64) -> f64 {
    m::pow(1.0 + rigidity.max(0.0) * 1e-9 / r0_gv, -gamma)
}

/// Share of the polar ground rate left at cut-off R (V): the Dorman
/// function 1 − exp(−α R_c^−k), R_c in GV; 1 at no cut-off.
pub fn ground_share(rigidity: f64, alpha: f64, k: f64) -> f64 {
    let gv = rigidity * 1e-9;
    if gv <= 1e-6 {
        return 1.0;
    }
    1.0 - m::exp(-alpha * m::pow(gv, -k))
}

/// Settings for the air's dose block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AirDose {
    /// Attenuation length of the shower's nucleonic part, kg/m².
    pub lambda_n: f64,
    /// Attenuation length of its penetrating (muon) part, kg/m².
    pub lambda_mu: f64,
    /// Share of the top-of-air dose that turns into the penetrating part.
    pub muon_share: f64,
}

/// Dose rate below an air column X (kg/m²), from the free-space dose D₀ and
/// the shares of it that pass the cut-off in space and at the ground. The
/// shower's own cut-off response moves from the space share (thin air) to
/// the ground share (thick air) as the nucleonic part is absorbed. `sky`
/// is the share of the sky not blocked by the body itself.
pub fn air_dose(free: f64, column: f64, space: f64, ground: f64, sky: f64, a: &AirDose) -> f64 {
    let x = column.max(0.0);
    let w = m::exp(-x / a.lambda_n);
    let share = w * space + (1.0 - w) * ground;
    free * sky
        * ((1.0 - a.muon_share) * w * share + a.muon_share * m::exp(-x / a.lambda_mu) * ground)
}
