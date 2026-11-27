//! L1c checks (life design sections 7 and 14).

use crate::biosphere::*;
use crate::scenario::*;
use crate::RelationKind;

fn get(n: &str) -> Biosphere {
    biospheres().into_iter().find(|(x, _)| x == n).unwrap().1
}

#[test]
fn earth_today_resolves() {
    let b = get("earth");
    for g in &b.guilds {
        println!(
            "{} [{}] share {:.3} intake {:.2e} {} / {}",
            g.id, g.grade, g.share, g.intake_gc_yr, g.physics_name, g.plain_name
        );
    }
    println!("{:?}\n{:?}", b.traits, b.diversity);
    assert!((8..=40).contains(&b.guilds.len()), "{}", b.guilds.len());
    let producers: Vec<&GuildDetail> = b.guilds.iter().filter(|g| g.role == "producer").collect();
    let share: f64 = producers.iter().map(|g| g.share).sum();
    assert!((share - 1.0).abs() < 1e-9, "{share}");
    let prod: f64 = producers.iter().map(|g| g.production_gc_yr).sum();
    assert!((prod / b.budget.production_gc_yr - 1.0).abs() < 1e-9);
    assert!(b.traits.colour_options.contains(&"green".to_string()));
    assert_eq!(b.traits.flight, "feasible");
    let land = b.traits.max_land_animal_kg.unwrap();
    assert!((5.0e4..1.0e5).contains(&land));
    assert!((120.0..140.0).contains(&b.traits.tallest_plant_m.unwrap()));
    assert!(b.traits.senses.iter().any(|s| s == "vision"));
    assert!(b.guilds.iter().any(|g| g.grade == "animal"));
    assert!(b.guilds.iter().any(|g| g.grade == "plant"));
    for k in [
        RelationKind::Ancestry,
        RelationKind::Resource,
        RelationKind::Habitat,
        RelationKind::Capability,
    ] {
        assert!(b.resolved.relations.iter().any(|r| r.kind == k), "{k:?}");
    }
}

#[test]
fn a_microbial_world_has_few_guilds() {
    let b = get("earth-archean");
    let earth = get("earth");
    assert!(
        b.guilds.len() < earth.guilds.len() && !b.guilds.is_empty(),
        "{}",
        b.guilds.len()
    );
    assert!(b.guilds.iter().all(|g| g.grade == "microbial"));
    assert_eq!(b.traits.flight, "not_evolved");
    assert_eq!(b.traits.max_land_animal_kg, None);
    assert!(
        ["purple", "brown", "olive green"].contains(&b.traits.producer_colour.as_deref().unwrap())
    );
    assert!(b.diversity.lineages < earth.diversity.lineages);
}

#[test]
fn relations_point_at_known_things() {
    let b = get("earth");
    let caps = all_capabilities();
    let ok = |x: &str| {
        b.guilds.iter().any(|g| g.id == x)
            || caps.iter().any(|c| c == x)
            || x.starts_with("lineage:")
            || b.budget.habitats.iter().any(|h| h.habitat.to_string() == x)
    };
    for r in &b.resolved.relations {
        assert!(ok(&r.from) && ok(&r.to), "{r:?}");
    }
}

#[test]
fn resolve_is_order_independent() {
    let bp = budget_params();
    let hp = history_params();
    let mut g = graph("earth");
    let caps = all_capabilities();
    let first = Default::default();
    let a = resolve(
        &g,
        &bp,
        &hp,
        &caps,
        &first,
        None,
        &physics("earth"),
        sse_core::Seed::from_text("x"),
        0,
    )
    .unwrap();
    g.habitats.reverse();
    let mut rc = caps.clone();
    rc.reverse();
    let b = resolve(
        &g,
        &bp,
        &hp,
        &rc,
        &first,
        None,
        &physics("earth"),
        sse_core::Seed::from_text("x"),
        0,
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn traits_respect_physical_limits_in_random_worlds() {
    use sse_core::{Address, Draws, Seed, Stream};
    let bp = budget_params();
    let hp = history_params();
    let graphs = [
        graph("earth"),
        graph("europa"),
        graph("mars"),
        archean_earth(),
    ];
    let all = all_capabilities();
    let d = Draws::new(
        Seed::from_text("l1c"),
        Address::universe("l1c").unwrap(),
        Stream::new("life.l1c.test.v1").unwrap(),
    );
    let first = Default::default();
    let (mut flyers, mut impossible) = (0, 0);
    for i in 0..10_000u64 {
        let u = |k: u64| d.unit(i * 16 + k);
        let lg = |k: u64, lo: f64, hi: f64| sse_core::math::pow(10.0, lo + (hi - lo) * u(k));
        let phys = WorldPhysics {
            surface_gravity_m_s2: lg(0, -0.5, 1.7),
            air_density_kg_m3: (u(1) < 0.8).then(|| lg(2, -4.0, 2.0)),
            star_teff_k: 2400.0 + 7000.0 * u(3),
            day_hours: lg(4, 0.5, 4.0),
            obliquity_deg: 90.0 * u(5),
            tidally_locked: u(6) < 0.2,
        };
        let caps: Vec<String> = all
            .iter()
            .filter(|_| true)
            .enumerate()
            .filter(|(k, _)| u(7 + (*k as u64 % 8)) < 0.7)
            .map(|(_, c)| c.clone())
            .collect();
        let g = &graphs[(u(15) * 4.0) as usize % 4];
        let b = resolve(
            g,
            &bp,
            &hp,
            &caps,
            &first,
            None,
            &phys,
            Seed::from_text(&format!("w{i}")),
            0,
        )
        .unwrap();
        let t = &b.traits;
        if t.flight == "impossible" {
            impossible += 1;
        }
        for gd in &b.guilds {
            assert!(
                gd.intake_gc_yr.is_finite()
                    && gd.production_gc_yr.is_finite()
                    && gd.share.is_finite()
            );
            if gd.can_fly {
                flyers += 1;
                assert!(t.flight == "feasible" || t.flight == "marginal", "{i}");
                assert!(
                    gd.max_mass_kg.unwrap() <= t.max_flyer_kg.unwrap() * (1.0 + 1e-12),
                    "{i}"
                );
            }
            if gd.grade == "animal" && gd.habitat_kind == "land-surface" {
                assert!(
                    gd.max_mass_kg.unwrap() <= t.max_land_animal_kg.unwrap() * (1.0 + 1e-12),
                    "{i}"
                );
            }
            if gd.role == "consumer" {
                let p: f64 = b
                    .guilds
                    .iter()
                    .filter(|x| x.role == "producer" && x.habitat_kind == gd.habitat_kind)
                    .map(|x| x.production_gc_yr)
                    .sum();
                assert!(gd.intake_gc_yr <= p * 0.1 + 1e-6, "{i}");
            }
        }
        let p: Vec<&GuildDetail> = b.guilds.iter().filter(|g| g.role == "producer").collect();
        if !p.is_empty() {
            let s: f64 = p.iter().map(|g| g.share).sum();
            assert!((s - 1.0).abs() < 1e-9, "{i}: {s}");
        }
        assert!(b.guilds.len() <= 40, "{i}: {}", b.guilds.len());
        if t.flight == "impossible" {
            assert!(t.max_flyer_kg.is_none());
        }
    }
    println!("{flyers} flying guilds, {impossible} worlds where flight is impossible");
    assert!(flyers > 100 && impossible > 100, "{flyers} {impossible}");
}

#[test]
fn biosphere_golden() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/life-biosphere.json"
    );
    let got = crate::scenario::biosphere_golden();
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, serde_json::to_string_pretty(&got).unwrap() + "\n").unwrap();
    }
    let want: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("bless once")).unwrap();
    assert_eq!(
        got, want,
        "tests/golden/life-biosphere.json moved: bless only on purpose"
    );
}

// LE3: where the energy goes (life-from-energy design section 5).

#[test]
fn shares_sum_to_the_captured_power_on_every_world() {
    for (n, b) in biospheres() {
        let captured: f64 = b
            .guilds
            .iter()
            .filter(|g| g.role == "producer")
            .map(|g| g.power_w)
            .sum();
        let used: f64 = b.guilds.iter().map(|g| g.uses_w).sum();
        let share: f64 = b.guilds.iter().map(|g| g.share_of_captured).sum();
        if captured > 0.0 {
            assert!(
                (used / captured - 1.0).abs() < 1e-9,
                "{n}: {used:e} of {captured:e}"
            );
            assert!((share - 1.0).abs() < 1e-9, "{n}: {share}");
        }
        // The producers' power is the budget's.
        assert!(
            (captured / b.budget.power_w - 1.0).abs() < 1e-9 || b.budget.power_w == 0.0,
            "{n}"
        );
        for g in &b.guilds {
            assert!(!g.diet.is_empty() || g.role == "symbiont", "{n} {}", g.id);
            if g.role == "producer" {
                assert!(g.diet[0].starts_with("gradient:"), "{}", g.id);
                assert!(g.sizes.is_none(), "producers are a gap, never a zero");
            }
        }
    }
}

#[test]
fn earth_land_grazers_weigh_about_what_earths_land_mammals_do() {
    // The design asked for Damuth's densities within a factor of ten. His
    // line (about 10^4.23 M^-0.75 per km^2, M in g) is ONE species'
    // density; this count is every warm-blooded grazer of the class, so the
    // two differ by the number of species and are not compared. The fair
    // check is mass: the class's standing mass against Earth's land mammals
    // today, about 0.17 Gt C (livestock 0.1, people 0.06, wild 0.007; Bar-On,
    // Phillips and Milo 2018, unverified), at about 15% carbon by wet mass.
    let b = get("earth");
    let grazers = b
        .guilds
        .iter()
        .find(|g| g.id == "consumer.2.land-surface")
        .expect("land grazers");
    let sizes = grazers.sizes.as_ref().unwrap();
    for c in sizes {
        println!(
            "{} {:.0e}-{:.0e} kg {} {:.2e} W {:.2e}",
            c.class, c.from_kg, c.to_kg, c.rate, c.power_w, c.count
        );
        assert!(c.count.is_finite() && c.count > 0.0, "{c:?}");
    }
    let warm = sizes
        .iter()
        .find(|c| c.class == "animal-sized" && c.rate == "endotherm")
        .expect("warm-blooded animal-sized grazers");
    // Numbers fall as M^-3/4 over ln M, so the mass in [a, b] is
    // N / (a^-3/4 - b^-3/4) x 3 x (b^1/4 - a^1/4).
    let (a, z) = (warm.from_kg, warm.to_kg);
    let kg = warm.count / (sse_core::math::pow(a, -0.75) - sse_core::math::pow(z, -0.75))
        * 3.0
        * (sse_core::math::pow(z, 0.25) - sse_core::math::pow(a, 0.25));
    let gt_c = kg * 1000.0 * 0.15 / 1.0e15;
    println!("warm-blooded grazers {:.2e}, {gt_c:.3} Gt C", warm.count);
    assert!((0.017..1.7).contains(&gt_c), "{gt_c} Gt C");
}
