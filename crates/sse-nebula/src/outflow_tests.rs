use crate::outflow::*;
use crate::{Kind, PC_M, YEAR_S};
use sse_core::Seed;

fn sheet(id: &str) -> f64 {
    match id {
        "L_sun" => sse_core::sheet::L_SUN,
        "R_sun" => sse_core::sheet::R_SUN,
        "M_sun" => sse_core::sheet::M_SUN,
        _ => panic!("{id}"),
    }
}

fn giant(ends: Option<f64>) -> GiantInput {
    GiantInput {
        address: "u:t/sys:0/neb:shell".into(),
        star: "u:t/sys:0/star:0".into(),
        luminosity_w: 5000.0 * sheet("L_sun"),
        radius_m: 200.0 * sheet("R_sun"),
        mass_kg: 1.2 * sheet("M_sun"),
        teff_k: 3000.0,
        started_s: 0.0,
        ends_s: ends,
        wind_m_s: None,
        ambient_h_m3: 1e5,
    }
}

#[test]
fn an_agb_star_blows_a_dusty_wind_that_comes_away_when_it_stops() {
    let seed = Seed::from_text("t");
    assert!(dust_shell(&seed, &giant(None), -1.0).unwrap().is_none());
    let at = 1e4 * YEAR_S;
    let d = dust_shell(&seed, &giant(None), at).unwrap().unwrap();
    assert_eq!(d.kind, Kind::DustShell);
    assert_eq!(d.phase, "blowing");
    // A 5000 L☉, 200 R☉ giant at 3000 K sheds a few 1e-7 to 1e-6 M☉ a year
    // (the AGB's range); the mass in the shell is that rate times its age.
    let rate = d.shells[0].mass_kg / at * YEAR_S / sse_core::sheet::M_SUN;
    assert!(rate > 1e-7 && rate < 1e-5, "{rate}");
    // 15 km/s for 10 000 years is about 0.15 pc.
    let r = d.shells[0].radius_m / PC_M;
    assert!((r - 0.153).abs() < 0.01, "{r}");
    assert!(d.light.parts.iter().any(
        |p| matches!(p, sse_light::Part::Thermal { t_k, .. } if (*t_k - 1200.0).abs() < 1e-9)
    ));
    // Once it stops the shell detaches, cools, and drifts out.
    let late = dust_shell(&seed, &giant(Some(at)), 2.0 * at)
        .unwrap()
        .unwrap();
    assert_eq!(late.phase, "detached");
    assert!(late.shells[0].thickness_m < late.shells[0].radius_m);
    let t = late
        .light
        .parts
        .iter()
        .find_map(|p| match p {
            sse_light::Part::Thermal { t_k, .. } => Some(*t_k),
            _ => None,
        })
        .unwrap();
    assert!(t < 200.0, "{t}");
    assert!(dust_shell(&seed, &giant(Some(at)), 1e7 * YEAR_S)
        .unwrap()
        .is_none());
}

#[test]
fn a_young_star_fires_knotted_jets_along_its_axis_while_it_feeds() {
    let seed = Seed::from_text("t");
    let p = YoungStarInput {
        address: "u:t/sys:0/neb:jet".into(),
        star: "u:t/sys:0/star:0".into(),
        mass_kg: 0.5 * sheet("M_sun"),
        radius_m: 2.0 * sheet("R_sun"),
        accretion_kg_s: 1e-7 * sheet("M_sun") / YEAR_S,
        started_s: 0.0,
        ends_s: 1e6 * YEAR_S,
        axis: [0.0, 0.0, 1.0],
        launch_radius_m: None,
        knot_period_s: None,
        ambient_h_m3: 1e9,
    };
    let at = 1000.0 * YEAR_S;
    let d = young_jet(&seed, &p, at).unwrap().unwrap();
    assert_eq!(d.kind, Kind::Jet);
    let j = d.jet.as_ref().unwrap();
    // Escape speed from five radii of a 0.5 M☉, 2 R☉ star: about 140 km/s.
    assert!(j.speed_m_s > 1e5 && j.speed_m_s < 2e5, "{}", j.speed_m_s);
    assert_eq!(j.knots.len(), 12);
    assert!((j.mass_rate_kg_s / p.accretion_kg_s - 0.1).abs() < 1e-12);
    assert_eq!(d.lobes.as_ref().unwrap().axis, [0.0, 0.0, 1.0]);
    assert!(d.light.lines[0].power_w > 0.0);
    assert!(young_jet(&seed, &p, 2e6 * YEAR_S).unwrap().is_none());
}
