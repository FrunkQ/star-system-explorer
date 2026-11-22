use super::*;

const AU2: f64 = 1.0; // densities below are given per au from the Sun
const MP: f64 = 1.672_621_9e-27;

/// The Sun's wind at d au: 5 protons per cm³ at 1 au, 440 km/s.
fn wind(d_au: f64) -> Wind {
    Wind {
        density_kg_m3: 5e6 * MP / (d_au * d_au) * AU2,
        speed_m_s: 4.4e5,
        bz_t: 0.0,
    }
}

/// Dipole moment from the equatorial surface field (T) and radius (m).
fn moment(b_eq: f64, r: f64) -> f64 {
    4.0 * m::PI / MU0 * b_eq * r * r * r
}

fn body(address: &str, r: f64, mom: f64, d_au: f64) -> Body {
    Body {
        address: address.into(),
        radius_m: r,
        moment_a_m2: mom,
        air: None,
        wind: wind(d_au),
        host: None,
        layers: vec![],
    }
}

const R_J: f64 = 7.1492e7;
const R_S: f64 = 6.0268e7;

fn jupiter_host(orbit_rj: f64, vary_nt: f64) -> Host {
    Host {
        address: "u:sol/sys:0/body:5".into(),
        radius_m: R_J,
        moment_a_m2: moment(4.17e-4, R_J),
        orbit_m: orbit_rj * R_J,
        plasma_pa: 0.0,
        varying_field_t: vary_nt * 1e-9,
        varying_period_s: 11.1 * 3600.0,
    }
}

/// Own-field stand-offs against measured noses: Earth about 10 radii
/// (Shue et al. 1998), Mercury about 1.5 (Anderson et al. 2011, doi:10.1126/science.1211001).
/// Jupiter's dipole alone gives about 40 radii; its plasma inflates the real
/// bubble to 60 to 90 (Joy et al. 2002), which waits on the plasma sources (MF7).
#[test]
fn own_field_noses() {
    let set = Settings::default();
    let e = read(&body("earth", 6.371e6, 8.0e22, 1.0), &set).unwrap();
    assert_eq!(e.obstacle.kind, "field");
    assert!(
        (8.0..=12.0).contains(&e.obstacle.standoff_radii),
        "{}",
        e.obstacle.standoff_radii
    );
    let a = e.flaring.unwrap();
    assert!((0.5..=0.7).contains(&a), "{a}");
    let me = read(
        &body("mercury", 2.44e6, moment(195e-9, 2.44e6), 0.387),
        &set,
    )
    .unwrap();
    assert_eq!(me.obstacle.kind, "field");
    assert!(
        (1.2..=1.9).contains(&me.obstacle.standoff_radii),
        "{}",
        me.obstacle.standoff_radii
    );
    let j = read(&body("jupiter", R_J, moment(4.17e-4, R_J), 5.2), &set).unwrap();
    assert!(
        (30.0..=50.0).contains(&j.obstacle.standoff_radii),
        "{}",
        j.obstacle.standoff_radii
    );
}

/// Bodies with air and no field: an induced boundary a few hundred km up
/// (Venus ionopause 250 to 400 km, Brace and Kliore 1991; Mars 300 to 800 km);
/// the airless Moon is struck at its surface.
#[test]
fn air_and_bare_surfaces() {
    let set = Settings::default();
    let mut v = body("venus", 6.052e6, 0.0, 0.723);
    v.air = Some(Air {
        surface_pressure_pa: 9.2e6,
        scale_height_m: 15.9e3,
    });
    let v = read(&v, &set).unwrap();
    assert_eq!(v.obstacle.kind, "air");
    let h = (v.obstacle.standoff_m - 6.052e6) / 1e3;
    assert!((125.0..=800.0).contains(&h), "{h}");
    let mut ma = body("mars", 3.39e6, 0.0, 1.524);
    ma.air = Some(Air {
        surface_pressure_pa: 610.0,
        scale_height_m: 11.1e3,
    });
    let ma = read(&ma, &set).unwrap();
    assert_eq!(ma.obstacle.kind, "air");
    let h = (ma.obstacle.standoff_m - 3.39e6) / 1e3;
    assert!((150.0..=1600.0).contains(&h), "{h}");
    let moon = read(&body("moon", 1.737e6, 0.0, 1.0), &set).unwrap();
    assert_eq!(moon.obstacle.kind, "surface");
}

/// Moons in a host's bubble: Ganymede's own field stands off Jupiter's about
/// two Ganymede radii out (Kivelson et al. 1998); Titan at 20.3 Saturn radii
/// crosses Saturn's edge as the wind gusts (Bertucci et al. 2008,
/// doi:10.1126/science.1165280).
#[test]
fn moons_inside_and_crossing() {
    let set = Settings::default();
    let mut g = body("ganymede", 2.634e6, moment(719e-9, 2.634e6), 5.2);
    g.host = Some(jupiter_host(15.0, 0.0));
    let g = read(&g, &set).unwrap();
    assert_eq!(g.pressed_by, "host field");
    assert_eq!(g.host_bubble.as_ref().unwrap().place, "inside");
    assert_eq!(g.obstacle.kind, "field");
    assert!(
        (1.5..=3.0).contains(&g.obstacle.standoff_radii),
        "{}",
        g.obstacle.standoff_radii
    );
    let mut t = body("titan", 2.575e6, 0.0, 9.54);
    t.air = Some(Air {
        surface_pressure_pa: 1.467e5,
        scale_height_m: 40e3,
    });
    t.host = Some(Host {
        address: "u:sol/sys:0/body:6".into(),
        radius_m: R_S,
        moment_a_m2: moment(2.1e-5, R_S),
        orbit_m: 20.3 * R_S,
        plasma_pa: 0.0,
        varying_field_t: 0.0,
        varying_period_s: 0.0,
    });
    let t = read(&t, &set).unwrap();
    assert_eq!(t.host_bubble.as_ref().unwrap().place, "crosses");
    assert_eq!(t.obstacle.kind, "air");
}

/// Europa's ocean answers Jupiter's swinging field close to a perfect
/// conductor (Kivelson et al. 2000: A about 0.97); a fresh-water layer barely answers.
#[test]
fn oceans_answer_a_changing_field() {
    let set = Settings::default();
    let mut e = body("europa", 1.561e6, 0.0, 5.2);
    e.host = Some(jupiter_host(9.4, 230.0));
    e.layers = vec![Layer {
        name: "salty ocean".into(),
        conductivity_s_m: 2.75,
        thickness_m: 100e3,
        outer_radius_m: 1.541e6,
    }];
    let f = read(&e, &set).unwrap();
    let a = f.induced[0].amplitude;
    assert!((0.97 * 0.7..=1.0).contains(&a), "{a}");
    e.layers[0].conductivity_s_m = 1e-4;
    let fresh = read(&e, &set).unwrap().induced[0].amplitude;
    assert!(fresh < 0.1, "{fresh}");
    assert!(f.references.iter().any(|r| r.text.contains("Zimmer")));
}
