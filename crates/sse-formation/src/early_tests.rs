//! F6 evidence: early worlds, and the hand-off to the surface, climate and life suites.
use crate::early::*;
use crate::hurley::Fits;
use crate::lifeline::LifeLine;
use crate::scenario::world;
use std::collections::BTreeMap as Map;

fn sun() -> (Fits, LifeLine) {
    let f = Fits::new(0.02);
    let l = LifeLine::new(&f, 1.0).unwrap();
    (f, l)
}

#[test]
fn earth_starts_like_earth() {
    let (f, l) = sun();
    // Earth: about 0.1 percent water by mass all told (oceans 0.02 percent, plus
    // mantle), formed by about 50 Myr.
    let w = early_life(&world(3, 1.0, 1.0, 0.001, 0.0), &f, &l);
    assert!(
        (0.28..0.36).contains(&w.core_mass_fraction),
        "CMF {}",
        w.core_mass_fraction
    );
    assert!((0.97..1.03).contains(&w.radius), "radius {}", w.radius);
    assert_eq!(w.envelope, Envelope::None);
    assert!(!w.desiccated);
    // The young Sun was about 70 percent as bright (faint young Sun).
    assert!(
        (0.65..0.75).contains(&w.early_insolation),
        "S {}",
        w.early_insolation
    );
    // 0.1 percent water, 70 percent outgassed: about 3 Earth oceans at first.
    assert!(
        (2.0..4.5).contains(&w.surface_water_oceans),
        "oceans {}",
        w.surface_water_oceans
    );
    assert!(w.magma_ocean_end_myr < 60.0);
    // Outgassed CO2 of order Venus's 92 bar, which is about Earth's carbonate
    // inventory (Kasting 1993; Venus: Pioneer Venus and Venera).
    assert!((30.0..300.0).contains(&w.co2_bar), "CO2 {} bar", w.co2_bar);
}

#[test]
fn venus_inside_the_runaway_limit_dries_out() {
    let (f, l) = sun();
    let v = early_life(&world(2, 0.815, 0.723, 0.001, 0.0), &f, &l);
    assert!(v.early_insolation > 1.06, "S {}", v.early_insolation);
    assert!(v.desiccated);
    assert_eq!(v.surface_water_oceans, 0.0);
    // Hamano type II: about 100 Myr of magma ocean.
    assert!(v.magma_ocean_end_myr > 140.0);
    // Mars, small and far out: no long magma ocean, keeps its water.
    let m = early_life(&world(4, 0.107, 1.524, 0.001, 0.0), &f, &l);
    assert!(!m.desiccated);
    assert!((0.50..0.56).contains(&m.radius), "Mars radius {}", m.radius);
}

#[test]
fn photoevaporation_carves_the_radius_valley() {
    // Owen and Wu 2017: a few-Earth-mass core with a few percent hydrogen is
    // stripped close in and keeps it farther out, so sub-Neptunes sit above
    // about 2 Earth radii and stripped cores below 1.6 (Fulton et al. 2017).
    let (f, l) = sun();
    let close_in = early_life(&world(1, 5.0, 0.05, 0.0, 0.15), &f, &l);
    let far = early_life(&world(5, 5.0, 0.5, 0.0, 0.15), &f, &l);
    assert_eq!(close_in.envelope, Envelope::Stripped);
    assert!(close_in.radius < 1.7, "stripped core {}", close_in.radius);
    assert_eq!(far.envelope, Envelope::Kept);
    assert!(far.radius > 2.0, "sub-Neptune {}", far.radius);
    // Lopez and Fortney 2014 eq. 3 on a 5 Earth-mass core at 10 S_earth and 5 Gyr:
    // 1 to 5 percent hydrogen spans the sub-Neptune peak Fulton et al. 2017 see
    // (about 2 to 3.5 Earth radii, mode near 2.4).
    for fenv in [0.01, 0.02, 0.05] {
        let r = rocky_radius(5.0, 0.33) + envelope_thickness(5.0, fenv, 10.0, 5.0);
        assert!((1.9..3.5).contains(&r), "LF14 {fenv}: {r}");
    }
    // A gas-rich body is a giant.
    let g = early_life(&world(6, 10.0, 5.2, 0.3, 300.0), &f, &l);
    assert_eq!(g.envelope, Envelope::Giant);
}

#[test]
fn water_worlds_are_bigger_for_their_mass() {
    let (f, l) = sun();
    let dry = early_life(&world(1, 2.0, 1.2, 0.001, 0.0), &f, &l);
    let wet = early_life(&world(2, 2.0, 1.2, 0.5, 0.0), &f, &l);
    assert!(wet.water_fraction > 0.49);
    // Zeng et al. 2019: half water by mass is about a quarter larger.
    let ratio = wet.radius / dry.radius;
    assert!((1.15..1.35).contains(&ratio), "ratio {ratio}");
    // Water oxidises iron: the core is smaller.
    assert!(wet.core_mass_fraction < dry.core_mass_fraction);
}

#[test]
fn early_life_is_deterministic() {
    let (f, l) = sun();
    let p = world(3, 1.0, 1.0, 0.001, 0.0);
    let a = serde_json::to_string(&early_life(&p, &f, &l)).unwrap();
    let b = serde_json::to_string(&early_life(&p, &f, &l)).unwrap();
    assert_eq!(a, b);
}

fn base_stack() -> sse_pack::ResolvedStack {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/packs/base-physics.json"
    );
    let pack = sse_pack::PackManifest::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    sse_pack::resolve(&[pack], "0.1.0").unwrap()
}

#[test]
fn no_quantity_has_two_publishers() {
    // Every kind this stage hands off is published by it alone: no base-physics
    // flow publishes any of them.
    let stack = base_stack();
    let mut flow_kinds = Map::new();
    for f in stack.flows.values() {
        for p in &f.doc.publishes {
            assert!(
                flow_kinds
                    .insert(p.kind.clone(), f.doc.id.clone())
                    .is_none(),
                "{} published twice",
                p.kind
            );
        }
    }
    for (kind, _, _) in HAND_OFF_KINDS {
        assert!(
            !flow_kinds.contains_key(kind),
            "{kind} is also published by {}",
            flow_kinds[kind]
        );
    }
    let mut seen = Map::new();
    for (kind, _, _) in HAND_OFF_KINDS {
        assert!(seen.insert(kind, ()).is_none(), "{kind} declared twice");
    }
}

#[test]
fn hand_off_values_are_the_ones_the_flows_read() {
    // The facts are SI in the kinds' own dimensions, and the base flows that read
    // them give Earth back from F6's Earth.
    let stack = base_stack();
    let atoms = sse_atoms::registry();
    let cx = sse_flow::Context {
        atoms: &atoms,
        kinds: &stack.kinds,
        units: &stack.units,
    };
    let (fi, l) = sun();
    let w = early_life(&world(3, 1.0, 1.0, 0.001, 0.0), &fi, &l);
    let facts: Map<&str, f64> = w.facts.iter().map(|h| (h.kind.as_str(), h.value)).collect();
    assert_eq!(facts.len(), HAND_OFF_KINDS.len());
    let mut read = 0;
    for f in stack.flows.values() {
        let flow = sse_flow::Flow::compile(f.doc.clone(), &cx).unwrap();
        if !f
            .doc
            .inputs
            .iter()
            .all(|i| facts.contains_key(i.kind.as_str()))
        {
            continue;
        }
        let mut ins = Map::new();
        for i in &f.doc.inputs {
            let dim = stack.kinds[&i.kind].dim;
            ins.insert(
                i.name.clone(),
                sse_flow::InputValue {
                    q: sse_core::Quantity::si(facts[i.kind.as_str()], dim),
                    source: sse_flow::SourceLabel::Resolved,
                },
            );
        }
        let ev = flow
            .eval(&cx, &ins)
            .unwrap_or_else(|e| panic!("{}: {e:?}", f.doc.id));
        if let Some(g) = ev.published("body.surface_gravity") {
            assert!((9.4..10.2).contains(&g.value), "Earth gravity {}", g.value);
        }
        read += 1;
    }
    assert!(read >= 1, "no base flow reads the hand-off");
    // Every hand-off kind the kernel already knows has the unit F6 writes.
    for (kind, unit, _) in HAND_OFF_KINDS {
        if let Some(k) = stack.kinds.get(kind) {
            let q = stack
                .units
                .q(1.0, unit)
                .unwrap_or_else(|e| panic!("{kind} {unit}: {e}"));
            assert_eq!(q.dim, k.dim, "{kind}");
            assert_eq!(q.value, 1.0, "{kind} is not SI");
        }
    }
}
#[test]
fn early_worlds_golden() {
    // Six reference worlds, byte-stable across devices (tests/golden/formation-early.json).
    crate::golden::check("formation-early.json", &crate::scenario::early_golden());
}
