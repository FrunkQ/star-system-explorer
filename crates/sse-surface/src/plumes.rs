//! Plumes above the limb (third audit R10): water vapour and ice grains
//! jetting from an ocean through cracks in an ice shell (Enceladus, and
//! perhaps Europa), as process strengths read from the world's own heat and
//! ice, never a class gate (processes, not classes).
//!
//! The chain, each link a number the drawing and the card can show:
//! 1. **An ocean under the ice.** The water layer's thickness H, from the
//!    world's water share as an outer shell over the rest, against the ice
//!    shell's conductive thickness d = 651 ln(T_melt / T_s) / q (the same
//!    shell [`crate::shell_activity`] uses). A shell thinner than the layer
//!    leaves liquid under it; the strength is smooth in (H / d)⁴.
//! 2. **Cracks that reach it, where the shell is thinnest.** Tidal heat in a
//!    thin shell is focused at the poles, and the shell thins there in
//!    response (Beuthe 2019; Čadek et al. 2016 find Enceladus's south polar
//!    shell about four times thinner than its mean). Radiogenic heat is not
//!    focused. The crack-through strength is [`crate::shell_activity`] at
//!    the polar heat flow.
//! 3. **How fast it leaves.** Vapour from water at its triple point expands
//!    into vacuum at the adiabatic limit v = √(2γ/(γ−1) · R T / M), γ = 4/3
//!    for a gas of bent three-atom molecules (three translations and three
//!    rotations). Grains condense in the vent and leave slower, at a share
//!    of the gas speed (Schmidt et al. 2008; Hedman et al. 2009).
//! 4. **How high it stands.** A grain thrown straight up rises
//!    h = R v² / (v_esc² − v²) (energy, not the flat-ground v²/2g), or
//!    escapes when v ≥ v_esc: then it feeds a ring or torus along the orbit.
//! 5. **How much.** A share of the world's tidal and radiogenic heat goes to
//!    evaporation at the vents: vapour rate = share · strength · P / L.
//!
//! Which pole is a draw: the two are alike to the tides, and which one
//! broke first is history the kernel does not have (Nimmo and Pappalardo
//! 2006 put Enceladus's hot spot at its pole by reorientation). A catalogue
//! may say which.
//!
//! Solar-driven nitrogen geysers (Triton) are a different engine and are
//! not here yet.

use crate::{
    shell_activity, Body, SurfaceEnvelope, K_ICE_651, M_EARTH_KG, RHO_ICE, R_EARTH_M, T_MELT_ICE_K,
};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;

pub const PLUMES_VERSION: &str = "surface-plumes-v1";

/// Polar focusing of a thin shell's tidal heat: polar shell thickness over
/// the mean, about 1/4 on Enceladus (Čadek et al. 2016, Geophys. Res. Lett.
/// 43, 5653, doi:10.1002/2016GL068634; Beuthe 2019, Icarus 332, 66,
/// doi:10.1016/j.icarus.2019.05.035). A model value (fudge register: one
/// number for every world, where a thin-shell tidal pattern would replace it).
pub const POLAR_FOCUS: f64 = 4.0;
/// Grain launch speed over gas speed: grains of 80 to 160 m/s (Hedman et al.
/// 2009, ApJ 693, 1749, doi:10.1088/0004-637X/693/2/1749) under gas near
/// 1 km/s; grains condensing in the channel are slower than the gas (Schmidt
/// et al. 2008, Nature 451, 685, doi:10.1038/nature06491). Model value.
pub const GRAIN_SPEED_SHARE: f64 = 0.12;
/// Share of the world's heat spent evaporating at the vents: Enceladus's
/// plume carries about 200 to 300 kg/s (Hansen et al. 2006, Science 311,
/// 1422, doi:10.1126/science.1121254) out of some 10 GW (Howett et al. 2011,
/// JGR 116, E03003, doi:10.1029/2010JE003718). Calibrated (fudge register).
pub const VENT_HEAT_SHARE: f64 = 0.05;
/// Under this strength nothing erupts that a picture or a card would show.
pub const PLUME_FLOOR: f64 = 0.01;

const R_GAS: f64 = sse_core::sheet::R_GAS;
const M_H2O: f64 = sse_core::sheet::molar_mass::H2O;
const T_TRIPLE: f64 = sse_core::sheet::triple_point_k::H2O;
const L_VAP: f64 = sse_core::sheet::latent_vaporisation::H2O;
const G: f64 = sse_core::sheet::G;

/// A world's vapour plumes now.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plumes {
    /// 0 to 1: an ocean under the ice times cracks reaching it at the pole.
    pub strength: f64,
    pub ocean: f64,
    pub crack: f64,
    /// +1 the north pole, −1 the south.
    pub pole: i8,
    /// The vents lie within this angle of the pole, degrees: the cap that
    /// holds the focused heat (a 1/POLAR_FOCUS share of the hemisphere).
    pub cap_deg: f64,
    pub water_layer_m: f64,
    pub shell_mean_m: f64,
    pub shell_pole_m: f64,
    pub gas_speed_m_s: f64,
    pub grain_speed_m_s: f64,
    pub escape_speed_m_s: f64,
    /// How high grains rise above the vents, m; None when they escape.
    pub grain_height_m: Option<f64>,
    /// The gas leaves faster than escape: it feeds a torus along the orbit.
    pub gas_escapes: bool,
    pub vapour_kg_s: f64,
    pub why: Vec<String>,
}

/// A world's plumes, or None when nothing erupts (no ice over water, no
/// ocean, or a shell too thick for cracks to cross at its pole).
pub fn plumes(b: &Body, env: &SurfaceEnvelope, seed: &Seed) -> Option<Plumes> {
    let r = b.radius_earth * R_EARTH_M;
    let mass = b.mass_earth * M_EARTH_KG;
    let t_s = env.t_surface_now_k;
    let q = env
        .timeline
        .last()
        .map(|s| s.heat_flow_w_m2[1])
        .unwrap_or(0.0);
    let w = crate::ice_ground::water_share(b).clamp(0.0, 1.0);
    if b.deep_envelope || r <= 0.0 || mass <= 0.0 || w <= 0.0 || q <= 0.0 || t_s >= T_MELT_ICE_K {
        return None;
    }
    // 1. The water layer as an outer shell of ice density over the rest.
    let v_body = 4.0 / 3.0 * m::PI * r * r * r;
    let v_water = (w * mass / RHO_ICE).min(v_body);
    let r_in = m::cbrt((v_body - v_water) * 3.0 / (4.0 * m::PI));
    let layer = r - r_in;
    let d_mean = K_ICE_651 * m::ln(T_MELT_ICE_K / t_s.max(1.0)) / q;
    let x = layer / d_mean.max(1.0);
    let x4 = x * x * x * x;
    let ocean = x4 / (1.0 + x4);
    // 2. Cracks through the polar shell: only the tidal share is focused.
    let tidal = env.tidal_w_m2.clamp(0.0, q);
    let q_pole = q - tidal + POLAR_FOCUS * tidal;
    let crack = shell_activity(t_s, q_pole);
    let strength = ocean * crack;
    if strength < PLUME_FLOOR {
        return None;
    }
    // 3. Speeds.
    let gamma = 4.0 / 3.0;
    let gas = m::sqrt(2.0 * gamma / (gamma - 1.0) * R_GAS * T_TRIPLE / M_H2O);
    let grain = GRAIN_SPEED_SHARE * gas;
    let esc = m::sqrt(2.0 * G * mass / r);
    // 4. Height.
    let height = |v: f64| (v < esc).then(|| r * v * v / (esc * esc - v * v));
    let grain_height_m = height(grain);
    // 5. Rate.
    let power = 4.0 * m::PI * r * r * q;
    let vapour_kg_s = VENT_HEAT_SHARE * strength * power / L_VAP;
    let cap_deg = m::acos(1.0 - 1.0 / POLAR_FOCUS) * 180.0 / m::PI;
    let d = Draws::new(
        *seed,
        b.address.clone(),
        Stream::new("surface.plumes.pole.v1").expect("stream name"),
    );
    let pole = if d.unit(0) < 0.5 { -1 } else { 1 };
    let km = |x: f64| x / 1000.0;
    let mut why = vec![
        format!(
            "Its water layer is about {:.0} km deep and the ice conducting its heat ({:.3} W/m²) about {:.0} km thick, so there is {} under the ice (ocean {:.2}).",
            km(layer),
            q,
            km(d_mean),
            if ocean >= 0.5 { "liquid" } else { "little liquid" },
            ocean
        ),
        format!(
            "Tidal heat gathers at the poles and thins the shell there to about {:.0} km, which cracks can cross (cracks {:.2}).",
            km(d_mean * q / q_pole),
            crack
        ),
        format!(
            "Vapour from water at its triple point leaves at about {:.0} m/s, grains at about {:.0} m/s, against an escape speed of {:.0} m/s.",
            gas, grain, esc
        ),
    ];
    why.push(match grain_height_m {
        Some(h) => format!(
            "Grains rise about {:.0} km above the vents before falling back.",
            km(h)
        ),
        None => "Grains leave faster than escape and spread along the orbit.".into(),
    });
    if gas >= esc {
        why.push("The gas escapes and feeds a torus along the orbit.".into());
    }
    why.push(format!("About {vapour_kg_s:.0} kg of vapour a second."));
    Some(Plumes {
        strength,
        ocean,
        crack,
        pole,
        cap_deg,
        water_layer_m: layer,
        shell_mean_m: d_mean,
        shell_pole_m: d_mean * q / q_pole,
        gas_speed_m_s: gas,
        grain_speed_m_s: grain,
        escape_speed_m_s: esc,
        grain_height_m,
        gas_escapes: gas >= esc,
        vapour_kg_s,
        why,
    })
}

#[cfg(test)]
#[path = "plumes_tests.rs"]
mod tests;
