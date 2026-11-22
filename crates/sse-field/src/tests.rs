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
        surface_gravity_m_s2: 0.0,
        free_space_dose_msv_day: None,
        orbits_m: vec![],
        magnetic_tilt_deg: 0.0,
        plasma_source_kg_s: 0.0,
        pedersen_conductance_s: 0.0,
        moons: vec![],
        rings_outer_m: 0.0,
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

fn within(got: f64, want: f64, factor: f64, what: &str) {
    assert!(
        got > want / factor && got < want * factor,
        "{what}: got {got}, want {want} within a factor of {factor}"
    );
}

/// Cut-off and dose against measurement. Earth: 14.9 GV at the magnetic
/// equator (Smart and Shea 2005); about 0.39 mSv a year from cosmic rays at
/// sea level (UNSCEAR 2008). Mars: 0.64 mSv a day on the surface at Gale
/// (Hassler et al. 2014, doi:10.1126/science.1244797). Moon: 1.37 mSv a day
/// on the surface (Zhang et al. 2020, doi:10.1126/sciadv.aaz1334).
#[test]
fn cosmic_ray_dose_follows_field_and_air() {
    let set = Settings::default();
    let mut e = body("earth", 6.371e6, 8.0e22, 1.0);
    e.air = Some(Air {
        surface_pressure_pa: 101_325.0,
        scale_height_m: 8_500.0,
    });
    e.surface_gravity_m_s2 = 9.81;
    e.orbits_m = vec![6.771e6, 4.2164e7, 1.0e9];
    let s = read(&e, &set).unwrap().shield.unwrap();
    within(
        s.by_latitude[0].cutoff_gv,
        14.9,
        1.05,
        "Earth equatorial cut-off",
    );
    assert!(s.by_latitude[6].cutoff_gv < 1e-6);
    within(s.surface_mean_msv_yr, 0.39, 2.0, "Earth sea-level dose");
    assert!(s.by_latitude[6].surface_dose_msv_yr > s.by_latitude[0].surface_dose_msv_yr);
    // In orbit: low orbit below free space, beyond the bubble nothing turns them aside.
    let low = &s.orbits[0];
    assert!(low.equator_msv_yr < low.pole_msv_yr && low.mean_msv_yr < s.free_space_msv_yr);
    assert!(
        s.orbits[2].cutoff_equator_gv == 0.0,
        "beyond the magnetopause"
    );
    assert!(s.orbits[0].mean_msv_yr > s.surface_mean_msv_yr * 100.0);

    let mut mars = body("mars", 3.3895e6, 0.0, 1.52);
    mars.air = Some(Air {
        surface_pressure_pa: 610.0,
        scale_height_m: 11_100.0,
    });
    mars.surface_gravity_m_s2 = 3.721;
    let s = read(&mars, &set).unwrap().shield.unwrap();
    within(
        s.surface_mean_msv_yr / 365.25,
        0.64,
        2.0,
        "Mars surface dose per day",
    );

    let moon = read(&body("moon", 1.7374e6, 0.0, 1.0), &set)
        .unwrap()
        .shield
        .unwrap();
    within(
        moon.surface_mean_msv_yr / 365.25,
        1.37,
        2.0,
        "Moon surface dose per day",
    );

    // Air without gravity: no dose, said so.
    let mut bare = e.clone();
    bare.surface_gravity_m_s2 = 0.0;
    let r = read(&bare, &set).unwrap();
    assert!(r.shield.is_none() && r.why.iter().any(|w| w.contains("surface gravity")));
}

/// A moon deep in a host's bubble is shielded by the host's field too.
#[test]
fn host_field_shields_its_moons() {
    let set = Settings::default();
    let mut eu = body("europa", 1.5608e6, 0.0, 5.2);
    eu.host = Some(jupiter_host(9.38, 0.0));
    let r = read(&eu, &set).unwrap();
    let s = r.shield.unwrap();
    assert!(s.host_cutoff_gv > 10.0, "{}", s.host_cutoff_gv);
    let bare = read(&body("rock", 1.5608e6, 0.0, 5.2), &set)
        .unwrap()
        .shield
        .unwrap();
    assert!(s.surface_mean_msv_yr < bare.surface_mean_msv_yr * 0.2);
}

/// Aurorae: Earth's oval near 67° to 75° (Feldstein and Starkov 1967),
/// moving equatorward in a gusty wind; Jupiter's spin-driven main oval
/// inside its wind oval with Io's spot near 66°; Uranus's ovals far from its
/// spin poles (tilt 59°, Ness et al. 1986, doi:10.1126/science.233.4759.85);
/// Venus a diffuse glow.
#[test]
fn aurorae_follow_the_drivers() {
    let set = Settings::default();
    let e = read(&body("earth", 6.371e6, 8.0e22, 1.0), &set).unwrap();
    let a = e.aurora.unwrap();
    let o = &a.ovals[0];
    assert_eq!(o.driver, "wind");
    assert!(
        (66.0..=76.0).contains(&o.magnetic_latitude_deg),
        "{}",
        o.magnetic_latitude_deg
    );
    let [storm, quiet] = o.latitude_range_deg.unwrap();
    assert!(storm < o.magnetic_latitude_deg && quiet > o.magnetic_latitude_deg);

    let mut j = body("jupiter", R_J, moment(4.17e-4, R_J), 5.2);
    j.plasma_source_kg_s = 1000.0;
    j.pedersen_conductance_s = 0.1;
    j.magnetic_tilt_deg = 9.6;
    j.moons = vec![
        MoonOrbit {
            address: "io".into(),
            orbit_m: 5.9 * R_J,
            radius_m: 0.0,
        },
        MoonOrbit {
            address: "callisto".into(),
            orbit_m: 26.3 * R_J,
            radius_m: 0.0,
        },
        MoonOrbit {
            address: "far".into(),
            orbit_m: 300.0 * R_J,
            radius_m: 0.0,
        },
    ];
    let a = read(&j, &set).unwrap().aurora.unwrap();
    let spin = a
        .ovals
        .iter()
        .find(|o| o.driver == "spin")
        .expect("Io's gas makes a main oval");
    assert!(spin.magnetic_latitude_deg < a.ovals[0].magnetic_latitude_deg);
    assert!(
        (20.0..=35.0).contains(&(spin.maps_to_m / R_J)),
        "{}",
        spin.maps_to_m / R_J
    );
    let io = a.footprints.iter().find(|f| f.moon == "io").unwrap();
    assert!((io.magnetic_latitude_deg - 65.7).abs() < 1.0);
    assert!(
        a.footprints.iter().all(|f| f.moon != "far"),
        "outside the bubble"
    );

    let r_u = 2.5559e7;
    let mut u = body("uranus", r_u, moment(2.3e-5, r_u), 19.2);
    u.magnetic_tilt_deg = 59.0;
    let a = read(&u, &set).unwrap().aurora.unwrap();
    assert!(
        a.ovals[0].spin_latitude_range_deg[0] < 30.0,
        "{:?}",
        a.ovals[0].spin_latitude_range_deg
    );

    let mut v = body("venus", 6.0518e6, 0.0, 0.72);
    v.air = Some(Air {
        surface_pressure_pa: 9.2e6,
        scale_height_m: 15_900.0,
    });
    v.surface_gravity_m_s2 = 8.87;
    assert!(read(&v, &set).unwrap().aurora.unwrap().diffuse);
}

fn sheet() -> Settings {
    // Electron mass, charge and ε₀ (CODATA 2018), typed only in tests.
    Settings {
        electron: Some(Electron {
            mass_kg: 9.109_383_701_5e-31,
            charge_c: 1.602_176_634e-19,
            eps0: 8.854_187_812_8e-12,
        }),
        ..Settings::default()
    }
}

/// Belts (MF7 part 2). Earth's run from the top of the air to about 7
/// radii and peak in the outer zone; Saturn's begin at the rings; a moon
/// sweeps a gap; a world without a field has none; without the sheet's
/// constants none are computed.
#[test]
fn belts_where_field_lines_close() {
    let set = sheet();
    let mut e = body("earth", 6.371e6, 8.0e22, 1.0);
    e.air = Some(Air {
        surface_pressure_pa: 101325.0,
        scale_height_m: 8500.0,
    });
    e.surface_gravity_m_s2 = 9.81;
    e.orbits_m = vec![6.771e6, 2.6560e7, 4.2164e7];
    let f = read(&e, &set).unwrap();
    let b = f.belts.expect("Earth has belts");
    assert_eq!(b.inner_set_by, "air");
    assert!(
        (1.1..=1.2).contains(&(b.inner_m / 6.371e6)),
        "{}",
        b.inner_m
    );
    assert!(
        (6.0..=8.5).contains(&(b.outer_m / 6.371e6)),
        "{}",
        b.outer_m
    );
    // Mauk and Fox Figure 5: near 2e3 to 4e3 per cm² s sr keV at 1 MeV, L 5.
    let l5 = b
        .shells
        .iter()
        .min_by(|a, c| (a.l - 5.0).abs().total_cmp(&(c.l - 5.0).abs()))
        .unwrap();
    assert!(
        (5e2..=2e4).contains(&(l5.intensity_1mev * m::pow(l5.l / 5.0, 4.0))),
        "{}",
        l5.intensity_1mev
    );
    // Shielding: each thicker shield lets less through.
    assert!(l5.dose_sv_h[0] > l5.dose_sv_h[1] && l5.dose_sv_h[1] > l5.dose_sv_h[2]);
    // No sheet, no belts; no field, no belts.
    assert!(read(&e, &Settings::default()).unwrap().belts.is_none());
    assert!(read(&body("mars", 3.39e6, 0.0, 1.52), &set)
        .unwrap()
        .belts
        .is_none());

    // Saturn: belts begin at the main rings' edge (2.27 radii).
    let mut s = body("saturn", R_S, moment(2.1e-5, R_S), 9.5);
    s.rings_outer_m = 2.27 * R_S;
    let sb = read(&s, &set).unwrap().belts.unwrap();
    assert_eq!(sb.inner_set_by, "rings");
    assert!((sb.inner_m / R_S - 2.27).abs() < 1e-9);
}

/// Jupiter: the moons' plasma confines the waves (D = 3), the spectrum stays
/// hard, Io sweeps a gap, and Europa's orbit gets far more dose than
/// Callisto's.
#[test]
fn jupiter_belts_and_moons() {
    let set = sheet();
    let mut j = body("jupiter", R_J, moment(4.17e-4, R_J), 5.2);
    j.plasma_source_kg_s = 1000.0;
    j.moons = [
        ("io", 5.9, 1.8216e6),
        ("europa", 9.4, 1.5608e6),
        ("ganymede", 15.0, 2.6341e6),
        ("callisto", 26.3, 2.4103e6),
    ]
    .iter()
    .map(|(n, a, rad)| MoonOrbit {
        address: (*n).into(),
        orbit_m: a * R_J,
        radius_m: *rad,
    })
    .collect();
    let b = read(&j, &set).unwrap().belts.unwrap();
    assert_eq!(b.wave_path, "confined");
    assert!(b.gaps.iter().any(|g| g.by == "io"));
    let dose = |n: &str| {
        b.orbits
            .iter()
            .find(|o| o.moon.as_deref() == Some(n))
            .unwrap()
            .dose_sv_h[1]
    };
    let ratio = dose("europa") / dose("callisto");
    assert!(ratio > 10.0, "{ratio}");
}

/// The Solar System fixture: six magnetised planets from base-sol and the
/// measured-fields fixture. Earth's dipole points south at about 30 µT
/// (IGRF-13 g10 is −29.4 µT), Jupiter's north; Earth's belts start at its
/// air, Saturn's at the A ring; Io sweeps a gap.
#[test]
fn solar_system_fixture() {
    let set = sheet();
    let all = solar_system(&set, 1.672_621_923_69e-27).unwrap();
    let ids: Vec<&str> = all.iter().map(|b| b.read.address.as_str()).collect();
    assert_eq!(
        ids,
        [
            "sol/mercury",
            "sol/earth",
            "sol/jupiter",
            "sol/saturn",
            "sol/uranus",
            "sol/neptune"
        ]
    );
    let get = |id: &str| all.iter().find(|b| b.read.address == id).unwrap();
    let e = get("sol/earth");
    assert!(
        (-31e-6..=-28e-6).contains(&e.field.g10_t),
        "{}",
        e.field.g10_t
    );
    assert!(e.field.g11_t < 0.0);
    assert!((8.0..=12.0).contains(&e.read.obstacle.standoff_radii));
    assert_eq!(e.read.belts.as_ref().unwrap().inner_set_by, "air");
    let j = get("sol/jupiter");
    assert!(j.field.g10_t > 4.0e-4, "{}", j.field.g10_t);
    let jb = j.read.belts.as_ref().unwrap();
    assert!(jb.gaps.iter().any(|g| g.by == "sol/io"));
    assert_eq!(
        get("sol/saturn").read.belts.as_ref().unwrap().inner_set_by,
        "rings"
    );
    let u = get("sol/uranus");
    assert!((u.field.tilt_deg - 58.6).abs() < 1e-9);
    // Degree-1 strength is the dipole's equatorial field at the reference radius.
    let b0 = m::sqrt(u.field.g10_t * u.field.g10_t + u.field.g11_t * u.field.g11_t);
    assert!((b0 - 2.28e-5).abs() / 2.28e-5 < 0.05, "{b0}");
    assert!(!e.field.source.is_empty() && !e.field.polarity_source.is_empty());
}
