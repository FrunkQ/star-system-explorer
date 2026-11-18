use super::*;
use proptest::prelude::*;
use sse_core::Seed;

fn pubr() -> Publisher {
    Publisher::new("sse.base", "test", "0.1.0")
}
fn addr(n: u64) -> Address {
    Address::universe("t").unwrap().child("body", n).unwrap()
}
fn set_event(stamp: u64, at: i64, a: u64, field: &str, v: i128, causes: Vec<ContentHash>) -> Event {
    let mut set = BTreeMap::new();
    let mut f = BTreeMap::new();
    f.insert(field.to_string(), Value::Int(v));
    set.insert(addr(a), f);
    Event::new(EventBody {
        kind: kinds::AUTHORED_SET.into(),
        subject: vec![addr(a)],
        occurred_at: UniverseTime::seconds(at),
        activated_at: UniverseTime::seconds(at),
        recognised_at: SessionStamp(stamp),
        causes,
        payload: Payload {
            set,
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: pubr(),
    })
    .unwrap()
}

#[test]
fn append_rules() {
    let mut c = Chronicle::new();
    let e1 = set_event(0, 10, 1, "mass", 5, vec![]);
    let id1 = c.append(e1.clone()).unwrap();
    assert!(matches!(
        c.append(e1),
        Err(ChronicleError::Event(EventError::Duplicate(_)))
    ));
    let bad = set_event(1, 10, 1, "mass", 6, vec![ContentHash::of_bytes(b"nope")]);
    assert!(matches!(
        c.append(bad),
        Err(ChronicleError::Event(EventError::UnknownCause(_)))
    ));
    let back = set_event(0, 10, 1, "mass", 7, vec![]);
    assert!(matches!(
        c.append(back),
        Err(ChronicleError::Event(EventError::StampNotIncreasing { .. }))
    ));
    c.append(set_event(1, 5, 1, "mass", 6, vec![id1])).unwrap();
    // activation order wins over append order: value at t=10 is the t=10 edit
    assert_eq!(
        c.state_at(UniverseTime::seconds(10)).get(&addr(1), "mass"),
        Some(&Value::Int(5))
    );
    assert_eq!(
        c.state_at(UniverseTime::seconds(7)).get(&addr(1), "mass"),
        Some(&Value::Int(6))
    );
    assert_eq!(
        c.state_at(UniverseTime::seconds(1)).get(&addr(1), "mass"),
        None
    );
}

#[test]
fn stale_checkpoint_is_refused() {
    let mut c = Chronicle::new();
    c.append(set_event(0, 10, 1, "x", 1, vec![])).unwrap();
    let cp = c.checkpoint(UniverseTime::seconds(10));
    c.append(set_event(1, 5, 1, "x", 2, vec![])).unwrap(); // activates before the checkpoint
    assert_eq!(
        c.state_from(&cp, UniverseTime::seconds(20)),
        Err(ChronicleError::CheckpointStale)
    );
}

#[test]
fn commit_moves_branch_onto_main() {
    let mut c = Chronicle::new();
    let base = c.append(set_event(0, 0, 1, "x", 1, vec![])).unwrap();
    let b = c.branch("look ahead");
    let e = set_event(1, 100, 1, "x", 2, vec![base]);
    let eid = c.branch_append(&b, e).unwrap();
    c.branch_append(&b, set_event(2, 200, 2, "y", 3, vec![eid]))
        .unwrap();
    let ahead = c.branch_state_at(&b, UniverseTime::seconds(300)).unwrap();
    let commit = c
        .commit(&b, UniverseTime::seconds(0), vec![addr(1)], pubr())
        .unwrap();
    assert_eq!(c.state_at(UniverseTime::seconds(300)), ahead);
    assert_eq!(c.len(), 4);
    assert_eq!(c.events().last().unwrap().id, commit);
    assert_eq!(c.events().last().unwrap().body.causes.len(), 2);
}

#[test]
fn manifest_round_trips() {
    let mut c = Chronicle::new();
    c.append(set_event(0, 0, 1, "x", 1, vec![])).unwrap();
    let mut m = RunManifest::new(
        Seed::from_text("tarn"),
        "0.1.0",
        vec![PackRef {
            id: "sse.base-physics".into(),
            version: "0.1.0".into(),
            content_hash: ContentHash::of_bytes(b"pack"),
        }],
        c.authored_hash(),
    );
    m.checkpoints
        .push(c.checkpoint(UniverseTime::seconds(0)).state_hash);
    let bytes = sse_core::cbor::to_canonical(&m).unwrap();
    assert_eq!(
        sse_core::cbor::from_canonical::<RunManifest>(&bytes).unwrap(),
        m
    );
    let json = serde_json::to_string(&m).unwrap();
    let back: RunManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back, m);
    assert_eq!(back.id(), m.id());
}

/// Random edit scripts: (time, body, field, value).
fn script() -> impl Strategy<Value = Vec<(i64, u64, u8, i128)>> {
    proptest::collection::vec((0i64..50, 0u64..5, 0u8..3, -100i128..100), 1..40)
}

fn build(s: &[(i64, u64, u8, i128)]) -> Chronicle {
    let mut c = Chronicle::new();
    for (i, (t, a, f, v)) in s.iter().enumerate() {
        c.append(set_event(i as u64, *t, *a, &format!("f{f}"), *v, vec![]))
            .unwrap();
    }
    c
}

proptest! {
    #[test]
    fn fold_equals_checkpoint_path(s in script(), more in script(), cut in 0i64..60, at in 0i64..60) {
        let mut c = build(&s);
        let cp = c.checkpoint(UniverseTime::seconds(cut));
        prop_assert_eq!(&cp.state, &c.state_at(UniverseTime::seconds(cut)));
        // later events that activate at or after the checkpoint keep it valid
        for (i, (t, a, f, v)) in more.iter().enumerate() {
            c.append(set_event(1000 + i as u64, cut + t, *a, &format!("f{f}"), *v, vec![])).unwrap();
        }
        let at = UniverseTime::seconds(cut.max(at));
        let via = c.state_from(&cp, at).unwrap();
        prop_assert_eq!(via.hash(), c.state_at(at).hash());
    }

    #[test]
    fn branch_then_discard_leaves_state_unchanged(s in script(), extra in script()) {
        let mut c = build(&s);
        let before = c.state_at(UniverseTime::seconds(1000)).hash();
        let head = c.head();
        let b = c.branch("what if");
        for (i, (t, a, f, v)) in extra.iter().enumerate() {
            c.branch_append(&b, set_event(5000 + i as u64, *t, *a, &format!("f{f}"), *v, vec![])).unwrap();
        }
        c.discard(&b).unwrap();
        prop_assert_eq!(c.state_at(UniverseTime::seconds(1000)).hash(), before);
        prop_assert_eq!(c.head(), head);
    }

    #[test]
    fn state_hash_is_insensitive_to_map_insertion(s in script()) {
        let c = build(&s);
        let st = c.state_at(UniverseTime::seconds(1000));
        let mut rev = State::default();
        for (a, m) in st.facts.iter().rev() {
            let e = rev.facts.entry(a.clone()).or_default();
            for (k, v) in m.iter().rev() { e.insert(k.clone(), v.clone()); }
        }
        prop_assert_eq!(rev.hash(), st.hash());
    }
}

#[test]
fn golden_state_hash() {
    let s: Vec<(i64, u64, u8, i128)> = (0..200)
        .map(|i: i64| {
            (
                i % 37,
                (i % 7) as u64,
                (i % 3) as u8,
                (i * 31 % 201 - 100) as i128,
            )
        })
        .collect();
    let c = build(&s);
    assert_eq!(
        c.state_at(UniverseTime::seconds(20)).hash().hex(),
        "08a0bc876658e735cd957ddbb72afbe5b9b87dc4ce39064c01267b40c9314443"
    );
}

#[test]
fn branch_from_an_empty_chronicle() {
    // T2 found this: appending to a branch cut before any event underflowed.
    let mut c = Chronicle::new();
    let b = c.branch("look");
    let e = Event::new(EventBody {
        kind: kinds::AUTHORED_SET.into(),
        subject: vec!["u:t".parse::<sse_core::Address>().unwrap()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: c.next_stamp(),
        causes: vec![],
        payload: Payload::default(),
        evidence: Evidence::Authored,
        publisher: Publisher::new("t", "t", "1"),
    })
    .unwrap();
    c.branch_append(&b, e).unwrap();
    c.commit(
        &b,
        UniverseTime::ORIGIN,
        vec!["u:t".parse().unwrap()],
        Publisher::new("t", "t", "1"),
    )
    .unwrap();
    assert_eq!(c.len(), 2);
}
