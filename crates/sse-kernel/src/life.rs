//! `life_view` for a world on the chart (L3): the body's physical description,
//! as the surface and climate suites resolve it for the weather and the
//! survey (`body_conditions`), handed to the life suite's
//! `world_life::life`. Any system on the chart counts: catalogued, imported
//! from V3, grown by formation or typed in.

use crate::protocol::Response;
use crate::{adopt, err};
use sse_core::Address;

/// The outermost `sys` above `a` (the system a body belongs to).
fn system_of(a: &Address) -> Option<Address> {
    let mut chain = vec![a.clone()];
    while let Some(p) = chain.last().and_then(|x| x.parent()) {
        chain.push(p);
    }
    chain.into_iter().rev().find(|x| x.last().kind() == "sys")
}

fn code_of(r: &Response) -> String {
    serde_json::to_value(r)
        .ok()
        .and_then(|v| v["code"].as_str().map(str::to_string))
        .unwrap_or_default()
}

impl crate::Kernel {
    /// A world's life: (source, view, what was held or left out).
    pub(crate) fn body_life(
        &mut self,
        body: &str,
    ) -> Result<(String, serde_json::Value, Vec<String>), Response> {
        let (reading, seed) = self.body_reading(body)?;
        let s = sse_surface::scenario::settings();
        let life = sse_life::world_life::life(
            &reading,
            &[sse_life::scenario::base_life_pack()],
            &s.climate,
            &seed,
        )
        .map_err(|e| err("life_failed", format!("{body}: {e}")))?;
        Ok((
            body.to_string(),
            serde_json::to_value(&life.view).expect("plain data"),
            life.held,
        ))
    }

    /// What the life suite reads of a world now, and its seed text.
    pub(crate) fn body_reading(
        &mut self,
        body: &str,
    ) -> Result<(sse_life::world_life::WorldReading, String), Response> {
        let no = |why: String| err("no_habitats", why);
        let a: Address = body
            .parse()
            .map_err(|e| no(format!("{body} is not an address: {e}")))?;
        let sys = system_of(&a)
            .ok_or_else(|| no(format!("{body} is not a world of a system on the chart")))?;
        let system = sys.to_string();
        let (cj, epoch_myr, b, _) = match self.body_conditions(&system, body, None) {
            Ok(x) => x,
            Err(r) => {
                let code = code_of(&r);
                return Err(match code.as_str() {
                    "no_universe" => r,
                    "no_surface_body" | "unknown_system" | "bad_address" => no(format!(
                        "{body} is not a world the surface suites resolve from the chart's facts (they need its star, mass and orbit), so it has no habitats to read"
                    )),
                    _ => r,
                });
            }
        };
        let conditions: sse_climate::Conditions = serde_json::from_value(cj)
            .map_err(|e| err("life_failed", format!("{body}: conditions: {e}")))?;
        let s = sse_surface::scenario::settings();
        let env = sse_surface::envelope(&s, &b);
        let ice = sse_surface::ice_ground::ice_ground(&s, &b, &env, None);
        let climate = sse_climate::orbit(&s.climate, &conditions, epoch_myr);
        let age_gyr = epoch_myr / 1000.0;
        let heat = env
            .timeline
            .iter()
            .find(|x| x.from_gyr <= age_gyr && age_gyr <= x.to_gyr)
            .or(env.timeline.last())
            .map(|x| {
                let span = x.to_gyr - x.from_gyr;
                let f = if span > 0.0 {
                    ((age_gyr - x.from_gyr) / span).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                x.heat_flow_w_m2[0] + f * (x.heat_flow_w_m2[1] - x.heat_flow_w_m2[0])
            })
            .unwrap_or(env.tidal_w_m2);
        let u = self
            .universe
            .as_ref()
            .ok_or_else(|| err("no_universe", "open a universe first"))?;
        let w = adopt::worlds_at(&u.chronicle, &sys, None, u.runs.get(&system), &s)?;
        let star_teff = {
            use sse_formation::hurley::Fits;
            use sse_surface::Climate;
            match &b.climate {
                Climate::Star {
                    life,
                    z,
                    formed_myr,
                    ..
                } => Some(life.state_at(&Fits::new(*z), formed_myr + epoch_myr).teff),
                Climate::Fixed { then: Some(an), .. } => Some(
                    an.life
                        .state_at(&Fits::new(an.z), an.formed_myr + epoch_myr)
                        .teff,
                ),
                Climate::Fixed { then: None, .. } => None,
            }
        }
        .filter(|t| t.is_finite() && *t > 0.0)
        .or_else(|| {
            let st: Address = w.spec.as_ref()?.star_address.parse().ok()?;
            w.facts
                .get(&st)?
                .get("star.effective_temperature")
                .and_then(adopt::as_f64)
        })
        .unwrap_or(sse_core::sheet::T_SUN);
        let birth_year = w.spec.as_ref().map(|sp| sp.birth_year).unwrap_or_default();
        let has_water = b.oceans > 0.0
            || b.water_fraction > 0.0
            || ice.as_ref().is_some_and(|i| i.ice_share > 0.0)
            || conditions.gases.get("h2o").is_some_and(|x| *x > 0.0);
        let heat_history = heat_lost_by_age(&env, conditions.radius_m);
        let reading = sse_life::world_life::WorldReading {
            body: body.to_string(),
            name: b.label.clone(),
            conditions,
            climate,
            age_myr: epoch_myr,
            habitable_from_myr: b.magma_end_myr.max(0.0),
            heat_flow_w_m2: heat,
            star_teff_k: star_teff,
            regime: env.regime_now.clone(),
            oceans: b.oceans,
            ice_water_m: ice.filter(|i| i.ice).map(|i| i.shell_km * 1000.0),
            has_water,
            birth_year,
            heat_history,
        };
        let seed = format!("{}/life/{body}", u.seed.0.hex());
        Ok((reading, seed))
    }
}

/// A world's heat out of the top at each age, for life's deep-time history
/// (L3-heat): the interior's own thermal history (`core_history`), heat out
/// of the top over the surface. The one read of it, so the heat-flow
/// thread's coming `heat_lost` field (W/m² by age) and the time-scales
/// design's step times swap in here. Empty for a world without one.
fn heat_lost_by_age(
    env: &sse_surface::SurfaceEnvelope,
    radius_m: f64,
) -> Vec<sse_life::world_life::HeatSample> {
    let area = 4.0 * sse_core::math::PI * radius_m * radius_m;
    match (&env.core, area > 0.0) {
        (Some(h), true) => h
            .steps
            .iter()
            .filter(|x| x.q_surface_tw.is_finite())
            .map(|x| sse_life::world_life::HeatSample {
                at_myr: x.t_gyr * 1000.0,
                heat_flow_w_m2: x.q_surface_tw * 1.0e12 / area,
            })
            .collect(),
        _ => vec![],
    }
}
