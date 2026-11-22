//! The Solar System's magnetised planets, as measured, for `read_field`'s
//! fixture "solar-system" (the renderer's field lines start here, MF14a).
//!
//! Radii and orbits come from `packs/base-sol`, moments, tilts and offsets
//! from `tests/fixtures/magnetic/measured-fields.json` (MF0), and the rest
//! (the solar wind, each dipole's sense, rings, Earth's air, Io's plasma,
//! moons) from `data/sol-field.json`, each value with its source. Nothing
//! is typed here.

use super::*;
use sse_core::quantity::consts::AU;

const SOL: &str = include_str!("../../../packs/base-sol/pack.json");
const MEASURED: &str = include_str!("../../../tests/fixtures/magnetic/measured-fields.json");
const EXTRA: &str = include_str!("../data/sol-field.json");

/// A measured field as Gauss coefficients (degree 1 for now).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasuredField {
    pub reference_radius_m: f64,
    /// Schmidt semi-normalised, T.
    pub g10_t: f64,
    pub g11_t: f64,
    pub h11_t: f64,
    pub degree: u32,
    pub tilt_deg: f64,
    /// Longitude of the dipole's pole, degrees (not in the fixture yet: 0).
    pub tilt_longitude_deg: f64,
    pub offset_radii: f64,
    /// +1 when the moment points along the spin axis's north (Jupiter), −1
    /// when it points south (Earth).
    pub polarity: f64,
    pub grade: String,
    pub model: String,
    pub source: String,
    pub polarity_source: String,
    pub stand_ins: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolField {
    pub name: String,
    pub field: MeasuredField,
    pub read: FieldRead,
}

fn num(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut x = v;
    for p in path {
        x = x.get(p)?;
    }
    x.as_f64()
}

fn text(v: &serde_json::Value, key: &str) -> String {
    match v.get(key) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(o) => o
            .get("url")
            .or_else(|| o.get("doi"))
            .and_then(|u| u.as_str())
            .map(|u| {
                let what = o.get("what").and_then(|w| w.as_str()).unwrap_or("");
                format!("{what} {u}").trim().to_string()
            })
            .unwrap_or_default(),
        None => String::new(),
    }
}

/// Every magnetised planet of the Solar System, read as measured.
/// `proton_mass_kg` (from the data sheet) turns the wind's count into mass.
pub fn solar_system(set: &Settings, proton_mass_kg: f64) -> Result<Vec<SolField>, String> {
    let sol: serde_json::Value = serde_json::from_str(SOL).map_err(|e| format!("base-sol: {e}"))?;
    let measured: serde_json::Value =
        serde_json::from_str(MEASURED).map_err(|e| format!("measured fields: {e}"))?;
    let extra: serde_json::Value =
        serde_json::from_str(EXTRA).map_err(|e| format!("sol-field data: {e}"))?;
    let sol_bodies = sol["provides"]["other"]["sol"]["bodies"]
        .as_array()
        .ok_or("base-sol has no bodies")?;
    let sol_of = |id: &str| sol_bodies.iter().find(|b| b["id"] == id);
    let w = &extra["wind"];
    let speed = num(w, &["speed_m_s"]).ok_or("wind speed")?;
    let n1 = num(w, &["proton_density_1au_m3"]).ok_or("wind density")?;
    let mut out = vec![];
    for m in measured["bodies"].as_array().ok_or("no measured bodies")? {
        let id = m["id"].as_str().unwrap_or_default();
        let moment = num(m, &["moment_a_m2"]).unwrap_or(0.0);
        let (sb, ex) = match (sol_of(id), extra["bodies"].get(id)) {
            (Some(sb), Some(ex)) if moment > 0.0 && sb["host"] == "sun" => (sb, ex),
            _ => continue,
        };
        let radius = num(sb, &["values", "body.radius", "v"]).ok_or("radius")?;
        let a = num(sb, &["values", "orbit.semi_major_axis", "v"]).ok_or("orbit")?;
        let tilt = num(m, &["tilt_deg"]).unwrap_or(0.0);
        let pol = num(ex, &["polarity"]).unwrap_or(1.0);
        let mut moons: Vec<MoonOrbit> = ex
            .get("moons")
            .and_then(|v| v.as_array())
            .map(|v| {
                v.iter()
                    .map(|mo| MoonOrbit {
                        address: format!("sol/{}", mo["id"].as_str().unwrap_or_default()),
                        orbit_m: num(mo, &["orbit_m"]).unwrap_or(0.0),
                        radius_m: num(mo, &["radius_m"]).unwrap_or(0.0),
                    })
                    .collect()
            })
            .unwrap_or_default();
        for mid in ex
            .get("moons_from_base_sol")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(mb) = mid.as_str().and_then(sol_of) {
                moons.push(MoonOrbit {
                    address: format!("sol/{}", mid.as_str().unwrap_or_default()),
                    orbit_m: num(mb, &["values", "orbit.semi_major_axis", "v"]).unwrap_or(0.0),
                    radius_m: num(mb, &["values", "body.radius", "v"]).unwrap_or(0.0),
                });
            }
        }
        let air = ex.get("air").map(|x| Air {
            surface_pressure_pa: num(x, &["surface_pressure_pa"]).unwrap_or(0.0),
            scale_height_m: num(x, &["scale_height_m"]).unwrap_or(0.0),
        });
        let body = Body {
            address: format!("sol/{id}"),
            radius_m: radius,
            moment_a_m2: moment,
            air,
            wind: Wind {
                density_kg_m3: n1 * proton_mass_kg * (AU / a) * (AU / a),
                speed_m_s: speed,
                bz_t: 0.0,
            },
            host: None,
            layers: vec![],
            surface_gravity_m_s2: num(ex, &["surface_gravity_m_s2"]).unwrap_or(0.0),
            free_space_dose_msv_day: None,
            orbits_m: vec![],
            magnetic_tilt_deg: tilt,
            plasma_source_kg_s: num(ex, &["plasma_source_kg_s"]).unwrap_or(0.0),
            pedersen_conductance_s: 0.0,
            moons,
            rings_outer_m: num(ex, &["rings_outer_m"]).unwrap_or(0.0),
        };
        let read = read(&body, set)?;
        let rr = num(m, &["reference_radius_m"]).unwrap_or(radius);
        let b0 = MU0 * moment / (4.0 * m::PI * rr * rr * rr);
        let t = tilt * m::PI / 180.0;
        let psrc = match id {
            "earth" | "mercury" => text(&extra["polarity_src"], id),
            _ => text(&extra["polarity_src"], "giants"),
        };
        out.push(SolField {
            name: m["name"].as_str().unwrap_or(id).to_string(),
            field: MeasuredField {
                reference_radius_m: rr,
                g10_t: pol * b0 * m::cos(t),
                g11_t: pol * b0 * m::sin(t),
                h11_t: 0.0,
                degree: 1,
                tilt_deg: tilt,
                tilt_longitude_deg: 0.0,
                offset_radii: num(m, &["offset_radii"]).unwrap_or(0.0),
                polarity: pol,
                grade: m["grade"].as_str().unwrap_or("").to_string(),
                model: m["model"].as_str().unwrap_or("").to_string(),
                source: text(m, "src"),
                polarity_source: psrc,
                stand_ins: vec![
                    "dipole only (degree 1) from the measured moment and tilt; the pole's longitude and the offset's direction are not in the fixture yet, so the pole leans toward longitude 0 and the offset has no direction".into(),
                    "the solar wind is ECSS's mean at 1 au, thinned as the inverse square of distance, with no field of its own".into(),
                ],
            },
            read,
        });
    }
    Ok(out)
}
