//! Prints each giant fixture's bands, poles and storms: `cargo run -p sse-weather --example giants`.
use sse_weather::giants::{base_settings, fixtures, giant_systems, layout};
use sse_weather::Date;
use std::collections::BTreeMap;

fn main() {
    let gs = base_settings();
    for f in fixtures() {
        let c = &f.conditions;
        let l = layout(&gs, c);
        println!(
            "{:8} M {:5.0} ice {} jets {:2} H {:5.1} km Ld {:5.0} km Bu {:.2e} a/Ld {:5.1} jet {:5.1}° {:4.0} m/s Ld@jet {:5.0} waves {:5.2} | N {} {} poly {} | S {} {} poly {}",
            c.label,
            l.mass_earth,
            l.ice,
            l.jets,
            l.scale_height_km,
            l.deformation_km,
            l.burger,
            l.a_over_ld,
            l.polar_jet_lat_deg,
            l.polar_jet_m_s,
            l.jet_deformation_km,
            l.polygon_waves,
            l.poles[0].regime,
            l.poles[0].ring_count,
            l.poles[0].polygon_sides,
            l.poles[1].regime,
            l.poles[1].ring_count,
            l.poles[1].polygon_sides
        );
        for day in [0.0, 3000.0, 9000.0, 20000.0] {
            let s = giant_systems(&gs, c, Date { day });
            let mut kinds = BTreeMap::new();
            for x in &s {
                *kinds.entry(x.kind.clone()).or_insert(0) += 1;
            }
            println!("    day {day:6.0}: {kinds:?}");
        }
        let mut seen = std::collections::BTreeSet::new();
        for s in giant_systems(&gs, c, Date { day: 3000.0 }) {
            if seen.insert(s.kind.clone()) {
                println!(
                    "    {} [{}] {:.0} km: {}",
                    s.id, s.evidence, s.size_km, s.why
                );
            }
        }
    }
}
