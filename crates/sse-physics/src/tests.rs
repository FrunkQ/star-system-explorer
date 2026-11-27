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
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        with(&p, c, || panic!("boom"))
    }));
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
    assert!(p.blocks().iter().next().is_some());
}
