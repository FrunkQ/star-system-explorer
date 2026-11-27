//! ER1: the physics in force is built when a stack loads, and its hash
//! covers only what can change a worked-out number. ER2: a block edit in a
//! pack reaches the ops behind the blocks it edits.

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

/// ER2: a fresh kernel with `packs`, a universe and one adopted system; the
/// field read of that system, and the adopt answer.
fn field_read_with(packs: &[&str]) -> (serde_json::Value, serde_json::Value) {
    let mut k = crate::Kernel::new();
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let packs: Vec<serde_json::Value> = packs.iter().map(|p| pack(p)).collect();
    let r = send(serde_json::json!({"op": "load_stack", "packs": packs}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}));
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    let adopted = send(serde_json::json!({"op": "adopt", "formation": form}));
    assert_eq!(adopted["type"], "adopted", "{adopted}");
    let r = send(serde_json::json!({"op": "read_field", "system": "u:home/sys:5"}));
    assert_eq!(r["type"], "field_read", "{r}");
    (r, adopted)
}

const WITH_PHYSICS: [&str; 3] = [
    "packs/base-time/pack.json",
    "packs/base-reference/pack.json",
    "packs/base-physics/pack.json",
];

#[test]
fn a_block_edit_in_a_pack_reaches_the_ops_behind_it() {
    let (base, base_adopt) = field_read_with(&WITH_PHYSICS);
    let mut edited = WITH_PHYSICS.to_vec();
    edited.push("tests/fixtures/packs/block-edit-wind-and-heat.json");
    let (fast, fast_adopt) = field_read_with(&edited);
    // The field read stands on the stellar wind block: doubling the wind's
    // speed quadruples its pressure, and a field-held bubble's nose shrinks
    // as the pressure to the minus one sixth, 2^(-1/3) (Chapman and Ferraro).
    let (b, f) = (
        base["bodies"].as_array().unwrap(),
        fast["bodies"].as_array().unwrap(),
    );
    assert_eq!(b.len(), f.len());
    let mut held = 0;
    for (x, y) in b.iter().zip(f) {
        let (gx, gy) = (&x["glance"], &y["glance"]);
        if gx["held_off_by"] == "field" && gy["held_off_by"] == "field" {
            let ratio = gy["bubble_m"].as_f64().unwrap() / gx["bubble_m"].as_f64().unwrap();
            assert!((ratio - 0.793_700_525_984_099_7).abs() < 1e-12, "{ratio}");
            held += 1;
        }
    }
    assert!(held > 0, "no field-held world to check");
    // What stands on no edited block is bit-identical: the adopted system.
    assert_eq!(base_adopt, fast_adopt);
}

#[test]
fn a_bad_block_edit_refuses_the_stack_and_says_why() {
    let mut k = crate::Kernel::new();
    let mut bad = pack("tests/fixtures/packs/block-edit-wind-and-heat.json");
    bad["provides"]["blocks"][0]["settings"] = serde_json::json!({"speed_km_s": 1.0});
    let packs = vec![pack(WITH_PHYSICS[2]), bad];
    let r: serde_json::Value = serde_json::from_str(
        &k.handle_json(&serde_json::json!({"op": "load_stack", "packs": packs}).to_string()),
    )
    .unwrap();
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(
        r["report"].as_str().unwrap().contains(
            "edits block sse.light.stellar_wind: speed_km_s 1 is outside its range 10 to 5000"
        ),
        "{r}"
    );
    // The kernel keeps the physics it had.
    assert_eq!(
        k.physics().hash().hex(),
        crate::Kernel::new().physics().hash().hex()
    );
}
