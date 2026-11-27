//! CS1: the climate's sunlight and year come from the world's own star, a
//! moon's seasons from its planet's year, and its albedo from its own
//! measurement.

use crate::scenario::{fixtures, settings};
use crate::*;
use sse_formation::hurley::Fits;
use sse_formation::lifeline::LifeLine;

const Z: f64 = 0.0142;

fn body(label: &str) -> Body {
    fixtures()
        .into_iter()
        .find(|f| f.body.label == label)
        .unwrap()
        .body
}

/// The base stack's physics, as a kernel call puts it in force.
fn physics() -> std::sync::Arc<sse_physics::Physics> {
    let packs: Vec<sse_pack::PackManifest> = ["base-reference", "base-physics"]
        .iter()
        .map(|n| {
            let path = format!("{}/../../packs/{n}/pack.json", env!("CARGO_MANIFEST_DIR"));
            sse_pack::PackManifest::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
        })
        .collect();
    let resolved = sse_pack::resolve(&packs, "0.1.0").expect("stack resolves");
    let sheet = std::sync::Arc::new(sse_reference::Reference::from_packs(&packs).unwrap());
    std::sync::Arc::new(sse_physics::Physics::build(
        &packs,
        Some(&resolved),
        sheet,
        Default::default(),
    ))
}

fn conditions_of(b: &Body) -> (sse_climate::Conditions, Vec<String>) {
    let s = settings();
    let env = envelope(&s, b);
    sse_physics::with(&physics(), sse_core::constants::Constants::REAL, || {
        climate::conditions(&s, b, &env, &Default::default())
    })
}

#[test]
fn with_nothing_in_force_the_climate_says_it_assumed_a_sun() {
    // Outside a kernel call there is no physics to read the star's light
    // with: the climate falls back and says so, it never guesses silently.
    let b = lit_by(body("Earth"), 0.5, 1.0, 4000.0);
    let s = settings();
    let env = envelope(&s, &b);
    let (c, assumed) = climate::conditions(&s, &b, &env, &Default::default());
    assert_eq!(c.flux_w_m2, sse_core::sheet::S_SUN);
    assert!(
        assumed.iter().any(|a| a.contains("sun-like")),
        "{assumed:?}"
    );
}

/// A world lit by a star of `msun` on its life line, `age_myr` old.
fn lit_by(mut b: Body, msun: f64, a_au: f64, age_myr: f64) -> Body {
    let life = LifeLine::new(&Fits::new(Z), msun).unwrap();
    b.age_gyr = age_myr / 1000.0;
    b.climate = Climate::Star {
        life: Box::new(life),
        z: Z,
        a_au,
        albedo: 0.3,
        greenhouse_k: 0.0,
        formed_myr: 0.0,
        others: vec![],
    };
    b.orbit = Some(Orbit {
        a_au,
        e: 0.0,
        host_msun: msun,
    });
    b
}

#[test]
fn a_dim_star_gives_its_own_light_not_the_suns() {
    // A 0.5 solar-mass dwarf at 1 AU: its own luminosity over 4πd², far
    // below the 1361 W/m² a sun-like star would give.
    let b = lit_by(body("Earth"), 0.5, 1.0, 4000.0);
    let (c, assumed) = conditions_of(&b);
    let l = match &b.climate {
        Climate::Star { life, .. } => life.state_at(&Fits::new(Z), 4000.0).luminosity,
        _ => unreachable!(),
    };
    let want =
        l * sse_core::sheet::L_SUN / (4.0 * m::PI * sse_core::sheet::AU * sse_core::sheet::AU);
    assert!(
        (c.flux_w_m2 - want).abs() < 1e-6 * want,
        "{} vs {want}",
        c.flux_w_m2
    );
    assert!(
        c.flux_w_m2 < 200.0,
        "a red dwarf at 1 AU gave {}",
        c.flux_w_m2
    );
    assert!(
        !assumed.iter().any(|a| a.contains("sun-like")),
        "{assumed:?}"
    );
    // Its year is Kepler's round half a Sun: √2 times longer.
    assert!(
        (c.year_days / 365.25 - m::sqrt(2.0)).abs() < 1e-3,
        "{}",
        c.year_days
    );
}

#[test]
fn a_measured_star_lights_its_worlds_with_its_measured_light() {
    // An anchored (catalogued) world under a star measured at a tenth of the
    // Sun's light gets a tenth of the solar constant at 1 AU, today.
    let mut b = body("Earth");
    let fits = Fits::new(Z);
    let life = LifeLine::new(&fits, 1.0).unwrap();
    b.age_gyr = 4.57;
    b.climate = Climate::Fixed {
        t_surface_k: 288.0,
        then: Some(Anchor {
            life: Box::new(life),
            z: Z,
            formed_myr: 0.0,
            measured_myr: 4570.0,
            luminosity_lsun: Some(0.1),
        }),
    };
    b.orbit = Some(Orbit {
        a_au: 1.0,
        e: 0.0167,
        host_msun: 1.0,
    });
    let s0 = sse_core::sheet::L_SUN / (4.0 * m::PI * sse_core::sheet::AU * sse_core::sheet::AU);
    let (c, _) = conditions_of(&b);
    assert!(
        (c.flux_w_m2 - 0.1 * s0).abs() < 1e-9 * s0,
        "{}",
        c.flux_w_m2
    );
    // The Sun as measured gives the solar constant (1361 W/m², IAU 2015 B3).
    if let Climate::Fixed { then: Some(an), .. } = &mut b.climate {
        an.luminosity_lsun = Some(1.0);
    }
    let (c, _) = conditions_of(&b);
    assert!((c.flux_w_m2 - 1361.0).abs() < 0.5, "{}", c.flux_w_m2);
}

/// The Moon, catalogued: round the Earth, carried round the Sun.
fn moon() -> Body {
    let mut b = body("Moon");
    let life = LifeLine::new(&Fits::new(Z), 1.0).unwrap();
    b.age_gyr = 4.57;
    b.climate = Climate::Fixed {
        t_surface_k: 250.0,
        then: Some(Anchor {
            life: Box::new(life),
            z: Z,
            formed_myr: 0.0,
            measured_myr: 4570.0,
            luminosity_lsun: Some(1.0),
        }),
    };
    b.orbit = Some(Orbit {
        a_au: 0.002_57,
        e: 0.0549,
        host_msun: 3.0e-6,
    });
    b.locked = Some(true);
    b.spin_hours = None;
    b.star_au = Some(1.0);
    b.star_e = Some(0.0167);
    b
}

#[test]
fn a_moons_seasons_follow_its_planets_year() {
    let (c, _) = conditions_of(&moon());
    // Absolute: the year is the Earth's round the Sun, not the month.
    assert!((c.year_days - 365.26).abs() < 0.2, "year {}", c.year_days);
    assert!(
        (c.eccentricity - 0.0167).abs() < 1e-12,
        "e {}",
        c.eccentricity
    );
    // Locked to the Earth: its day is still its month (27.3 days).
    let day_d = c.rotation_hours / 24.0;
    assert!((day_d - 27.3).abs() < 0.3, "day {day_d}");
    // And the sunlight is the Sun's at 1 AU.
    assert!((c.flux_w_m2 - 1361.0).abs() < 0.5, "flux {}", c.flux_w_m2);
}

#[test]
fn a_world_reads_its_own_measured_albedo() {
    let mut b = moon();
    let (c, assumed) = conditions_of(&b);
    assert_eq!(c.albedo, settings().albedo_default);
    assert!(
        assumed.iter().any(|a| a.contains("Bond albedo")),
        "{assumed:?}"
    );
    // Measured (NSSDCA): the Moon's Bond albedo is 0.11.
    b.bond_albedo = Some(0.11);
    let (c, assumed) = conditions_of(&b);
    assert_eq!(c.albedo, 0.11);
    assert!(
        !assumed.iter().any(|a| a.contains("Bond albedo")),
        "{assumed:?}"
    );
}
