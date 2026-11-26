use super::*;

fn pack(name: &str) -> PackManifest {
    let path = format!(
        "{}/../../packs/{name}/pack.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    PackManifest::from_json(&text).expect("pack")
}

fn physics_of(names: &[&str]) -> Physics {
    let packs: Vec<PackManifest> = names.iter().map(|n| pack(n)).collect();
    let resolved = sse_pack::resolve(&packs, "0.1.0").expect("stack resolves");
    let sheet = Arc::new(Reference::from_packs(&packs).expect("sheet"));
    Physics::build(&packs, Some(&resolved), sheet, Arc::new(BTreeMap::new()))
}

const BASE: &[&str] = &["base-reference", "base-physics", "base-surface"];

#[test]
fn a_stack_that_differs_only_in_names_has_the_same_physics_hash() {
    let a = physics_of(BASE);
    let mut with_names = BASE.to_vec();
    with_names.push("base-names");
    let b = physics_of(&with_names);
    assert_eq!(a.hash(), b.hash());
    // The names block is still there to read; it is only left out of the hash.
    assert!(b.settings("names").is_some());
    assert!(a.settings("names").is_none());
}

#[test]
fn a_changed_sheet_value_or_setting_changes_the_physics_hash() {
    let base = physics_of(BASE);
    let mut packs: Vec<PackManifest> = BASE.iter().map(|n| pack(n)).collect();
    // A setting: one number in the surface block.
    let mut edited = packs.clone();
    let s = edited[2]
        .provides
        .other
        .get_mut("surface")
        .expect("surface block");
    let sse_core::cbor::Value::Map(m) = s else {
        panic!("surface block is a map")
    };
    m.insert("er1_probe".into(), sse_core::cbor::Value::Float(1.0));
    let r = sse_pack::resolve(&edited, "0.1.0").expect("resolves");
    let sheet = Arc::new(Reference::from_packs(&edited).expect("sheet"));
    let p = Physics::build(&edited, Some(&r), sheet, Arc::new(BTreeMap::new()));
    assert_ne!(p.hash(), base.hash());
    // A sheet value: the Wien constant, nudged.
    let consts = packs[0]
        .provides
        .reference
        .get_mut("constants")
        .expect("constants");
    use sse_core::cbor::Value as V;
    let V::Array(rows) = consts else {
        panic!("constants are rows")
    };
    for row in rows {
        let V::Map(row) = row else { continue };
        if row.get("id") != Some(&V::Text("b_wien".into())) {
            continue;
        }
        let Some(V::Map(value)) = row.get_mut("value") else {
            panic!("b_wien has a value")
        };
        let v = match value.get("v") {
            Some(V::Float(v)) => *v,
            other => panic!("b_wien value {other:?}"),
        };
        value.insert("v".into(), V::Float(1.001 * v));
    }
    let r = sse_pack::resolve(&packs, "0.1.0").expect("resolves");
    let sheet = Arc::new(Reference::from_packs(&packs).expect("sheet"));
    let p = Physics::build(&packs, Some(&r), sheet, Arc::new(BTreeMap::new()));
    assert_ne!(p.hash(), base.hash());
}

#[test]
fn the_hash_is_the_same_every_time() {
    assert_eq!(physics_of(BASE).hash(), physics_of(BASE).hash());
}

#[test]
fn with_puts_the_physics_and_its_two_views_in_force_and_restores_them() {
    let p = Arc::new(physics_of(BASE));
    assert!(now().is_none());
    assert!(sse_flow::sheet::now().is_none());
    let mut c = Constants::REAL;
    c.g *= 2.0;
    let (h, g, has_sheet) = with(&p, c, || {
        (
            now().map(|n| n.hash()),
            sse_core::constants::g(),
            sse_flow::sheet::now().is_some(),
        )
    });
    assert_eq!(h, Some(p.hash()));
    let (g_row, surface) = with(&p, c, || (constant("G"), settings("surface")));
    assert_eq!(g_row, Some(sse_core::sheet::G));
    assert!(surface.is_some());
    assert_eq!(constant("G"), None);
    assert_eq!(g, 2.0 * sse_core::sheet::G);
    assert!(has_sheet);
    assert!(now().is_none());
    assert_eq!(sse_core::constants::g(), sse_core::sheet::G);
    let r = std::panic::catch_unwind(|| with(&p, c, || panic!("boom")));
    assert!(r.is_err());
    assert!(now().is_none());
    assert!(sse_flow::sheet::now().is_none());
    assert_eq!(sse_core::constants::g(), sse_core::sheet::G);
}

#[test]
fn bare_physics_has_no_sheet_door_and_no_settings() {
    let p = Physics::bare(Arc::new(Reference::default()));
    assert!(p.sheet_door().is_none());
    assert_eq!(p.settings_keys().count(), 0);
    assert!(p.flows().is_empty());
    assert!(!p.blocks().is_empty());
}

mod table_tests {
    use crate::table::*;

    fn axis1(n: usize) -> Axis {
        Axis::new("x", "1", Spacing::Log, 1.0, 100.0, n, "x")
    }

    fn baked(n: usize, bound: f64) -> Baked {
        Baked {
            axes: vec![
                axis1(n),
                Axis::new("y", "1", Spacing::Even, -1.0, 1.0, 9, "y"),
            ],
            channels: vec![
                Channel::new("a", "1", "test", "x plus y"),
                Channel::new("b", "1", "test", "x squared"),
            ],
            exact: Box::new(|x: &[f64]| Ok(vec![x[0] + x[1], x[0] * x[0]])),
            bound: vec![bound, bound],
            floor: vec![1e-6, 1e-6],
            reads: vec![],
            why: vec![],
            stand_ins: vec![],
        }
    }

    #[test]
    fn axes_put_their_ends_on_nodes_and_index_back() {
        let a = axis1(5);
        assert_eq!(a.at(0), 1.0);
        assert_eq!(a.at(4), 100.0);
        assert!((a.at(2) - 10.0).abs() < 1e-12);
        assert!((a.index_of(10.0) - 2.0).abs() < 1e-12);
        assert_eq!(a.index_of(0.5), 0.0, "held to the first node");
        assert_eq!(a.index_of(1e9), 4.0, "held to the last node");
        assert!((a.index_of(a.at_index(2.5)) - 2.5).abs() < 1e-9);
    }

    #[test]
    fn a_sealed_table_is_exact_at_nodes_and_linear_in_its_axes() {
        let t = Table::seal(
            "test",
            serde_json::json!({}),
            Size::Medium,
            None,
            "p".into(),
            baked(33, 0.05),
        )
        .expect("bakes");
        assert_eq!(t.format, FORMAT);
        assert_eq!(t.values.len(), 33 * 9 * 2);
        // At nodes, exactly the f32 of the exact value.
        let x = [t.axes[0].at(7), t.axes[1].at(3)];
        let s = t.sample(&x);
        assert_eq!(s[0], (x[0] + x[1]) as f32 as f64);
        // y is an even axis and channel a is linear in y: exact between nodes.
        let y = 0.123;
        let s = t.sample(&[t.axes[0].at(7), y]);
        assert!((s[0] - (t.axes[0].at(7) + y)).abs() < 1e-5);
        assert!(t.sampling.within, "{:?}", t.sampling);
        assert!(t.sampling.probes > 0);
        assert_eq!(t.hash.len(), 64);
    }

    #[test]
    fn sampling_goes_red_when_the_table_is_too_coarse() {
        // x squared on a log axis of 3 nodes over two decades samples badly.
        let t = Table::seal(
            "test",
            serde_json::json!({}),
            Size::Small,
            None,
            "p".into(),
            baked(3, 0.01),
        )
        .expect("bakes");
        assert!(!t.sampling.within, "{:?}", t.sampling);
        assert!(t.sampling.worst[1] > 0.1, "{:?}", t.sampling.worst);
        // And green on a fine one.
        let t = Table::seal(
            "test",
            serde_json::json!({}),
            Size::Large,
            None,
            "p".into(),
            baked(257, 0.01),
        )
        .expect("bakes");
        assert!(t.sampling.within, "{:?}", t.sampling);
    }

    #[test]
    fn tiers_pick_sizes() {
        assert_eq!(Size::of_tier(Some("gpu-basic")), Size::Small);
        assert_eq!(Size::of_tier(Some("canvas")), Size::Small);
        assert_eq!(Size::of_tier(Some("gpu-high")), Size::Medium);
        assert_eq!(Size::of_tier(None), Size::Medium);
        assert_eq!(Size::of_tier(Some("gpu-ultra")), Size::Large);
        assert_eq!(Size::of_tier(Some("still")), Size::Large);
        assert_eq!(Size::of_tier(Some("basic")), Size::Small);
        assert_eq!(Size::of_tier(Some("standard")), Size::Medium);
        assert_eq!(Size::of_tier(Some("capable")), Size::Large);
    }

    #[test]
    fn a_table_refuses_bad_axes_and_values() {
        let mut b = baked(9, 0.1);
        b.axes[0] = Axis::new("x", "1", Spacing::Log, 0.0, 1.0, 9, "x");
        assert!(Table::seal("t", serde_json::json!({}), Size::Small, None, "p".into(), b).is_err());
        let mut b = baked(9, 0.1);
        b.exact = Box::new(|_| Ok(vec![f64::NAN, 1.0]));
        assert!(Table::seal("t", serde_json::json!({}), Size::Small, None, "p".into(), b).is_err());
    }
}
