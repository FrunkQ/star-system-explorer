//! CM0: the Sol today pack (`store-packs/sol-today`) goes into a campaign
//! through the stack like any pack. Its calendar carries an `about` line for
//! people (UTC days counted on TT), which the kernel carries and never reads;
//! before CM0 that line made the kernel refuse the whole pack.
//!
//! The one-file form a GM opens is built by `scripts/bundle-pack.mjs` and
//! checked, stacked in the browser kernel and read by Roll call in
//! `packages/ui-contract/test/pack-bundle.test.ts` and `tests/ui/rollcall.e2e.mjs`.

use serde_json::{json, Value as J};
use sse_kernel::Kernel;
use std::path::PathBuf;

fn read(rel: &str) -> J {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn ask(k: &mut Kernel, v: J) -> J {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

#[test]
fn sol_today_stacks_and_its_calendar_is_the_campaigns() {
    let pack = read("store-packs/sol-today/pack.json");
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        json!({"op": "load_stack", "packs": [read("packs/base-physics/pack.json"), pack]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let c = ask(&mut k, json!({"op": "calendar"}));
    assert_eq!(c["type"], "calendar_is", "{c}");
    let cal = &c["calendar"];
    assert_eq!(cal["name"], "Gregorian (UTC days, read on TT)");
    assert_eq!(cal["epoch_s"], 63_115_156_800_i64);
    assert_eq!(cal["months"].as_array().unwrap().len(), 12);
    assert_eq!(cal["leap"]["but_every"], 400);
    assert!(
        cal["about"].as_str().unwrap().contains("TT"),
        "the line for people is carried: {cal}"
    );
}

#[test]
fn a_calendar_without_an_about_line_reads_as_before() {
    let mut pack = read("store-packs/sol-today/pack.json");
    pack["provides"]["other"]["calendar"]
        .as_object_mut()
        .unwrap()
        .remove("about");
    let mut k = Kernel::new();
    let r = ask(&mut k, json!({"op": "load_stack", "packs": [pack]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let c = ask(&mut k, json!({"op": "calendar"}));
    assert!(c["calendar"].get("about").is_none(), "{c}");
}
