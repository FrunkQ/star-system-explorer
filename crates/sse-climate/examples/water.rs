//! Prints every weather fixture's water supply, saltation and area labels by
//! latitude (W9): `cargo run -p sse-climate --example water`.
use serde::Deserialize;
use sse_climate::{saltation, water, Conditions};

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
        let cl = sse_climate::orbit(&cs, &c, 4500.0);
        println!(
            "== {} ({:.3} bar, {:?}) eddy {:.1} coll {:.2} winds {:?}",
            c.label,
            c.pressure_bar,
            c.liquid,
            cl.circulation.strengths.eddy_wind_m_s,
            cl.circulation.strengths.collisional,
            [-60.0, -30.0, -10.0, 0.0, 10.0, 30.0, 60.0]
                .map(|l| (cl.wind_at(l) * 10.0).round() / 10.0)
        );
        for lat in [
            -80.0, -60.0, -45.0, -30.0, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0, 45.0, 60.0, 80.0,
        ] {
            let ws = water::water_at(&cs, &cl, 6.5, lat, 0.0);
            let t = ws.t_k;
            let g = saltation::grain_for(&cs.saltation, t);
            let s = saltation::at(&cs.saltation, &cl, lat, g);
            let dune = saltation::dune_activity(&cs.saltation, s.transport_share, ws.aridity);
            let (a, _) = water::area(&cs, &ws);
            println!(
                "{lat:5.0} T {:5.1} [{:5.1},{:5.1}] P {:5.2} snow {:4.2} melt {:4.2} bal {:6.2} PE {:4.2} E {:4.2} R {:5.3} AI {:6.2} Pth {:6.2} | U {:5.2} thr {:6.1}/{:5.1} ini {:.1e} sus {:.1e} loft {:.2} W {:.1e} dune {:.2} | {a}",
                t, ws.t_cold_k, ws.t_warm_k, ws.precip_m_yr, ws.snow_m_yr, ws.melt_m_yr, ws.ice_balance_m_yr,
                ws.pet_m_yr, ws.evap_m_yr, ws.runoff_m_yr, ws.aridity.min(999.0), ws.p_over_pth.min(999.0),
                s.mean_wind_m_s, s.fluid_wind_m_s.min(9999.0), s.impact_wind_m_s.min(9999.0), s.initiation, s.sustained, s.lofting, s.transport_share, dune
            );
        }
    }
}
