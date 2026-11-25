//! What stains a world's surface (TRUE4, third audit R15 and R16): the
//! coatings that give a ground its colour over the rock or ice beneath, each a
//! process strength read off the world's own air, water, light and heat, never
//! a class gate (processes, not classes). The colour itself comes from each
//! stain's data-sheet optics in the renderer (sheet-optics.ts); this names the
//! material and how much of the surface it coats.
//!
//! 1. **Oxidised iron.** Iron in surface rock rusts where it meets liquid
//!    water, or an oxidising air: free oxygen, or carbon dioxide and water
//!    vapour that sunlight splits into oxidants (Mars's dust is coated in
//!    nanophase ferric oxide this way; Bibring et al. 2006, Science 312, 400;
//!    Morris et al. 2000, JGR 105, 1757). Reducing gases (methane, hydrogen)
//!    in the air cancel it. Rock worlds only.
//! 2. **Tholin.** Methane in the air or on the ground, broken by ultraviolet
//!    and charged particles, builds red-brown organics that settle out (Khare
//!    et al. 1984; Grundy et al. 2016, Science 351, aad9189 on Pluto). The
//!    dose is the methane's partial pressure times the starlight times age.
//! 3. **Sulphur and SO₂ frost.** A dry world whose heat flow is high enough
//!    to keep volcanoes going paints itself with sulphur and freezes SO₂ gas
//!    (Io: Carlson et al. 2007, Io after Galileo, ch. 9). Strength from the
//!    heat flow over a reference flow, on a world without oceans.
//! 4. **Frost.** The climate's own exchange of a volatile (sse-climate
//!    exchange.rs): the share of the surface whose bands hold its ice all
//!    year, at least a millimetre deep.
//!
//! Each rate constant is a calibration and is in the fudge register.

use crate::{Body, SurfaceEnvelope};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

pub const STAINS_VERSION: &str = "surface-stains-v1";

/// Oxidising partial pressure, bar, that rusts a surface in a billion years
/// (calibrated: Mars's 6 mbar of CO2 over 4 Gyr leaves its dust fully
/// oxidised, a world with none leaves none). Fudge register.
pub const OXIDISE_BAR_GYR: f64 = 1e-3;
/// Years of standing water, Gyr, that rust a surface (calibrated). Fudge register.
pub const OXIDISE_WET_GYR: f64 = 0.1;
/// Methane partial pressure, bar, times starlight, W/m², that coats a surface
/// in tholin in a billion years (calibrated so Pluto's 50 nbar under 0.9 W/m²
/// stains most of it and Triton's few nbar a little). Fudge register.
pub const THOLIN_BAR_W_GYR: f64 = 3e-8;
/// Heat flow, W/m², at which a dry world's volcanism coats half of it with
/// sulphur (Io's 2 to 3 W/m² coats nearly all; Veeder et al. 2012). Fudge register.
pub const SULPHUR_HALF_W_M2: f64 = 0.5;
/// Below this share a stain is not reported.
pub const STAIN_FLOOR: f64 = 0.02;
/// Ice a band must hold all year to count as frosted, kg/m² (a millimetre).
pub const FROST_KG_M2: f64 = 1.0;

/// One stain on a world's surface.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Stain {
    /// What it is: "ferric-oxide", "tholin", "sulphur", "frost".
    pub kind: String,
    /// The data-sheet row its colour comes from (a mixture or substance id).
    pub material_id: String,
    /// 0 to 1: the share of the surface it coats (frost), or how thoroughly
    /// it coats the ground it reaches (the rest).
    pub share: f64,
    pub why: String,
}

/// What the stains read from the world's climate (the survey's one climate).
pub struct StainInputs<'a> {
    /// Volume mixing ratios of the air's gases, by formula.
    pub gases: &'a std::collections::BTreeMap<String, f64>,
    pub pressure_bar: f64,
    /// Starlight at the world, W/m².
    pub flux_w_m2: f64,
    /// The volatile exchange, when the climate runs one.
    pub exchange: Option<&'a sse_climate::exchange::Exchange>,
    /// Band latitudes, degrees (for the frost's area).
    pub lat_deg: &'a [f64],
    /// The air's make-up is the pack's default mix, not the world's own: then
    /// nothing is read off it (no rust or tholin from a guessed air).
    pub gases_assumed: bool,
}

/// A world's stains, strongest first (none: bare rock or ice).
pub fn stains(b: &Body, env: &SurfaceEnvelope, i: &StainInputs) -> Vec<Stain> {
    let mut out = vec![];
    let age = b.age_gyr.max(0.0);
    let x = |g: &str| {
        if i.gases_assumed {
            0.0
        } else {
            i.gases.get(g).copied().unwrap_or(0.0).max(0.0)
        }
    };
    let rocky = !b.deep_envelope && b.water_fraction < 0.5;
    let q = env
        .timeline
        .last()
        .map(|s| s.heat_flow_w_m2[1])
        .unwrap_or(0.0)
        .max(0.0);
    // 1. Oxidised iron.
    if rocky {
        // Oxygen, and the oxidants sunlight makes from carbon dioxide and water vapour.
        let ox_bar = i.pressure_bar * (x("o2") + x("co2") + x("h2o"));
        let red_bar = i.pressure_bar * (x("ch4") + x("h2"));
        let net_bar = (ox_bar - 4.0 * red_bar).max(0.0);
        let wet_gyr = if b.oceans > 0.0 { age } else { 0.0 };
        let dose = net_bar / OXIDISE_BAR_GYR * age + wet_gyr / OXIDISE_WET_GYR;
        let share = 1.0 - m::exp(-dose);
        if share >= STAIN_FLOOR {
            out.push(Stain {
                kind: "ferric-oxide".into(),
                material_id: "mgs-1".into(),
                share,
                why: format!(
                    "Iron in its rock has rusted: {:.2} mbar of oxidising air{} over {age:.1} Gyr (coating {:.0}%).",
                    net_bar * 1e3,
                    if wet_gyr > 0.0 { " and standing water" } else { "" },
                    share * 100.0
                ),
            });
        }
    }
    // 2. Tholin.
    let ch4_bar = i.pressure_bar * x("ch4");
    let dose = ch4_bar * i.flux_w_m2.max(0.0) * age / THOLIN_BAR_W_GYR;
    let share = 1.0 - m::exp(-dose);
    if !b.deep_envelope && share >= STAIN_FLOOR {
        out.push(Stain {
            kind: "tholin".into(),
            material_id: "titan-tholin".into(),
            share,
            why: format!(
                "Methane in its air ({:.3} µbar) broken by {:.2} W/m² of starlight over {age:.1} Gyr has built red-brown tholin (coating {:.0}%).",
                ch4_bar * 1e6,
                i.flux_w_m2,
                share * 100.0
            ),
        });
    }
    // 3. Sulphur on a hot, dry world.
    if rocky && b.oceans <= 0.0 && q > 0.0 {
        let share = q / (q + SULPHUR_HALF_W_M2);
        let s = share * share;
        if s >= STAIN_FLOOR {
            out.push(Stain {
                kind: "sulphur".into(),
                material_id: "sulfur".into(),
                share: s,
                why: format!(
                    "Its heat flow of {q:.2} W/m² keeps dry volcanoes going that paint it with sulphur (coating {:.0}%).",
                    s * 100.0
                ),
            });
        }
    }
    // 4. Frost from the climate's exchange.
    if let Some(ex) = i.exchange {
        let (mut iced, mut all) = (0.0, 0.0);
        for (k, lat) in i.lat_deg.iter().enumerate() {
            let w = m::cos(lat.to_radians()).max(0.0);
            all += w;
            let ice = ex.ice_mean_kg_m2.get(k).copied().unwrap_or(0.0);
            if ex.perennial.get(k).copied().unwrap_or(false) && ice >= FROST_KG_M2 {
                iced += w;
            }
        }
        let share = if all > 0.0 { iced / all } else { 0.0 };
        if share >= STAIN_FLOOR {
            out.push(Stain {
                kind: "frost".into(),
                material_id: ex.gas.clone(),
                share,
                why: format!(
                    "{} frost lies all year over {:.0}% of it (the climate's exchange).",
                    ex.name,
                    share * 100.0
                ),
            });
        }
    }
    out.sort_by(|a, b| b.share.total_cmp(&a.share));
    out
}

/// A world's stains from its surveyed atlas (the one climate's air, light and
/// exchange), or none when the atlas has no layers (no ground).
pub fn stains_of(b: &Body, env: &SurfaceEnvelope, atlas: &crate::atlas::Atlas) -> Vec<Stain> {
    let Some(l) = atlas.layers() else {
        return vec![];
    };
    let k = &l.climate.world.climate;
    stains(
        b,
        env,
        &StainInputs {
            gases: &k.conditions.gases,
            pressure_bar: k.conditions.pressure_bar,
            flux_w_m2: k.conditions.flux_w_m2,
            exchange: k.exchange.as_ref(),
            lat_deg: &k.lat_deg,
            gases_assumed: l
                .climate
                .world
                .assumed
                .iter()
                .any(|a| a.contains("default mix")),
        },
    )
}

#[cfg(test)]
#[path = "stains_tests.rs"]
mod tests;
