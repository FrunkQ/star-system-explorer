//! Prints every weather fixture's process strengths and label (W7b):
//! `cargo run -p sse-climate --example strengths`.
use serde::Deserialize;
use sse_climate::Conditions;

#[derive(Deserialize)]
struct Fx {
    conditions: Conditions,
}
#[derive(Deserialize)]
struct Fxs {
    worlds: Vec<Fx>,
}

fn main() {
    let f: Fxs =
        serde_json::from_str(include_str!("../../../tests/fixtures/weather/worlds.json")).unwrap();
    let cs = sse_climate::base_settings();
    for w in f.worlds {
        let c = w.conditions;
        let k = sse_climate::circulation::circulation(&cs, &c);
        let s = &k.strengths;
        println!(
            "{:18} {:15} cells {} edge {:4.0}  RoT {:9.3}  U {:6.1}/{:5.1}  sr {:.2}  bc {:.2}  dn {:.3}  night {:5.0} cond {:>5}  collapse {:.2}  coll {:.2}  trad {:8.2} d",
            c.label, k.regime, k.cells, k.hadley_edge_deg, s.thermal_rossby, s.wind_scale_m_s, s.eddy_wind_m_s,
            s.super_rotation, s.baroclinic, s.day_night, s.night_k,
            s.condense_k.map(|x| format!("{x:.0}")).unwrap_or("-".into()), s.collapse, s.collisional,
            s.radiative_s / 86400.0
        );
    }
}
