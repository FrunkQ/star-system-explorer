//! Ice ground (IG1): whether a world's outer layer is ice it can keep. The
//! blocks are Equations in `packs/base-physics/formulas/surface.json`; these
//! are the same sums for the surface tiles (a test holds the two together):
//!
//! - `sse.surface.ice_share_from_density`: the share of a small world's mass
//!   that is ice, from its mean density, mixing rock and ice by volume
//!   (self-compression left out, so a big icy moon comes out a little dry);
//! - `sse.surface.open_ice_loss`: how deep open ice is lost over a world's
//!   age with no air over it: the ice sublimes at its vapour pressure
//!   (Hertz-Knudsen), and of each molecule's ballistic hops a share is lost,
//!   broken up by starlight in flight (photolysis) or fast enough to escape
//!   (Jeans); the rest lands again.

use sse_core::math as m;

/// Share of the mass that is ice from the mean density, mixing by volume:
/// 1/ρ = w/ρ_ice + (1 − w)/ρ_rock, clamped to 0..1.
pub fn ice_share(rho: f64, rho_rock: f64, rho_ice: f64) -> f64 {
    if rho <= 0.0 || rho_rock <= rho_ice {
        return 0.0;
    }
    let w = (1.0 / rho - 1.0 / rho_rock) / (1.0 / rho_ice - 1.0 / rho_rock);
    w.clamp(0.0, 1.0)
}

/// Depth of open ice lost over `age_s`, m. `p_pa` the ice's vapour pressure
/// at `t_k`, `molar_kg` its molar mass, `g` and `v_esc` the world's surface
/// gravity and escape speed, `d_au` the distance from the star, `life_1au_s`
/// how long a molecule of the ice's vapour lasts in starlight at 1 AU (the
/// data sheet's `photochemistry.lifetime_1au`; it grows as d²).
#[allow(clippy::too_many_arguments)]
pub fn open_loss_m(
    p_pa: f64,
    t_k: f64,
    molar_kg: f64,
    g: f64,
    v_esc: f64,
    d_au: f64,
    age_s: f64,
    rho_ice: f64,
    life_1au_s: f64,
) -> f64 {
    if p_pa <= 0.0 || t_k <= 0.0 || g <= 0.0 || rho_ice <= 0.0 {
        return 0.0;
    }
    let r = sse_core::sheet::R_GAS;
    let flux = p_pa * m::sqrt(molar_kg / (2.0 * m::PI * r * t_k));
    let v2 = 2.0 * r * t_k / molar_kg;
    let hop_s = 2.0 * m::sqrt(v2) / g;
    let rate = if d_au > 0.0 && life_1au_s > 0.0 {
        1.0 / (life_1au_s * d_au * d_au)
    } else {
        0.0
    };
    let photo = 1.0 - m::exp(-hop_s * rate);
    let lam = v_esc * v_esc / v2;
    let jeans = (1.0 + lam) * m::exp(-lam);
    let share = (photo + jeans).min(1.0);
    flux * share * age_s.max(0.0) / rho_ice
}

/// Time for open ice ground to come within 1/e of a full dark lag of
/// dust and rock left by infall and sublimation, s (about 2.3 Gyr; fitted
/// so Callisto's 4.5 Gyr-old ground reflects about 0.19 and Ganymede's
/// about 2 Gyr-old ground about 0.43, Moore et al. 1999). A stand-in in the
/// fudge register.
pub const LAG_TAU_S: f64 = 7.3e16;

/// Share of open ice ground covered by a dark lag after `t_s` seconds open.
pub fn lag_share(t_s: f64, tau_s: f64) -> f64 {
    if tau_s <= 0.0 {
        return 0.0;
    }
    1.0 - m::exp(-t_s.max(0.0) / tau_s)
}

/// Reflectance of ice ground with a lag share `d` of dust of reflectance
/// `r_dust` over ice of reflectance `r_ice` (an areal mix).
pub fn lagged_reflectance(r_ice: f64, r_dust: f64, d: f64) -> f64 {
    let x = d.clamp(0.0, 1.0);
    (1.0 - x) * r_ice + x * r_dust
}
