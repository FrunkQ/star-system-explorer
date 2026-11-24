//! RK12: effect at range, checked by hand.

use crate::reach::*;
use sse_core::math as m;
use sse_reference::Reference as Sheet;

fn beam(p: f64, eff: Option<f64>) -> Weapon {
    Weapon::Beam {
        beam_power_w: p,
        wavelength_m: 1e-6,
        aperture_m: 1.0,
        spread: 1.0,
        jitter_rad: 0.0,
        dwell_s: 1.0,
        efficiency: eff,
        reject_k: Some(600.0),
    }
}

fn target(takes: Option<f64>) -> Target {
    Target {
        skin: Skin {
            material: None,
            thickness_m: 0.01,
            effect: "boil".into(),
            start_k: None,
            absorbed_share: None,
            takes_j_m2: takes,
        },
        radius_m: 10.0,
        accel_m_s2: 0.0,
        closing_m_s: 0.0,
    }
}

fn ask(weapon: Weapon, target: Target, d: f64) -> Effect {
    effect_at_range(
        &Ask {
            weapon,
            target,
            distance_m: d,
        },
        &Sheet::base(),
    )
    .unwrap()
}

#[test]
fn a_beam_spreads_by_diffraction_and_its_rings_fall_as_the_square() {
    // 1 MW at 1 µm from a 1 m mirror, at 100 km: spot 1.22 λ d / D = 0.122 m,
    // 83.8% of the power inside it.
    let e = ask(beam(1e6, None), target(Some(1e7)), 1e5);
    let w = e.spot_radius_m.unwrap();
    assert!((w - 0.122).abs() < 1e-9, "{w}");
    let want = 0.838 * 1e6 / (m::PI * w * w);
    assert!((e.arrives_j_m2 / want - 1.0).abs() < 1e-12);
    assert_eq!(e.reading, "marginal", "{:?}", e.ratio);
    assert_eq!(e.takes_from.as_deref(), Some("authored"));
    // Fluence falls as the square, so the rings sit √2 either side of
    // where it is just enough.
    let r = &e.rings;
    let enough = r.enough_m.unwrap();
    assert!((r.in_range_m.unwrap() * m::sqrt(2.0) / enough - 1.0).abs() < 1e-6);
    assert!((r.marginal_m.unwrap() / m::sqrt(2.0) / enough - 1.0).abs() < 1e-6);
    let at = ask(beam(1e6, None), target(Some(1e7)), enough);
    assert!((at.ratio.unwrap() - 1.0).abs() < 1e-6);
    // Closer in it is plenty; further out it is a small share.
    assert_eq!(
        ask(beam(1e6, None), target(Some(1e7)), 2e4).reading,
        "in range"
    );
    assert_eq!(
        ask(beam(1e6, None), target(Some(1e7)), 1e6).reading,
        "out of range"
    );
}

#[test]
fn a_twenty_megawatt_beam_puts_forty_seven_on_the_heat_bus() {
    // Plan section 13: a 20 MW beam from a 30% efficient laser.
    let e = ask(beam(2e7, Some(0.3)), target(Some(1e7)), 1e5);
    let l = e.load.unwrap();
    assert!((l.electric_w - 2e7 / 0.3).abs() < 1e-3);
    let heat = l.electric_w - 2e7;
    assert!((heat / 1e6 - 46.667).abs() < 1e-3, "{heat}");
    assert_eq!(l.reject_k, Some(600.0));
    assert_eq!(l.role.as_deref(), Some("weapon"));
    assert!(ask(beam(2e7, None), target(Some(1e7)), 1e5).load.is_none());
}

#[test]
fn light_lag_is_the_real_limit_on_range() {
    // A 10 m target that can pull 1 g-ish (10 m/s²), one light second away:
    // two seconds of lag, 20 m of wander, one chance in four.
    let mut t = target(Some(1.0));
    t.accel_m_s2 = 10.0;
    let c = Sheet::base().constant("c").unwrap();
    let e = ask(beam(1e9, None), t.clone(), c);
    assert!((e.delay_s.unwrap() - 2.0).abs() < 1e-12);
    assert!((e.odds.unwrap() - 0.25).abs() < 1e-3, "{:?}", e.odds);
    // Even odds where ½ a t² = R √2: t² = 2√2 R / a, d = c t / 2.
    let tt = m::sqrt(2.0 * m::sqrt(2.0) * 10.0 / 10.0);
    let even = c * tt / 2.0;
    assert!((e.rings.even_odds_m.unwrap() / even - 1.0).abs() < 1e-6);
    // A target that cannot move is always hit.
    assert_eq!(ask(beam(1e9, None), target(Some(1.0)), c).odds, Some(1.0));
}

#[test]
fn a_burst_in_space_falls_as_the_square_and_leaves_delivery_to_nav() {
    // 1 kt (4.184e12 J), 70% as X-rays, at 10 km.
    let w = Weapon::Burst {
        yield_j: 4.184e12,
        xray_share: 0.7,
    };
    let e = ask(w, target(Some(2e3)), 1e4);
    let want = 0.7 * 4.184e12 / (4.0 * m::PI * 1e8);
    assert!((e.arrives_j_m2 / want - 1.0).abs() < 1e-12);
    assert_eq!(e.reading, "marginal");
    assert!(e.odds.is_none());
    assert!(e.why.iter().any(|w| w.contains("NAV")));
}

#[test]
fn a_slug_at_three_kilometres_a_second_carries_its_mass_in_explosive() {
    // ½ v² at 3 km/s is 4.5 MJ/kg; TNT is 4.184 MJ/kg.
    let w = Weapon::Projectile {
        mass_kg: 1.0,
        muzzle_m_s: 2000.0,
        face_m2: 0.01,
    };
    let mut t = target(Some(1e8));
    t.closing_m_s = 1000.0;
    t.accel_m_s2 = 1.0;
    let e = ask(w, t, 1e5);
    assert!((e.arrives_j_m2 - 4.5e8).abs() < 1e-3);
    assert_eq!(e.reading, "in range");
    // Energy does not fall with distance, so only the odds draw a ring.
    assert!(e.rings.enough_m.is_none() && e.rings.in_range_m.is_none());
    let even = e.rings.even_odds_m.unwrap();
    assert!(even > 1e3 && even < 1e5, "{even}");
    // Light lag plus a 33 s flight.
    let c = Sheet::base().constant("c").unwrap();
    assert!((e.delay_s.unwrap() - (1e5 / c + 1e5 / 3000.0)).abs() < 1e-9);
}

#[test]
fn a_skin_without_its_figures_says_which_are_missing() {
    let mut t = target(None);
    t.skin.material = Some("h2o".into());
    let e = ask(beam(1e6, None), t.clone(), 1e5);
    if e.takes_from.as_deref() == Some("sheet") {
        // The sheet has grown the figures: the reading is a real one.
        assert_ne!(e.reading, "unknown");
    } else {
        assert_eq!(e.reading, "unknown");
        assert!(!e.missing.is_empty());
        assert!(e.missing.iter().all(|k| k.starts_with("h2o.")));
        // A typed figure stands in, kept as authored.
        t.skin.takes_j_m2 = Some(1e7);
        let e = ask(beam(1e6, None), t, 1e5);
        assert_eq!(e.takes_from.as_deref(), Some("authored"));
    }
    let mut bad = target(None);
    bad.skin.material = Some("unobtainium".into());
    assert!(effect_at_range(
        &Ask {
            weapon: beam(1e6, None),
            target: bad,
            distance_m: 1e5
        },
        &Sheet::base()
    )
    .is_err());
}

#[test]
fn every_reading_cites_its_blocks() {
    let e = ask(beam(1e6, None), target(Some(1e7)), 1e5);
    let text: Vec<&str> = e.references.iter().map(|r| r.text.as_str()).collect();
    assert!(text.iter().any(|t| t.contains("Born")), "{text:?}");
    assert!(e
        .references
        .iter()
        .all(|r| r.doi.is_some() || r.url.is_some()));
}

#[test]
fn an_aluminium_skin_reads_its_heats_from_the_sheet() {
    // 1 cm of aluminium from 298.15 K: ρ t (H_m + L_f + H_b + L_v), every
    // figure read here from the sheet's row (NIST-JANAF, MP27).
    let sheet = Sheet::base();
    let row = sheet.row("mixtures", "aluminium").unwrap();
    let v = |k: &str| row[k]["v"].as_f64().unwrap();
    let want = v("density")
        * 0.01
        * (v("heat_to_melt") + v("latent_fusion") + v("heat_to_boil") + v("latent_vaporisation"));
    let mut t = target(None);
    t.skin.material = Some("aluminium".into());
    t.skin.start_k = Some(298.15);
    let e = ask(beam(1e6, None), t.clone(), 1e5);
    assert_eq!(e.takes_from.as_deref(), Some("sheet"), "{:?}", e.missing);
    assert!((e.takes_j_m2.unwrap() / want - 1.0).abs() < 1e-12);
    // About 382 MJ/m² to boil through: boiling is most of it.
    assert!((want / 1e6 - 382.0).abs() < 2.0, "{want}");
    // To melt is far less.
    t.skin.effect = "melt".into();
    let melt = ask(beam(1e6, None), t, 1e5).takes_j_m2.unwrap();
    assert!(melt < want / 10.0, "{melt}");
    // Tungsten has no boiling figures yet, and says so.
    let mut w = target(None);
    w.skin.material = Some("tungsten".into());
    let e = ask(beam(1e6, None), w, 1e5);
    assert_eq!(e.reading, "unknown");
    assert!(
        e.missing.iter().any(|k| k == "tungsten.heat_to_boil"),
        "{:?}",
        e.missing
    );
}
