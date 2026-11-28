use super::*;
use crate::figure as fg;
use sse_core::{Address, Seed};
use sse_formation::leftovers::{Cloud, Spread, Tilt, ZETA_1_2};

fn sun() -> Host {
    Host {
        luminosity_lsun: 1.0,
        mass_msun: 1.0,
        age_myr: 4_570.0,
    }
}

/// A hand-built cloud (formation fills `n_top`, `norm` and `over_1km` itself).
fn cloud(kind: &str, a: [f64; 2], e: f64, i_deg: f64, solid: [u64; 6], over_1km: f64) -> Cloud {
    Cloud {
        address: Address::universe("t").unwrap().child("cloud", 0).unwrap(),
        kind: kind.into(),
        around: None,
        source: "belt-0".into(),
        a_min_au: a[0],
        a_max_au: a[1],
        a_log: false,
        e: Spread::Rayleigh { sigma: e },
        i: Tilt::Rayleigh { sigma_deg: i_deg },
        solid,
        water_from: vec![],
        largest_km: 900.0,
        over_1km,
        n_top: 1.0,
        norm: ZETA_1_2,
    }
}

fn main_belt() -> Cloud {
    // Mass shares as formation gives a sunlike system's belt (scenario system 5).
    cloud(
        "asteroid-belt",
        [2.1, 3.3],
        0.12,
        8.0,
        [2_860, 5_550, 1_590, 0, 0, 0],
        1.2e6,
    )
}

fn cold_kuiper() -> Cloud {
    cloud(
        "outer-belt",
        [42.0, 47.0],
        0.04,
        2.0,
        [500, 3_000, 2_000, 4_000, 300, 200],
        1.0e8,
    )
}

fn many(c: &Cloud, ranks: std::ops::Range<u64>) -> Vec<Member> {
    let seed = Seed::from_text("sb1");
    let s = Settings::default();
    ranks
        .map(|k| resolve(&seed, c, k, &sun(), None, None, &s))
        .collect()
}

#[test]
fn pack_parses_and_every_number_is_named() {
    let s = Settings::default();
    assert_eq!(s.pairs.yorp_contact, 0.11);
    assert_eq!(s.pairs.yorp_separated, 0.16);
    assert_eq!(s.recipes.len(), 4);
    for r in &s.recipes {
        for i in &r.inputs {
            assert!(["km", "au", "kg"].contains(&i.unit.as_str()), "{}", i.unit);
        }
    }
}

#[test]
fn same_member_every_time_in_any_order() {
    let c = main_belt();
    let seed = Seed::from_text("sb1");
    let s = Settings::default();
    let a = resolve(&seed, &c, 7, &sun(), None, None, &s);
    let _ = resolve(&seed, &c, 3, &sun(), None, None, &s);
    let b = resolve(&seed, &c, 7, &sun(), None, None, &s);
    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert!(a.address.ends_with("obj:7"), "{}", a.address);
    assert_eq!(crate::member::rank_of(&a.address.parse().unwrap()), Some(7));
}

#[test]
fn mass_is_the_clouds_and_makeup_sums_to_one() {
    let c = main_belt();
    for mb in many(&c, 1..200) {
        assert!((mb.mass_kg - c.object_mass_kg(mb.rank)).abs() <= 1e-9 * mb.mass_kg);
        assert!((mb.makeup.total() - 1.0).abs() < 1e-12);
        assert!(mb.facts["radius_m"] > 0.0);
        assert!(mb.spin_hours > 0.0);
        let [a, b, c3] = mb.figure.axes_m;
        assert!(a >= b * 0.999 && b >= c3 * 0.999, "{:?}", mb.figure);
        if mb.structure.kind == "rubble-pile" {
            assert!(
                mb.spin_hours >= mb.barrier_hours * 0.99,
                "rubble cannot beat the barrier"
            );
        }
    }
}

#[test]
fn main_belt_mixes_stony_and_carbon_types_with_a_few_metal() {
    let members = many(&main_belt(), 1_000..5_000);
    let share = |l: &str| {
        members.iter().filter(|m| m.type_label == l).count() as f64 / members.len() as f64
    };
    let stony = share("S") + share("Q");
    let carbon = share("C");
    let metal = share("M");
    assert!(stony > 0.15 && carbon > 0.15, "S {stony} C {carbon}");
    assert!(metal > 0.0 && metal < 0.15, "M {metal}");
    assert!(share("V") > 0.0);
}

#[test]
fn km_sized_main_belt_bodies_are_mostly_rubble() {
    // Members ranked far down the size law are a few km across.
    let members = many(&main_belt(), 200_000..200_400);
    let d = members[0].diameter_km;
    assert!(d > 0.3 && d < 50.0, "{d}");
    let rubble = members
        .iter()
        .filter(|m| m.structure.kind == "rubble-pile")
        .count();
    assert!(rubble as f64 / members.len() as f64 > 0.5, "{rubble}");
}

#[test]
fn small_inner_bodies_pair_up_at_the_radar_shares() {
    // A near-Earth-like population: rubble piles a few hundred metres across at 1.2 AU.
    let c = cloud(
        "asteroid-belt",
        [1.0, 1.4],
        0.3,
        10.0,
        [2_000, 6_000, 1_500, 500, 0, 0],
        2.0e4,
    );
    let members = many(&c, 50_000_000..50_004_000);
    let d = members[0].diameter_km;
    assert!(d > 0.2 && d < 3.0, "{d}");
    let n = members.len() as f64;
    let contact = members.iter().filter(|m| m.pair.kind == "contact").count() as f64 / n;
    let sep = members
        .iter()
        .filter(|m| m.pair.kind == "separated")
        .count() as f64
        / n;
    // YORP has had time, and most are rubble: close to 11% and 16%.
    assert!((contact - 0.11).abs() < 0.04, "contact {contact}");
    assert!((sep - 0.16).abs() < 0.05, "separated {sep}");
}

#[test]
fn cold_kuiper_belt_makes_gentle_flattened_contact_binaries() {
    let members = many(&cold_kuiper(), 100_000..103_000);
    let gentle: Vec<&Member> = members
        .iter()
        .filter(|m| m.pair.kind == "contact" && m.pair.origin.as_deref() == Some("gentle-merger"))
        .collect();
    assert!(gentle.len() > 100, "{}", gentle.len());
    let m0 = gentle[0];
    assert_eq!(m0.figure.lobes.len(), 2);
    // Arrokoth's lobes are flattened, c/a near 0.45 to 0.65.
    let l = &m0.figure.lobes[0].axes_m;
    assert!(l[2] / l[0] < 0.6, "{l:?}");
    assert!(m0.history.iter().any(|e| e.says.contains("Arrokoth")));
    assert!(members.iter().filter(|m| m.type_label == "D").count() > members.len() / 2);
    // Little YORP out there.
    assert!(members.iter().all(|m| m.yorp_strength < 0.5));
}

#[test]
fn lobes_keep_the_volume_and_the_barycentre() {
    let members = many(&cold_kuiper(), 100_000..101_000);
    for mb in members.iter().filter(|m| m.pair.kind == "contact") {
        let v: f64 = mb
            .figure
            .lobes
            .iter()
            .map(|l| l.axes_m[0] * l.axes_m[1] * l.axes_m[2])
            .sum();
        let r = 500.0 * mb.diameter_km;
        assert!((v / (r * r * r) - 1.0).abs() < 1e-9);
        let (l1, l2) = (&mb.figure.lobes[0], &mb.figure.lobes[1]);
        let m1 = l1.axes_m.iter().product::<f64>();
        let m2 = l2.axes_m.iter().product::<f64>();
        let bary = (m1 * l1.offset_m + m2 * l2.offset_m) / (m1 + m2);
        // Lobe masses follow the size ratio cubed, so the barycentre is near zero.
        assert!(bary.abs() < 0.3 * r, "{bary} {r}");
    }
}

#[test]
fn spin_barrier_is_near_2_2_hours_for_rubble() {
    // Pravec and Harris 2000: the barrier sits near 2.2 h for bulk densities about 2,000 to 2,500.
    let p = fg::barrier_hours(2_250.0);
    assert!((p - 2.2).abs() < 0.1, "{p}");
}

#[test]
fn maclaurin_and_jacobi_match_chandrasekhar() {
    // At the bifurcation, e = 0.81267 so c/a = 0.5827.
    assert!((fg::maclaurin_q(0.812_67) - fg::Q_BIFURCATION).abs() < 2e-4);
    assert!((fg::maclaurin_c_over_a(fg::Q_BIFURCATION) - 0.5827).abs() < 1e-3);
    assert!((fg::maclaurin_q(0.929_96) - fg::Q_MACLAURIN_MAX).abs() < 2e-4);
    // Slow spin: nearly a sphere.
    assert!(fg::maclaurin_c_over_a(0.001) > 0.998);
    let (b, c) = fg::jacobi_axes(fg::Q_MACLAURIN_MAX);
    assert!((b - 0.432).abs() < 1e-9 && (c - 0.345).abs() < 1e-9);
}

#[test]
fn reference_bodies_spin_figures() {
    // Earth-like slow spin and Vesta (5.34 h, 3,456 kg/m3): small flattening,
    // Vesta's measured c/a is 0.78 but it is not fully relaxed (impact basins).
    let q_vesta = fg::q_of(5.342, 3_456.0);
    let ca = fg::maclaurin_c_over_a(q_vesta);
    assert!(ca > 0.75 && ca < 0.95, "{ca}");
    // Haumea, 3.92 h at about 2,600 kg/m3: right at the bifurcation, strongly flattened.
    let q_h = fg::q_of(3.915, 2_600.0);
    assert!(q_h > 0.3 && q_h < 0.45, "{q_h}");
}

#[test]
fn largest_bodies_are_round() {
    let mut c = main_belt();
    c.solid = [20_000_000, 60_000_000, 15_000_000, 5_000_000, 0, 0];
    let mb = &many(&c, 1..2)[0];
    assert!(mb.diameter_km > 600.0, "{}", mb.diameter_km);
    assert_eq!(mb.structure.kind, "relaxed");
    assert!(mb.figure.relief < 0.05);
}

#[test]
fn rings_resolve_to_icy_particles_with_no_pairs() {
    let mut c = cloud(
        "ring",
        [0.0005, 0.0009],
        0.0,
        0.0,
        [0, 20, 10, 9_970, 0, 0],
        1.0,
    );
    c.around = Some("w5".into());
    let seed = Seed::from_text("sb1");
    let s = Settings::default();
    let mb = resolve(&seed, &c, 1_000, &sun(), Some(9.5), None, &s);
    assert_eq!(mb.type_label, "ring");
    assert_eq!(mb.structure.kind, "ring-aggregate");
    assert_eq!(mb.pair.kind, "single");
    assert!(mb.temperature_k < 100.0);
}

#[test]
fn a_shattered_world_leaves_its_core_as_a_notable_metal_body() {
    use sse_formation::{disc, growth, hurley::Fits, leftovers, scenario, violent};
    let s = Settings::default();
    let fits = Fits::new(sse_formation::hurley::Z_SUN);
    let mut found = false;
    'outer: for text in ["f3-golden", "sb-psyche-1", "sb-psyche-2", "sb-psyche-3"] {
        let seed = Seed::from_text(text);
        for n in 0..24u64 {
            let sys = scenario::system(n);
            let dsc = disc::Disc::new(disc::disc_params(
                &seed,
                &sys,
                1.0,
                fits.l_zams(1.0),
                0.0,
                0.0,
                0.55,
                300.0,
                None,
            ));
            let g = growth::grow(&seed, &sys, &dsc);
            let u = violent::upheave(&seed, &sys, &dsc, &g);
            let l = leftovers::leftovers(&seed, &sys, &dsc, &u);
            for nb in notables(&u, &l) {
                let c = l
                    .clouds
                    .iter()
                    .find(|c| c.address.to_string() == nb.cloud)
                    .unwrap();
                let mb = resolve(&seed, c, nb.rank, &sun(), None, Some(&u), &s);
                assert_eq!(mb.structure.kind, "stripped-core");
                assert!(mb.makeup.metal > c.solid[0] as f64 / c.solid.iter().sum::<u64>() as f64);
                assert!(mb.history[0].says.contains("iron core"), "{:?}", mb.history);
                if mb.type_label == "M" {
                    found = true;
                    break 'outer;
                }
            }
        }
    }
    assert!(
        found,
        "no shattered world gave a metal core in the systems tried"
    );
}
