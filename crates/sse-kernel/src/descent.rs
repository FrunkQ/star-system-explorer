//! `plan_descent`: S5's descent scene. A world is surveyed (through the same
//! cache as `survey_surface`), the landing site is judged on its one height
//! map, and NAV's descent flies a lander from the top of the air (or low
//! orbit on an airless world) to the ground there.

use crate::err;
use crate::protocol::Response;
use crate::survey::{surveyed, Cache, From};
use sse_core::math as m;
use sse_nav::descent::{descend, hazards, level, EntryState, Hazard, Lander, World};
use sse_surface::site::{site, SiteLimits};

/// Gas constant, J/(mol K).
const R_GAS: f64 = 8.314_462_618;
/// Highest entry interface searched for, m.
const TOP_SEARCH_M: f64 = 1.0e6;
/// Entry height on an airless world, m above the site.
pub const AIRLESS_ENTRY_M: f64 = 15_000.0;

pub(crate) struct Ask<'a> {
    pub from: From<'a>,
    pub options: Option<&'a serde_json::Value>,
    pub quick: bool,
    pub lander: &'a serde_json::Value,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub entry: Option<&'a serde_json::Value>,
}

fn bad(e: impl std::fmt::Display) -> Response {
    err("bad_request", e.to_string())
}

pub(crate) fn plan_descent(a: Ask, cache: &mut Cache) -> Result<serde_json::Value, Response> {
    let nav = sse_nav::scenario::settings();
    let d = &nav.descent;
    let lander: Lander = match a.lander {
        serde_json::Value::String(id) => d.landers.get(id).cloned().ok_or_else(|| {
            err(
                "unknown_lander",
                format!(
                    "no lander template {id}; templates: {}",
                    d.landers.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            )
        })?,
        other => serde_json::from_value(other.clone()).map_err(|e| bad(format!("lander: {e}")))?,
    };
    if !(-90.0..=90.0).contains(&a.lat_deg) || !a.lon_deg.is_finite() {
        return Err(bad("site: lat_deg in -90..90 and a finite lon_deg"));
    }
    let (source, body, at_s, sv) = surveyed(a.from, a.options, a.quick, cache)?;
    let layers = sv.atlas.layers().ok_or_else(|| {
        err(
            "no_ground",
            format!("{} has no surface to land on", body.label),
        )
    })?;
    let limits = SiteLimits {
        slope_limit_deg: d.slope_limit_deg,
        roughness_limit_m: d.roughness_limit_m,
        site_radius_m: d.site_radius_m,
    };
    let rep = site(&sv.atlas, a.lat_deg, a.lon_deg, &limits);
    let cover = sse_surface::scenario::settings().cover;
    let g = sv.envelope.gravity_m_s2;
    let radius = sv.atlas.radius_m;
    let air = &layers.air;
    let molar = air.molar_kg.max(1e-4);
    let r_spec = R_GAS / molar;
    let gamma = if air.cp_j_kg_k > r_spec {
        air.cp_j_kg_k / (air.cp_j_kg_k - r_spec)
    } else {
        1.4
    };
    let has_air = air.pressure_bar > 0.0;
    let air_fn = |z: f64| -> (f64, f64) {
        if !has_air {
            return (0.0, 0.0);
        }
        let (t, p_bar) = air.profile(&cover, g, z);
        let t = t.max(1.0);
        (p_bar * 1.0e5 / (r_spec * t), m::sqrt(gamma * r_spec * t))
    };
    let k = d
        .sutton_graves
        .get(&air.gas)
        .or_else(|| {
            air.gas
                .split_whitespace()
                .next()
                .and_then(|g| d.sutton_graves.get(g))
        })
        .copied()
        .unwrap_or(d.sutton_graves_default);
    let w = World {
        name: body.label.clone(),
        radius_m: radius,
        gravity_m_s2: g,
        ground_m: rep.height_m,
        air: &air_fn,
        sutton_graves_k: k,
        surface_wind_m_s: 0.0,
    };
    let entry = match a.entry {
        Some(e) => serde_json::from_value(e.clone()).map_err(|e| bad(format!("entry: {e}")))?,
        None => {
            // The top of the air, else a low orbit over the site.
            let mut z = rep.height_m.max(0.0);
            if has_air {
                while z < TOP_SEARCH_M && air_fn(z).0 > d.run.top_density {
                    z += 1000.0;
                }
            } else {
                z += AIRLESS_ENTRY_M;
            }
            let mu = g * radius * radius;
            EntryState {
                altitude_m: z,
                speed_m_s: m::sqrt(mu / (radius + z)),
                flight_path_deg: if has_air { d.entry_fpa_deg } else { 0.0 },
            }
        }
    };
    let out = descend(&d.run, &w, &lander, &entry);
    let landed = matches!(out.outcome.result.as_str(), "landed");
    let verdict = match (landed, rep.ok) {
        (true, true) => "landed",
        (true, false) => "landed-on-a-bad-site",
        _ => out.outcome.result.as_str(),
    };
    let (rho_g, a_g) = air_fn(rep.height_m);
    let site_h = |kind: &str, lv: &str, why: String| Hazard {
        kind: kind.into(),
        level: lv.into(),
        source: "site".into(),
        why,
    };
    let mut hz = vec![
        site_h(
            "slope",
            level(rep.slope_deg.max(rep.local_slope_deg), d.slope_limit_deg),
            format!(
                "{:.1}° across the site, {:.1}° locally, limit {:.0}°",
                rep.slope_deg, rep.local_slope_deg, d.slope_limit_deg
            ),
        ),
        site_h(
            "roughness",
            level(rep.roughness_m, d.roughness_limit_m),
            format!(
                "{:.1} m about the mean ground, limit {:.0} m",
                rep.roughness_m, d.roughness_limit_m
            ),
        ),
    ];
    if let Some(l) = &rep.under_liquid {
        hz.push(site_h(
            "liquid",
            "danger",
            format!("the site lies under {l}"),
        ));
    }
    hz.extend(hazards(&lander, &out.outcome));
    let mut o = serde_json::json!({
        "body": body.address.to_string(),
        "label": body.label,
        "source": source,
        "lander": lander,
        "entry": entry,
        "site": rep,
        "air_at_site": {"density_kg_m3": rho_g, "sound_m_s": a_g, "gas": has_air.then_some(&air.gas)},
        "gravity_m_s2": g,
        "hazards": hz,
        "verdict": verdict,
        "descent": out,
    });
    if let Some(t) = at_s {
        o["at_s"] = t.into();
    }
    Ok(o)
}
