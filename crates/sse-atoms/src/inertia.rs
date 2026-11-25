//! A body's moment of inertia factor C = I / (M R²): how much of its mass
//! sits near the middle. It sets how much a spinning world flattens
//! (Darwin-Radau) and how it precesses. The blocks are Equations in
//! `packs/base-physics/formulas/interior.json`; these are the same sums for
//! the kernel's published fact (a test holds the two together):
//!
//! - `sse.interior.inertia_two_layer`: a dense core in a uniform mantle,
//!   for rocky and icy worlds (Earth about 0.35 uncompressed, 0.3307
//!   measured);
//! - `sse.interior.inertia_giant`: a giant as an n = 1 polytrope, whose C is
//!   2/3 − 4/π² (Hubbard 1984), less the part of its heavy elements gathered
//!   into a small central core.

use sse_core::math as m;

pub const H84: &str = "Hubbard, W. B. 1984, Planetary Interiors, Van Nostrand Reinhold: the n = 1 polytrope for a giant planet, density ρ = ρc sin(πr/R)/(πr/R), whose moment of inertia factor is 2/3 − 4/π² ≈ 0.261";
pub const ST02: &str = "Stacey, F. D. and Davis, P. M. 2008, Physics of the Earth, 4th edn, Cambridge University Press: the moment of inertia of a layered Earth, C/MR² = 0.3307 measured against 0.4 for a uniform sphere. doi:10.1017/CBO9780511812910";
pub const WAHL17: &str = "Wahl, S. M. et al. 2017, Geophysical Research Letters 44, 4649: Juno's gravity field asks for a dilute core, Jupiter's heavy elements spread through about half its radius rather than gathered in the middle. doi:10.1002/2017GL073160";

/// Share of a giant's heavy elements taken as a small central core; the
/// rest mixes through the envelope (a dilute core, Wahl et al. 2017).
pub const CORE_SHARE: f64 = 0.5;

/// C of a core of mass share `q` and radius `rc` inside a uniform mantle
/// out to `r`.
pub fn two_layer(q: f64, rc: f64, r: f64) -> f64 {
    if r <= 0.0 || rc >= r {
        return 0.4;
    }
    let q = q.clamp(0.0, 1.0);
    let x = (rc / r).max(0.0);
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    0.4 * (q * x2 + (1.0 - q) * (1.0 - x5) / (1.0 - x3))
}

/// C of a giant of heavy-element share `z`, `share` of it in a small core.
pub fn giant(z: f64, share: f64) -> f64 {
    (2.0 / 3.0 - 4.0 / (m::PI * m::PI)) * (1.0 - share.clamp(0.0, 1.0) * z.clamp(0.0, 1.0))
}
