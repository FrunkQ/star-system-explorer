use crate::impacts::*;
use crate::regimes::*;
use crate::scenario::settings;
use crate::wear::WaterHistory;
use crate::*;
use sse_core::draw::Seed;

fn b(j: &str) -> Body { serde_json::from_str(j).unwrap() }

#[test]
fn probe() {
    let s = settings();
    let worlds = [
     r#"{"address":"u:sol/sys:0/body:9","label":"Pluto","mass_earth":0.00218,"radius_earth":0.1863,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":40},"water_fraction":0.3,"pressure_bar":0.00001}"#,
     r#"{"address":"u:sol/sys:0/body:9/moon:1","label":"Charon","mass_earth":0.000271,"radius_earth":0.0951,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":53},"water_fraction":0.4,"locked":true}"#,
     r#"{"address":"u:sol/sys:0/body:8/moon:1","label":"Triton","mass_earth":0.00359,"radius_earth":0.2122,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":38},"water_fraction":0.25,"pressure_bar":0.000014,"locked":true}"#,
     r#"{"address":"u:sol/sys:0/body:6/moon:2","label":"Enceladus","mass_earth":0.0000181,"radius_earth":0.0396,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":75},"water_fraction":0.4,"tidal_w_m2":0.2,"locked":true}"#,
     r#"{"address":"u:sol/sys:0/belt:1/body:1","label":"Ceres","mass_earth":0.000157,"radius_earth":0.0738,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":168},"water_fraction":0.25}"#,
     r#"{"address":"u:sol/sys:0/body:6/moon:8","label":"Iapetus","mass_earth":0.000302,"radius_earth":0.1153,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":110},"water_fraction":0.7,"locked":true}"#,
     r#"{"address":"u:sol/sys:0/body:7/moon:5","label":"Miranda","mass_earth":0.0000110,"radius_earth":0.0370,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":60},"water_fraction":0.6,"locked":true}"#,
     r#"{"address":"u:sol/sys:0/body:5/moon:4","label":"Callisto","mass_earth":0.018,"radius_earth":0.378,"core_mass_fraction":0.0,"age_gyr":4.5,"climate":{"from":"fixed","t_surface_k":134},"water_fraction":0.5,"locked":true}"#,
    ];
    for j in worlds {
        let body = b(j);
        let e = envelope(&s, &body);
        let seed = Seed::from_text("probe");
        let inp = RegimeInputs { rotation_hours: None, mass_earth: body.mass_earth, impacts: None };
        let rs = regime_surface(&s, &e, &seed, &inp, body.age_gyr);
        let mut kinds: Vec<(String, usize, f64)> = vec![];
        for l in &rs.landforms {
            if let Some(k) = kinds.iter_mut().find(|k| k.0 == l.kind) { k.1 += 1; k.2 += rs.epoch_myr - l.formed_myr; }
            else { kinds.push((l.kind.clone(), 1, rs.epoch_myr - l.formed_myr)); }
        }
        let w = WaterHistory::steady(body.oceans, body.pressure_bar, body.age_gyr);
        let plates = crate::plates::history(&s, &e, &seed, body.age_gyr);
        let hist = ImpactHistory::lunar();
        let ii = ImpactInputs { history: &hist, water: &w, pressure_bar: body.pressure_bar, plates: Some(&plates) };
        let craters = impacts(&s, &e, &seed, &ii, body.age_gyr).map(|(f, _)| (f.craters.len(), f.erased)).ok();
        let r = e.reference_radius_m / 1000.0;
        eprintln!("PROBE {} regime={} kind={} q_now? T={} landforms={:?} chaos={:.4} craters={:?}",
            body.label, e.regime_now, e.kind, e.t_surface_now_k,
            kinds.iter().map(|k| format!("{} x{} mean {:.0} Myr", k.0, k.1, k.2 / k.1 as f64)).collect::<Vec<_>>(),
            rs.cover_fraction("chaos", r, 48), craters);
        for sp in &e.timeline { eprintln!("   span {:?}", serde_json::to_string(sp).unwrap().chars().take(200).collect::<String>()); }
    }
}
