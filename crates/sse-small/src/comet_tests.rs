//! SB4 part 1: comet activity from the energy balance.

use super::*;
use sse_core::{Address, Seed};
use sse_formation::leftovers::{Cloud, Spread, Tilt, ZETA_1_2};

fn sun() -> Host {
    Host {
        luminosity_lsun: 1.0,
        mass_msun: 1.0,
        age_myr: 4_570.0,
    }
}

fn icy_cloud() -> Cloud {
    Cloud {
        address: Address::universe("c").unwrap().child("cloud", 0).unwrap(),
        kind: "scattered-disc".into(),
        around: None,
        source: "belt-0".into(),
        a_min_au: 30.0,
        a_max_au: 60.0,
        a_log: false,
        e: Spread::Rayleigh { sigma: 0.3 },
        i: Tilt::Rayleigh { sigma_deg: 10.0 },
        // Rock, carbon and a water-rich ice with some CO2 and CO.
        solid: [300, 3_000, 2_000, 4_000, 600, 400],
        water_from: vec![],
        largest_km: 500.0,
        over_1km: 1e9,
        n_top: 1.0,
        norm: ZETA_1_2,
    }
}

/// A 4 km nucleus of albedo 0.06 (67P-like) made from the icy cloud.
fn nucleus() -> (Member, Vec<f64>, Settings) {
    let s = Settings::default();
    let c = icy_cloud();
    let a = Address::universe("c").unwrap().child("small", 1).unwrap();
    let mass = mass_for_diameter(&c, 4.0, &s);
    let f = made_field(&c, a, "made-comet", 3.5, mass);
    let mut m = resolve(&Seed::from_text("comet"), &f, 1, &sun(), None, None, &s);
    m.albedo = 0.06;
    let shares = ice_shares(&c, &s.comet);
    (m, shares, s)
}

#[test]
fn ices_wake_in_order_of_their_vapour_curves() {
    let (m, shares, s) = nucleus();
    let a = activity(&m, &shares, 1.0, &sun(), &s.comet);
    let onset = |id: &str| a.volatiles.iter().find(|v| v.id == id).unwrap().onset_au;
    let (w, c2, c) = (onset("h2o"), onset("co2"), onset("co"));
    // Observed: water drives activity inside about 3 AU, CO2 out to about 10 to 13, CO beyond.
    assert!((1.5..5.0).contains(&w), "water {w} AU");
    assert!((5.0..20.0).contains(&c2), "CO2 {c2} AU");
    assert!(c > 15.0 && c > c2 && c < 2000.0, "CO {c} AU");
}

#[test]
fn water_flux_and_a_67p_like_production_are_the_right_size() {
    let (m, shares, s) = nucleus();
    let a = activity(&m, &shares, 1.0, &sun(), &s.comet);
    let w = a.volatiles.iter().find(|v| v.id == "h2o").unwrap();
    // Molecules per square metre per second from free sublimation at 1 AU,
    // averaged over a fast-turning nucleus: 1e16 to 1e18 per cm^2.
    let per_m2 = w.flux_kg_m2_s / s.comet.volatiles[0].molar_kg_mol * s.comet.n_a;
    assert!((1e20..1e22).contains(&per_m2), "{per_m2:e} per m^2 s");
    // 67P near perihelion (1.24 AU): Rosetta saw of order 1e28 water molecules a second.
    let p = activity(&m, &shares, 1.24, &sun(), &s.comet);
    let q = p
        .volatiles
        .iter()
        .find(|v| v.id == "h2o")
        .unwrap()
        .production_per_s;
    assert!((1e27..1e29).contains(&q), "{q:e} per s");
    assert!(p.active);
    assert_eq!(p.dominant.as_deref(), Some("h2o"));
}

#[test]
fn coma_and_tails_grow_near_the_star_and_vanish_far_out() {
    let (m, shares, s) = nucleus();
    let near = activity(&m, &shares, 1.0, &sun(), &s.comet);
    assert!(
        (1e4..1e6).contains(&near.coma_km),
        "coma {} km",
        near.coma_km
    );
    assert!(
        near.dust_tail_km > 1e6,
        "dust tail {} km",
        near.dust_tail_km
    );
    assert!(near.ion_tail_km > near.coma_km);
    assert!(near.dust_kg_s > 0.0);
    // Far past CO's onset nothing stirs.
    let far = activity(&m, &shares, 500.0, &sun(), &s.comet);
    assert!(!far.active && far.dust_tail_km == 0.0 && far.dominant.is_none());
}

#[test]
fn a_rocky_body_has_no_activity() {
    let s = Settings::default();
    let mut m = nucleus().0;
    m.makeup = Makeup::from_array([0.2, 0.8, 0.0, 0.0]);
    let a = activity(&m, &[1.0, 0.0, 0.0], 1.0, &sun(), &s.comet);
    assert!(a.volatiles.is_empty() && !a.active);
}
