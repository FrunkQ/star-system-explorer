//! MF2: the field ledger's Make test, the same sums for every conducting
//! layer (a planet's liquid core, a giant's metallic hydrogen, an ice giant's
//! ionic water, a star's envelope). The blocks are Equations in
//! `packs/base-physics/formulas/dynamo.json`; a test holds the two together.
//!
//! - `sse.dynamo.flow_speed`: how fast the layer stirs, from the buoyancy its
//!   power gives, its spin and its depth (Christensen and Aubert 2006);
//! - `sse.dynamo.reynolds`: the magnetic Reynolds number Rm = u D / η, and
//!   the time an unstirred field takes to leak away, D² / (π² η) (Keep);
//! - `sse.dynamo.onset`: 0 to 1 as Rm passes the dynamo threshold;
//! - `sse.dynamo.rossby`: spin period over stirring time, and the factor
//!   that weakens a slowly spinning star's field (saturated below it).
//!
//! No list of field-making materials: η = 1 / (μ0 σ) from the layer's own
//! conductivity, so iron, metallic hydrogen and ionic water differ only in σ.

use sse_core::math as m;

pub const CA06: &str = "Christensen, U. R. and Aubert, J. 2006, Geophysical Journal International 166, 97: scaling properties of convection-driven dynamos in rotating spherical shells; Ro = c Ra_Q*^b with Ra_Q* = (α g q / ρ c_p) / (Ω³ D²), and dynamos need Rm above about 40 to 50. doi:10.1111/j.1365-246X.2006.03009.x";
pub const MOF78: &str = "Moffatt, H. K. 1978, Magnetic Field Generation in Electrically Conducting Fluids, Cambridge University Press: free decay of a field in a conducting sphere, τ = D² / (π² η)";
pub const REI22: &str = "Reiners, A. et al. 2022, A&A 662, A41: magnetism, rotation and nonthermal emission in cool stars; the average field saturates below Rossby number about 0.13 and falls as Ro^-1.26 above it. doi:10.1051/0004-6361/202243251";

/// The flow-speed block's defaults.
pub const FLOW_C: f64 = 0.85;
pub const FLOW_B: f64 = 0.41;
/// The onset block's defaults: the field grows from Rm `lo` to `hi`.
pub const ONSET_LO: f64 = 40.0;
pub const ONSET_HI: f64 = 60.0;
/// The Rossby block's defaults.
pub const ROSSBY_SAT: f64 = 0.13;
pub const ROSSBY_SLOPE: f64 = 1.26;

/// Flow speed (m/s) and local Rossby number u / (Ω D) of a layer `d_m` deep
/// passing `q` W/m² of stirring power, with gravity `g`, expansivity
/// `alpha`, density `rho`, heat capacity `cp` and spin period `p_s`.
#[allow(clippy::too_many_arguments)]
pub fn flow_speed(
    q: f64,
    g: f64,
    alpha: f64,
    rho: f64,
    cp: f64,
    p_s: f64,
    d_m: f64,
    c: f64,
    b: f64,
) -> (f64, f64) {
    let f = g * alpha * q / (rho * cp);
    let w = 2.0 * m::PI / p_s;
    let ra = f.max(0.0) / ((w * w * w) * (d_m * d_m));
    let u = c * w * d_m * m::pow(ra, b);
    (u, u / (w * d_m))
}

/// Magnetic Reynolds number, and the free decay time (s), of a layer `d_m`
/// deep stirred at `u` with conductivity `sigma` (S/m).
pub fn reynolds(u: f64, d_m: f64, sigma: f64, mu0: f64) -> (f64, f64) {
    let eta = 1.0 / (mu0 * sigma);
    (u * d_m / eta, (d_m * d_m) / (m::PI * m::PI * eta))
}

/// How far a dynamo runs, 0 to 1, at magnetic Reynolds number `rm`.
pub fn onset(rm: f64, lo: f64, hi: f64) -> f64 {
    ((rm - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// Rossby number (spin period over stirring time D / u) and the factor on
/// the field: 1 when saturated, falling as (sat / Ro)^k above it.
pub fn rossby(p_s: f64, d_m: f64, u: f64, sat: f64, k: f64) -> (f64, f64) {
    let ro = p_s * u / d_m;
    let f = if ro <= sat { 1.0 } else { m::pow(sat / ro, k) };
    (ro, f)
}
