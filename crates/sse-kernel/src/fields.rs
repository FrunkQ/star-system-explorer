//! `read_field` with `system`: every world of an adopted system read for
//! what holds off the star's wind, how well its surface is shielded and
//! where its belts lie, so a chart can shade them all at once (gap item 7,
//! "which worlds are shielded, at a glance"; the Mag band at system scale,
//! MF14c, draws it).
//!
//! Each world is read from its facts: `body.magnetic_moment` (MF2: zero
//! when the dynamo is off, absent while unresolved), `field.tilt` (MF3,
//! 0 when absent), `atmosphere.surface_pressure`, `atmosphere.scale_height`
//! and `body.surface_gravity`, with the surface envelope's radius and
//! pressure where facts are missing. The wind is the `sse.light.stellar_wind`
//! block at each orbit. The Solar System's planets keep their measured
//! fields (the MF14a fixture), matched by name.

use crate::adopt::worlds_at;
use crate::err;
use crate::protocol::Response;
use sse_chronicle::Chronicle;
use sse_core::cbor::Value;
use sse_core::quantity::consts::{AU, R_EARTH};
use sse_core::Address;
use sse_formation::pipeline::SystemRun;

fn num(v: Option<&Value>) -> Option<f64> {
    match v? {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// The star's wind at `d` metres, from the stellar wind block's defaults.
fn wind_at(d: f64) -> Option<sse_field::Wind> {
    let atoms = sse_atoms::atoms();
    let a = atoms
        .iter()
        .find(|a| a.decl().id == "sse.light.stellar_wind")?;
    let p: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
    let out = a.eval(&[d], &p).ok()?;
    Some(sse_field::Wind {
        density_kg_m3: out[0],
        speed_m_s: out[1],
        bz_t: 0.0,
    })
}

/// A one-line digest of a world's read, for shading a chart.
fn glance(r: &sse_field::FieldRead, field: &str, dynamo_now: Option<bool>) -> serde_json::Value {
    let shield = r.shield.as_ref();
    let share = shield
        .filter(|s| s.free_space_msv_yr > 0.0)
        .map(|s| s.surface_mean_msv_yr / s.free_space_msv_yr);
    serde_json::json!({
        "held_off_by": r.obstacle.kind,
        "bubble_m": r.obstacle.standoff_m,
        "bubble_radii": r.obstacle.standoff_radii,
        "shielded": r.obstacle.kind == "field",
        "ground_dose_share": share,
        "belts_m": r.belts.as_ref().map(|b| [b.inner_m, b.outer_m]),
        "field": field,
        "dynamo_now": dynamo_now,
    })
}

pub(crate) fn read_system(
    ch: &Chronicle,
    system: &Address,
    at_s: Option<i64>,
    run: Option<&SystemRun>,
    sheet: &sse_reference::Reference,
) -> Result<Response, Response> {
    let ss = sse_surface::scenario::settings();
    let w = worlds_at(ch, system, at_s, run, &ss)?;
    let spec = w.spec.ok_or_else(|| {
        err(
            "no_orbits",
            format!("{system} has no star mass and age, so no orbits to read the wind at"),
        )
    })?;
    let set = crate::systems::field_settings(sheet);
    let sol: Vec<sse_field::SolField> = if spec.name.eq_ignore_ascii_case("sol") {
        sheet
            .constant("m_p")
            .and_then(|mp| sse_field::solar_system(&set, mp).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };
    let mut out = vec![];
    for p in &spec.planets {
        let addr: Address = p
            .address
            .parse()
            .map_err(|e| err("bad_address", format!("{e}")))?;
        let facts = w.facts.get(&addr);
        let fact = |k: &str| facts.and_then(|r| num(r.get(k)));
        let env = w.bodies.iter().find(|b| b.address == addr);
        if let Some(f) = sol.iter().find(|f| f.name.eq_ignore_ascii_case(&p.name)) {
            let mut v = serde_json::to_value(&f.read).expect("serialisable");
            v["address"] = serde_json::Value::String(p.address.clone());
            v["name"] = serde_json::Value::String(p.name.clone());
            v["glance"] = glance(&f.read, "measured", Some(true));
            out.push(v);
            continue;
        }
        let Some(radius) = env.map(|b| b.radius_earth * R_EARTH).filter(|r| *r > 0.0) else {
            continue;
        };
        // The surface envelope (as `resolved_system` builds it) gives the
        // gravity and, from MF1, whether a dynamo runs now and why.
        let envl = env.map(|b| sse_surface::envelope(&ss, b));
        let (moment, field) = match fact("body.magnetic_moment") {
            Some(m) if m > 0.0 => (m, "computed"),
            Some(_) => (0.0, "none"),
            None => (0.0, "unresolved"),
        };
        let pressure = fact("atmosphere.surface_pressure")
            .or_else(|| env.map(|b| b.pressure_bar * 1e5))
            .unwrap_or(0.0);
        let air_missing =
            pressure <= 0.0 && !fact("atmosphere.surface_pressure").is_some_and(|x| x == 0.0);
        let air = (pressure > 0.0).then(|| sse_field::Air {
            surface_pressure_pa: pressure,
            scale_height_m: fact("atmosphere.scale_height").unwrap_or(0.0),
        });
        let Some(wind) = wind_at(p.a_au.max(1e-3) * AU) else {
            return Err(err("bad_field", "the stellar wind block is missing"));
        };
        let body = sse_field::Body {
            address: p.address.clone(),
            radius_m: radius,
            moment_a_m2: moment,
            air,
            wind,
            host: None,
            layers: vec![],
            surface_gravity_m_s2: fact("body.surface_gravity")
                .or_else(|| envl.as_ref().map(|e| e.gravity_m_s2))
                .unwrap_or(0.0),
            free_space_dose_msv_day: None,
            orbits_m: vec![],
            magnetic_tilt_deg: fact("field.tilt").unwrap_or(0.0),
            plasma_source_kg_s: 0.0,
            pedersen_conductance_s: 0.0,
            moons: vec![],
            rings_outer_m: 0.0,
        };
        let mut r = sse_field::read(&body, &set).map_err(|e| err("bad_field", e))?;
        if field == "unresolved" {
            r.stand_ins.push("no magnetic moment fact yet (MF2 has not resolved this world), so it is read as having no field of its own".into());
            if let Some(c) = envl.as_ref().and_then(|e| e.core.as_ref()) {
                r.why.push(format!(
                    "Its core's own history (MF1) says the dynamo is {} now: {}",
                    if c.dynamo_now { "running" } else { "off" },
                    c.why_now
                ));
            }
        }
        if air_missing {
            r.stand_ins.push("no air facts yet for this world, so it is read as airless (no ionopause, no air column over the ground)".into());
        }
        if facts.is_none_or(|f| !f.contains_key("atmosphere.scale_height")) && pressure > 0.0 {
            r.stand_ins.push("no air scale height fact: the air's top is taken at the surface for the ionopause and the belts' inner edge".into());
        }
        r.stand_ins.push("the star's wind is the stellar wind block's solar defaults at this orbit, until the star's own wind comes from its activity (O10)".into());
        let mut v = serde_json::to_value(&r).expect("serialisable");
        v["name"] = serde_json::Value::String(p.name.clone());
        let dynamo = envl
            .as_ref()
            .and_then(|e| e.core.as_ref())
            .map(|c| c.dynamo_now);
        v["glance"] = glance(&r, field, dynamo);
        out.push(v);
    }
    Ok(Response::FieldRead { bodies: out })
}
