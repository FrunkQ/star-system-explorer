//! ER2: the block door. A block edit in a pack reaches every place the
//! block runs; it says what it replaces; a bad one is refused with a reason.

use super::*;
use sse_flow::Atom;
use sse_pack::{BlockEdit, BlockReplaces};

fn base() -> PackManifest {
    let path = format!(
        "{}/../../packs/base-physics/pack.json",
        env!("CARGO_MANIFEST_DIR")
    );
    PackManifest::from_json(&std::fs::read_to_string(&path).unwrap()).expect("pack")
}

fn fixture() -> PackManifest {
    let path = format!(
        "{}/../../tests/fixtures/packs/block-edit-wind-and-heat.json",
        env!("CARGO_MANIFEST_DIR")
    );
    PackManifest::from_json(&std::fs::read_to_string(&path).unwrap()).expect("fixture")
}

fn edit(pack: &str, id: &str, settings: &[(&str, f64)], from: &str, range: &str) -> PackManifest {
    let mut p = fixture();
    p.id = pack.into();
    p.provides.blocks = vec![BlockEdit {
        id: id.into(),
        settings: settings.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        replaces: BlockReplaces {
            pack: from.into(),
            range: range.into(),
        },
    }];
    p
}

fn refused(packs: &[PackManifest]) -> String {
    let e = Blocks::from_packs(packs).expect_err("refused");
    e.iter()
        .map(|(p, m)| format!("{p}: {m}"))
        .collect::<Vec<_>>()
        .join("\n")
}

const WIND: &str = "sse.light.stellar_wind";
const AT_1AU: f64 = 1.495978707e11;

#[test]
fn an_edited_setting_runs_everywhere_the_block_runs() {
    let shipped = Blocks::shipped();
    let edited = Blocks::from_packs(&[base(), fixture()]).expect("loads");
    let before = shipped.run(WIND, &[AT_1AU]).unwrap();
    let after = edited.run(WIND, &[AT_1AU]).unwrap();
    // The wind's speed is the second output; the edit doubles it exactly.
    assert_eq!(before[1], 440.0e3);
    assert_eq!(after[1], 880.0e3);
    // Its face carries the value in force, so a flow compiled against the
    // registry starts from it.
    let face = edited.registry();
    let d = face.get(WIND).unwrap().decl();
    assert_eq!(d.param("speed_km_s").unwrap().1.default, 880.0);
    // A value passed on the call (a node's own setting) still wins.
    let p: Vec<f64> = shipped
        .get(WIND)
        .unwrap()
        .decl()
        .params
        .iter()
        .map(|q| q.default)
        .collect();
    let node = edited.get(WIND).unwrap().eval(&[AT_1AU], &p).unwrap();
    assert_eq!(node, before);
    // The edits are listed with their pack, the shipped value and the new one.
    assert_eq!(
        edited.edits().iter().find(|e| e.block == WIND),
        Some(&SettingEdit {
            block: WIND.into(),
            setting: "speed_km_s".into(),
            shipped: 440.0,
            value: 880.0,
            pack: "test.block-edit".into(),
        })
    );
    assert!(shipped.edits().is_empty());
}

#[test]
fn an_unedited_block_gives_the_same_bits_as_the_shipped_one() {
    let edited = Blocks::from_packs(&[base(), fixture()]).expect("loads");
    for a in sse_atoms::formulas::loaded() {
        let d = a.decl();
        if d.id == WIND || d.id == "sse.heat.radiogenic_heating" || d.inputs.len() != 1 {
            continue;
        }
        let p: Vec<f64> = d.params.iter().map(|q| q.default).collect();
        let want = a.eval(&[1.0], &p);
        let got = edited.get(&d.id).unwrap().eval(&[1.0], &[]);
        assert_eq!(
            format!("{want:?}"),
            format!("{got:?}"),
            "{} differs through the door",
            d.id
        );
    }
}

#[test]
fn the_door_follows_the_physics_in_force() {
    let packs = [base(), fixture()];
    let resolved = sse_pack::resolve(&packs, "0.1.0").expect("resolves");
    let p = Arc::new(Physics::build(
        &packs,
        Some(&resolved),
        Arc::new(Reference::default()),
        Arc::new(BTreeMap::new()),
    ));
    let outside = run(WIND, &[AT_1AU]).unwrap()[1];
    let inside = with(&p, p.constants(), || run(WIND, &[AT_1AU]).unwrap()[1]);
    assert_eq!((outside, inside), (440.0e3, 880.0e3));
    let face = with(&p, p.constants(), || {
        block(WIND)
            .unwrap()
            .decl()
            .param("speed_km_s")
            .unwrap()
            .1
            .default
    });
    assert_eq!(face, 880.0);
    // A block edit is physics: it moves the physics hash.
    let plain = Physics::build(
        &packs[..1],
        Some(&sse_pack::resolve(&packs[..1], "0.1.0").unwrap()),
        Arc::new(Reference::default()),
        Arc::new(BTreeMap::new()),
    );
    assert_ne!(p.hash(), plain.hash());
}

#[test]
fn a_kept_value_lives_and_dies_with_its_physics() {
    let packs = [base(), fixture()];
    let p = Arc::new(Physics::build(
        &packs,
        None,
        Arc::new(Reference::default()),
        Arc::new(BTreeMap::new()),
    ));
    let speed = || run(WIND, &[AT_1AU]).unwrap()[1];
    let shipped = memo("test.wind", speed);
    let edited = with(&p, p.constants(), || memo("test.wind", speed));
    let again = with(&p, p.constants(), || memo("test.wind", || 0.0));
    assert_eq!((*shipped, *edited, *again), (440.0e3, 880.0e3, 880.0e3));
    // A new physics, even one built from the same packs, starts empty.
    let q = Arc::new(Physics::build(
        &packs,
        None,
        Arc::new(Reference::default()),
        Arc::new(BTreeMap::new()),
    ));
    assert_eq!(*with(&q, q.constants(), || memo("test.wind", || 1.0)), 1.0);
}

#[test]
fn a_pack_without_block_edits_writes_none() {
    let v = serde_json::to_value(base()).unwrap();
    assert!(v["provides"].get("blocks").is_none());
}

#[test]
fn bad_block_edits_are_refused_with_the_reason() {
    let b = base();
    let e = refused(&[
        b.clone(),
        edit("t.a", "sse.no.such", &[], SHIPPED_BY, "^0.1"),
    ]);
    assert!(
        e.contains("t.a: edits block sse.no.such, but no block has that id"),
        "{e}"
    );
    let e = refused(&[
        b.clone(),
        edit("t.a", WIND, &[("speed", 1.0)], SHIPPED_BY, "^0.1"),
    ]);
    assert!(
        e.contains("no setting \"speed\" (its settings are n_1au, speed_km_s"),
        "{e}"
    );
    let e = refused(&[
        b.clone(),
        edit("t.a", WIND, &[("speed_km_s", 1.0)], SHIPPED_BY, "^0.1"),
    ]);
    assert!(
        e.contains("speed_km_s 1 is outside its range 10 to 5000"),
        "{e}"
    );
    let e = refused(&[
        b.clone(),
        edit("t.a", WIND, &[("speed_km_s", 500.0)], SHIPPED_BY, "^2"),
    ]);
    assert!(
        e.contains("from sse.base-physics ^2, but the stack has 0.1.0"),
        "{e}"
    );
    let e = refused(&[edit(
        "t.a",
        WIND,
        &[("speed_km_s", 500.0)],
        SHIPPED_BY,
        "^0.1",
    )]);
    assert!(
        e.contains("from sse.base-physics, which is not lower in the stack"),
        "{e}"
    );
    let e = refused(&[
        b.clone(),
        edit("t.a", WIND, &[("speed_km_s", 500.0)], "t.other", "^0.1"),
    ]);
    assert!(
        e.contains(
            "edits block sse.light.stellar_wind from t.other, but it comes from sse.base-physics"
        ),
        "{e}"
    );
    // A second edit of the same block names the first; naming the shipped
    // block again is refused, so nothing is overridden silently.
    let first = edit("t.a", WIND, &[("speed_km_s", 500.0)], SHIPPED_BY, "^0.1");
    let silent = edit("t.b", WIND, &[("n_1au", 9.0)], SHIPPED_BY, "^0.1");
    let e = refused(&[b.clone(), first.clone(), silent]);
    assert!(
        e.contains(
            "t.b: edits block sse.light.stellar_wind from sse.base-physics, but it comes from t.a"
        ),
        "{e}"
    );
    // Named properly, the second layers on the first.
    let second = edit("t.b", WIND, &[("n_1au", 9.0)], "t.a", "^0.1");
    let both = Blocks::from_packs(&[b, first, second]).expect("loads");
    let d = both.get(WIND).unwrap().decl().clone();
    assert_eq!(d.param("speed_km_s").unwrap().1.default, 500.0);
    assert_eq!(d.param("n_1au").unwrap().1.default, 9.0);
    let by: Vec<&str> = both.edits().iter().map(|e| e.pack.as_str()).collect();
    assert_eq!(by, ["t.b", "t.a"]);
}
