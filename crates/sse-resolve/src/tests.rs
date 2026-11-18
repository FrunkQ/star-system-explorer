//! K7 acceptance: order independence, budget closure with pins, temporal
//! coherence, persistence, "not found within budget", golden digest.

use crate::query::{
    commit_pick, location_first, outcome_first, search_children, Criterion, FieldOutlook,
    QueryOutcome, Wish,
};
use crate::*;
use proptest::prelude::*;
use sse_chronicle::{Chronicle, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{ContentHash, Seed};
use std::collections::BTreeMap;

fn fixture() -> Envelope {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/resolve/patch.json"
    ))
    .unwrap();
    serde_json::from_str(&s).unwrap()
}
fn seed() -> Seed {
    Seed::from_text("k7")
}
fn publisher() -> Publisher {
    Publisher::new("test.resolve", "patch", "1")
}
fn int(facts: &BTreeMap<String, Value>, k: &str) -> u64 {
    match facts.get(k) {
        Some(Value::Int(v)) => *v as u64,
        other => panic!("{k}: {other:?}"),
    }
}
fn refs(env: &Envelope) -> Vec<ChildRef> {
    let k = children(&seed(), env).unwrap();
    (0..k.pins.len())
        .map(ChildRef::Pinned)
        .chain((0..k.generated).map(ChildRef::Generated))
        .collect()
}

/// Resolve the patch and every system in it, in the given order of
/// (system, then optionally its bodies), recording into a fresh chronicle.
fn resolve_tree(order: &[usize], deep_first: bool) -> Chronicle {
    let env = fixture();
    let rs = refs(&env);
    let mut c = Chronicle::new();
    for &i in order {
        let r = &rs[i % rs.len()];
        if deep_first {
            // Zoom straight to the bodies: the system's cloud from an unrecorded resolve.
            let sys = resolve(&seed(), &env, r).unwrap().envelope.unwrap();
            for b in refs(&sys) {
                resolve_recorded(&mut c, &seed(), &sys, &b, &publisher()).unwrap();
            }
            resolve_recorded(&mut c, &seed(), &env, r, &publisher()).unwrap();
        } else {
            let rec = resolve_recorded(&mut c, &seed(), &env, r, &publisher()).unwrap();
            let sys = rec.envelope.unwrap();
            for b in refs(&sys) {
                resolve_recorded(&mut c, &seed(), &sys, &b, &publisher()).unwrap();
            }
        }
    }
    c
}

#[test]
fn fixture_is_valid_and_has_children() {
    let env = fixture();
    env.validate().unwrap();
    let k = children(&seed(), &env).unwrap();
    assert_eq!(k.pins.len(), 2);
    assert!(k.generated > 3, "{}", k.generated);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// H12 experiment 2: any order, any zoom path, identical facts.
    #[test]
    fn order_and_zoom_path_independent(perm in Just((0..40usize).collect::<Vec<_>>()).prop_shuffle(), deep in any::<bool>()) {
        let n = refs(&fixture()).len();
        let forward: Vec<usize> = (0..n).collect();
        let shuffled: Vec<usize> = perm.into_iter().filter(|i| *i < n).collect();
        let a = resolve_tree(&forward, false);
        let b = resolve_tree(&shuffled, deep);
        let t = UniverseTime::years(10_000_000_000);
        prop_assert_eq!(a.state_at(t).hash(), b.state_at(t).hash());
        prop_assert_eq!(a.len(), b.len());
    }
}

#[test]
fn single_resolve_equals_resolve_all() {
    let env = fixture();
    let all = resolve_all(&seed(), &env).unwrap();
    for (r, one) in refs(&env).iter().zip(&all) {
        let single = resolve(&seed(), &env, r).unwrap();
        assert_eq!(single.facts, one.facts);
        assert_eq!(single.envelope, one.envelope);
    }
}

/// H12 experiment 3: totals equal parent budgets exactly, with pins, at
/// every level.
#[test]
fn budgets_close_with_pins_at_every_level() {
    let env = fixture();
    let all = resolve_all(&seed(), &env).unwrap();
    let k = children(&seed(), &env).unwrap();
    for (id, b) in &env.budgets {
        let sum: u128 = all
            .iter()
            .map(|r| int(&r.facts, &format!("budget.{id}")) as u128)
            .sum();
        assert_eq!(
            sum + k.splits[id].residue as u128,
            b.amount as u128,
            "budget {id}"
        );
    }
    for sys in all.iter().filter_map(|r| r.envelope.as_ref()) {
        let bodies = resolve_all(&seed(), sys).unwrap();
        let kk = children(&seed(), sys).unwrap();
        for (id, b) in &sys.budgets {
            let sum: u128 = bodies
                .iter()
                .map(|r| int(&r.facts, &format!("budget.{id}")) as u128)
                .sum();
            assert_eq!(
                sum + kk.splits[id].residue as u128,
                b.amount as u128,
                "{} budget {id}",
                sys.address
            );
        }
    }
}

proptest! {
    #[test]
    fn closure_holds_for_any_cloud(
        amount in 0u64..u64::MAX / 2,
        n in 0u64..300,
        claims in proptest::collection::vec(0u64..u64::MAX / 8, 0..4),
        tree in any::<bool>(),
        jitter in 0.0f64..1.0,
    ) {
        let mut env = fixture();
        env.count = CountSpec::Fixed { n };
        env.pins_count_toward = false;
        env.child = None;
        env.partition = if tree { PartitionSpec::Tree { jitter } } else { PartitionSpec::Flat { weights: Weights::Exponential } };
        env.budgets.retain(|k, _| k == "gas");
        env.budgets.get_mut("gas").unwrap().amount = amount;
        env.pins = claims.iter().enumerate().map(|(i, c)| Pin {
            address: env.address.authored_child("sys", i as u64).unwrap(),
            claims: [("gas".to_string(), *c)].into(),
            facts: BTreeMap::new(),
            evidence: sse_chronicle::Evidence::Authored,
        }).collect();
        let all = resolve_all(&seed(), &env).unwrap();
        let k = children(&seed(), &env).unwrap();
        let s = &k.splits["gas"];
        let sum: u128 = all.iter().map(|r| int(&r.facts, "budget.gas") as u128).sum();
        let pinned: u128 = claims.iter().map(|c| *c as u128).sum();
        if pinned <= amount as u128 {
            prop_assert_eq!(sum + s.residue as u128, amount as u128);
            prop_assert_eq!(s.deficit, 0);
        } else {
            // Impossible pins are honoured and marked, never trimmed silently.
            prop_assert_eq!(sum, pinned);
            prop_assert_eq!(s.deficit as u128, pinned - amount as u128);
            for r in all.iter().filter(|r| matches!(r.child, ChildRef::Pinned(_))) {
                prop_assert!(r.facts.contains_key("resolve.over_budget"));
            }
        }
    }
}

/// Pins keep their measured facts; the rest is filled and listed.
#[test]
fn pins_keep_measured_facts_and_list_fill() {
    let env = fixture();
    let cat = resolve(&seed(), &env, &ChildRef::Pinned(0)).unwrap();
    assert_eq!(cat.facts["metallicity"], Value::Float(0.12));
    assert_eq!(
        cat.facts["resolve.evidence"],
        Value::Text("catalogued".into())
    );
    let fill: Vec<Value> = match &cat.facts["resolve.fill"] {
        Value::Array(v) => v.clone(),
        _ => panic!(),
    };
    assert_eq!(
        fill,
        vec![
            Value::Text("multiplicity".into()),
            Value::Text("life".into())
        ]
    );
    assert_eq!(int(&cat.facts, "budget.gas"), 2_000_000_000_000);
    // A pin's child cloud gets the pin's claims as its budgets.
    assert_eq!(
        cat.envelope.unwrap().budgets["gas"].amount,
        2_000_000_000_000
    );
}

/// H6.1 rule 6 / H12 experiment 4 (identity half): a resolved body read at 20
/// times in random order is bit-identical; only the phase changes.
#[test]
fn life_line_reads_are_coherent() {
    let env = fixture();
    let mut c = Chronicle::new();
    let r = resolve_recorded(&mut c, &seed(), &env, &ChildRef::Generated(1), &publisher()).unwrap();
    let addr = env.address.child("sys", 1).unwrap();
    let born = int(&r.facts, "life.born_s") as i64;
    let ends = int(&r.facts, "life.ends_s") as i64;
    assert!(ends >= born);
    let times: Vec<i64> = (0..20)
        .map(|k| ((k * 7919) % 20) as i64 * 50_000_000_000_000_000)
        .collect();
    let reference = c.state_at(UniverseTime::seconds(0)).facts[&addr].clone();
    for t in times {
        let st = c.state_at(UniverseTime::seconds(t));
        assert_eq!(st.facts[&addr], reference, "facts identical at t={t}");
        let ph = phase_at(&st.facts[&addr], UniverseTime::seconds(t));
        let expect = if t < born {
            Phase::Unborn { in_s: born - t }
        } else if t < ends {
            Phase::Alive { age_s: t - born }
        } else {
            Phase::Ended { since_s: t - ends }
        };
        assert_eq!(ph, expect);
    }
}

/// Recorded facts win over a changed algorithm (H5.5), and re-resolving
/// appends nothing.
#[test]
fn recorded_facts_persist() {
    let mut env = fixture();
    let mut c = Chronicle::new();
    let first =
        resolve_recorded(&mut c, &seed(), &env, &ChildRef::Generated(0), &publisher()).unwrap();
    assert!(first.new);
    env.fields.insert(
        "metallicity".into(),
        FieldSpec::Draw {
            dist: Distribution::Fixed { value: 9.0 },
            version: 2,
        },
    );
    let again =
        resolve_recorded(&mut c, &seed(), &env, &ChildRef::Generated(0), &publisher()).unwrap();
    assert!(!again.new);
    assert_eq!(again.facts, first.facts);
    assert_eq!(again.envelope, first.envelope);
    assert_eq!(c.len(), 1);
    // A child cites its parent's resolve as its cause.
    let sys = first.envelope.unwrap();
    let body =
        resolve_recorded(&mut c, &seed(), &sys, &ChildRef::Generated(0), &publisher()).unwrap();
    let ev = c.events().iter().find(|e| e.id == body.event).unwrap();
    assert_eq!(ev.body.causes, vec![first.event]);
}

#[test]
fn bad_envelopes_are_rejected_readably() {
    let mut env = fixture();
    env.pins[0].address = env.address.child("sys", 3).unwrap();
    assert!(env
        .validate()
        .unwrap_err()
        .to_string()
        .contains("generated index"));
    let mut env = fixture();
    env.pins[1].claims.insert("unobtainium".into(), 1);
    assert!(env
        .validate()
        .unwrap_err()
        .to_string()
        .contains("unknown budget"));
    let env = fixture();
    let n = children(&seed(), &env).unwrap().generated;
    assert!(resolve(&seed(), &env, &ChildRef::Generated(n))
        .unwrap_err()
        .to_string()
        .contains("does not exist"));
}

#[test]
fn location_first_outlook() {
    let env = fixture();
    let o = location_first(&env).unwrap();
    assert_eq!(o.expected_generated, 12.0);
    assert_eq!(o.pins, 2);
    match &o.fields["multiplicity"] {
        FieldOutlook::Category { probabilities } => {
            assert_eq!(probabilities[0], ("single".to_string(), 0.55))
        }
        other => panic!("{other:?}"),
    }
    match &o.fields["metallicity"] {
        FieldOutlook::Numeric { p05, p50, p95, .. } => {
            assert!(*p05 < *p50 && *p50 < *p95);
            assert!(p50.abs() < 0.05);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(o.budgets["gas"].pinned, 2_900_000_000_000);
}

fn clouds() -> Vec<Envelope> {
    let base = fixture();
    (0..30u64)
        .map(|i| {
            let mut e = base.clone();
            e.address = base
                .address
                .parent()
                .unwrap()
                .child_key("patch", &format!("3:{i:x}"))
                .unwrap();
            e.pins.clear();
            let mean = -0.6 + i as f64 * 0.04;
            e.fields.insert(
                "metallicity".into(),
                FieldSpec::Draw {
                    dist: Distribution::Normal {
                        mean,
                        sd: 0.1,
                        lo: None,
                        hi: None,
                    },
                    version: 1,
                },
            );
            e
        })
        .collect()
}

fn metal_rich() -> Wish {
    Wish {
        label: "metal-rich single star".into(),
        criteria: vec![
            Criterion {
                field: "metallicity".into(),
                min: Some(0.3),
                max: None,
                is: None,
            },
            Criterion {
                field: "multiplicity".into(),
                min: None,
                max: None,
                is: Some("single".into()),
            },
        ],
        min_expected: 0.5,
    }
}

#[test]
fn outcome_first_ranks_and_ignores_caller_order() {
    let mut cs = clouds();
    let a = outcome_first(&cs, &metal_rich(), 10_000, 5);
    cs.reverse();
    let b = outcome_first(&cs, &metal_rich(), 10_000, 5);
    assert_eq!(a, b);
    let QueryOutcome::Found {
        candidates,
        searched,
        ..
    } = a
    else {
        panic!("{a:?}")
    };
    assert_eq!(searched, 30);
    assert_eq!(candidates[0].address.to_string(), "u:k7/patch:3:1d");
    assert!(candidates
        .windows(2)
        .all(|w| w[0].expected_matches >= w[1].expected_matches));
}

#[test]
fn not_found_within_budget_never_impossible() {
    // Budget too small to reach the rich clouds (address order: 3:0.. first).
    let out = outcome_first(&clouds(), &metal_rich(), 9, 5);
    match out {
        QueryOutcome::NotFoundWithinBudget {
            searched,
            of,
            work,
            nearest,
        } => {
            assert_eq!((searched, of, work), (3, 30, 9));
            assert!(nearest.len() <= 3);
        }
        other => panic!("{other:?}"),
    }
    // A wish about something no cloud describes is still only "not found".
    let odd = Wish {
        label: "x".into(),
        criteria: vec![Criterion {
            field: "dragons".into(),
            min: Some(1.0),
            max: None,
            is: None,
        }],
        min_expected: 0.0,
    };
    let QueryOutcome::NotFoundWithinBudget { nearest, .. } =
        outcome_first(&clouds(), &odd, 1_000, 5)
    else {
        panic!()
    };
    assert!(nearest[0].reasons[0].contains("says nothing about dragons"));
    // Walking real children with a tiny budget stops and says where.
    let s = search_children(&seed(), &fixture(), &odd, 0, 3, 1).unwrap();
    assert!(s.not_found_within_budget());
    assert!(s.resume_at.unwrap() > 0);
}

#[test]
fn search_children_finds_the_real_children() {
    let env = fixture();
    let wish = Wish {
        label: "binary".into(),
        criteria: vec![Criterion {
            field: "multiplicity".into(),
            min: None,
            max: None,
            is: Some("binary".into()),
        }],
        min_expected: 0.0,
    };
    let s = search_children(&seed(), &env, &wish, 0, 1_000_000, 100).unwrap();
    assert_eq!(s.resume_at, None);
    for (addr, facts) in &s.hits {
        let i: u64 = addr.last().key().parse().unwrap();
        assert_eq!(
            &resolve(&seed(), &env, &ChildRef::Generated(i))
                .unwrap()
                .facts,
            facts
        );
        assert_eq!(facts["multiplicity"], Value::Text("binary".into()));
    }
}

#[test]
fn queries_mutate_nothing_until_a_pick_is_committed() {
    let env = fixture();
    let mut c = Chronicle::new();
    resolve_recorded(&mut c, &seed(), &env, &ChildRef::Generated(0), &publisher()).unwrap();
    let before = (c.len(), c.state_at(UniverseTime::ORIGIN).hash());
    let _ = location_first(&env).unwrap();
    let _ = outcome_first(&clouds(), &metal_rich(), 1000, 3);
    let _ = search_children(&seed(), &env, &metal_rich(), 0, 1000, 3).unwrap();
    assert_eq!(before, (c.len(), c.state_at(UniverseTime::ORIGIN).hash()));
    let addr = env.address.child("sys", 0).unwrap();
    commit_pick(
        &mut c,
        &addr,
        &metal_rich(),
        UniverseTime::ORIGIN,
        &publisher(),
    )
    .unwrap();
    let st = c.state_at(UniverseTime::ORIGIN);
    assert_eq!(
        st.facts[&addr]["query.selected_for"],
        Value::Text("metal-rich single star".into())
    );
}

/// Native golden digest; CI runs it on Linux, macOS and Windows.
#[test]
fn golden_digest() {
    let env = fixture();
    let mut c = Chronicle::new();
    for r in refs(&env) {
        let rec = resolve_recorded(&mut c, &seed(), &env, &r, &publisher()).unwrap();
        let sys = rec.envelope.unwrap();
        for b in refs(&sys) {
            resolve_recorded(&mut c, &seed(), &sys, &b, &publisher()).unwrap();
        }
    }
    let outlook =
        ContentHash::of(&serde_json::to_string(&location_first(&env).unwrap()).unwrap()).unwrap();
    let got = serde_json::json!({
        "resolve_tree_state": c.state_at(UniverseTime::ORIGIN).hash().hex(),
        "events": c.len(),
        "outlook": outlook.hex(),
    });
    crate::golden::check("resolve.json", &got);
}
