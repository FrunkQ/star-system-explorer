//! Prints a fixture's one climate band by band: `cargo run -p sse-climate --example bands -- Earth`.
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
    let want = std::env::args().nth(1).unwrap_or_else(|| "Earth".into());
    let f: Fxs =
        serde_json::from_str(include_str!("../../../tests/fixtures/weather/worlds.json")).unwrap();
    let c = f
        .worlds
        .into_iter()
        .find(|w| w.conditions.label == want)
        .expect("fixture")
        .conditions;
    let k = sse_climate::orbit(&sse_climate::base_settings(), &c, 0.0);
    for j in 0..k.lat_deg.len() {
        println!(
            "{:6.1}  T {:6.1} ({:6.1}..{:6.1})  P {:5.2} m/yr  E {:5.2}  cover {:4.2}  thermal wind {:6.2}  prevailing {:6.2}",
            k.lat_deg[j], k.t_k[j], k.t_min_k[j], k.t_max_k[j], k.precip_m_yr[j], k.evap_m_yr[j],
            k.cover[j], k.wind_m_s[j], k.circulation.prevailing_wind(k.lat_deg[j])
        );
    }
    for w in &k.why {
        println!("{w}");
    }
}
