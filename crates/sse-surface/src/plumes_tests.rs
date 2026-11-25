use super::*;
use crate::envelope;
use crate::scenario::{fixtures, settings};

fn world(name: &str) -> (Body, SurfaceEnvelope) {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body;
    let e = envelope(&settings(), &b);
    (b, e)
}

fn seed() -> Seed {
    Seed::from_text("plumes-test")
}

#[test]
fn enceladus_jets_from_its_ocean_through_the_polar_shell() {
    let (b, e) = world("Enceladus");
    let p = plumes(&b, &e, &seed()).expect("Enceladus erupts");
    assert!(p.ocean > 0.5, "{p:#?}");
    assert!(p.strength > 0.3, "{p:#?}");
    // Escape about 240 m/s (Enceladus 239 m/s); gas near 1 km/s escapes.
    assert!(
        (200.0..280.0).contains(&p.escape_speed_m_s),
        "{}",
        p.escape_speed_m_s
    );
    assert!(p.gas_escapes);
    // Grains stand tens to a few hundred km high (Porco et al. 2006: plume
    // seen to several hundred km; grain scale heights of tens of km).
    let h = p.grain_height_m.expect("grains fall back") / 1000.0;
    assert!((30.0..600.0).contains(&h), "{h} km");
    // Hansen et al. 2006: about 200 kg/s; within a factor of three.
    assert!(
        (70.0..600.0).contains(&p.vapour_kg_s),
        "{} kg/s",
        p.vapour_kg_s
    );
    // The vents lie in a polar cap like the south polar terrain (to about 55°S).
    assert!((30.0..50.0).contains(&p.cap_deg));
    assert!(p.pole == 1 || p.pole == -1);
    assert!(p.why.len() >= 4);
}

#[test]
fn cold_thick_and_dry_worlds_have_no_plumes() {
    let s = Seed::from_text("plumes-test");
    for n in [
        "Callisto", "Ganymede", "Earth", "Mars", "Moon", "Io", "Jupiter", "Miranda",
    ] {
        let (b, e) = world(n);
        assert!(
            plumes(&b, &e, &s).is_none(),
            "{n}: {:?}",
            plumes(&b, &e, &s)
        );
    }
}

#[test]
fn more_heat_means_stronger_plumes_without_a_cut_off() {
    let (mut b, _) = world("Enceladus");
    let mut last = 0.0;
    for q in [0.005, 0.01, 0.02, 0.04] {
        b.tidal_w_m2 = Some(q);
        let e = envelope(&settings(), &b);
        let s = plumes(&b, &e, &seed()).map(|p| p.strength).unwrap_or(0.0);
        assert!(s >= last, "{q}: {s} < {last}");
        last = s;
    }
    assert!(last > 0.5);
}

#[test]
fn the_pole_is_a_draw_and_repeats() {
    let (b, e) = world("Enceladus");
    let a = plumes(&b, &e, &seed()).unwrap().pole;
    assert_eq!(a, plumes(&b, &e, &seed()).unwrap().pole);
    let poles: Vec<i8> = (0..16)
        .map(|i| {
            plumes(&b, &e, &Seed::from_text(&format!("p{i}")))
                .unwrap()
                .pole
        })
        .collect();
    assert!(poles.contains(&1) && poles.contains(&-1), "{poles:?}");
}
