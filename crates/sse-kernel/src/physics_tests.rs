//! ER1: the physics in force is built when a stack loads, and its hash
//! covers only what can change a worked-out number.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn pack(p: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
}

/// Load `packs` into a fresh kernel; the stack hash and the physics hash.
fn load(packs: &[&str]) -> (String, String) {
    let mut k = crate::Kernel::new();
    let packs: Vec<serde_json::Value> = packs.iter().map(|p| pack(p)).collect();
    let r: serde_json::Value = serde_json::from_str(
        &k.handle_json(&serde_json::json!({"op": "load_stack", "packs": packs}).to_string()),
    )
    .unwrap();
    assert_eq!(r["type"], "stack_loaded", "{r}");
    (
        r["stack_hash"].as_str().unwrap().to_string(),
        k.physics().hash().hex(),
    )
}

const BASE: [&str; 2] = [
    "tests/fixtures/packs/base-physics.json",
    "packs/base-reference/pack.json",
];

#[test]
fn a_names_pack_changes_the_stack_hash_but_not_the_physics_hash() {
    let (stack, physics) = load(&BASE);
    let (stack_n, physics_n) = load(&[BASE[0], BASE[1], "packs/base-names/pack.json"]);
    assert_ne!(stack, stack_n);
    assert_eq!(physics, physics_n);
}

#[test]
fn a_sheet_edit_changes_the_physics_hash() {
    let (_, physics) = load(&["packs/base-physics/pack.json"]);
    let (_, edited) = load(&[
        "packs/base-physics/pack.json",
        "tests/fixtures/packs/reference-strong-gravity.json",
    ]);
    assert_ne!(physics, edited);
}

#[test]
fn the_physics_hash_is_the_same_on_every_load() {
    assert_eq!(load(&BASE).1, load(&BASE).1);
    // Before any stack the kernel has the bare physics, not a stack's.
    assert_ne!(crate::Kernel::new().physics().hash().hex(), load(&BASE).1);
}
