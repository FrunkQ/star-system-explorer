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
