//! The golden scenario: K7's fixture patch resolved, a few authored edits,
//! the fixture rule pack applied and its sifters run. Used by the tests and
//! by the browser harness.

use crate::{load, sift, Hit, SiftContent, Subject, TagState, World};
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash, Seed};
use sse_pack::PackManifest;
use sse_resolve::{resolve_recorded, ChildRef, Envelope};
use std::collections::BTreeMap;

pub const RULES_PACK_JSON: &str = include_str!("../../../tests/fixtures/sift/rules-pack.json");

pub fn pack() -> PackManifest {
    PackManifest::from_json(RULES_PACK_JSON).expect("fixture pack is valid")
}

fn facts(kv: &[(&str, Value)]) -> BTreeMap<String, Value> {
    kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

pub fn seed() -> Seed {
    Seed::from_text("k8")
}
pub fn patch() -> Envelope {
    sse_resolve::scenario::patch()
}

pub fn set(
    c: &mut Chronicle,
    kind: &str,
    a: &Address,
    f: BTreeMap<String, Value>,
    at: UniverseTime,
) -> ContentHash {
    let mut s = BTreeMap::new();
    s.insert(a.clone(), f);
    let e = Event::new(EventBody {
        kind: kind.into(),
        subject: vec![a.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: c.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: s,
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("test.gm", "edit", "1"),
    })
    .unwrap();
    c.append(e).unwrap()
}

pub struct Campaign {
    pub c: Chronicle,
    pub clouds: BTreeMap<Address, Envelope>,
    pub now: UniverseTime,
    pub moon: Address,
    pub moon_event: ContentHash,
}

pub fn campaign() -> Campaign {
    let env = patch();
    let mut c = Chronicle::new();
    let pubr = Publisher::new("test.resolve", "patch", "1");
    let k = sse_resolve::children(&seed(), &env).unwrap();
    let mut first_sys = None;
    for i in 0..k.generated {
        let r = resolve_recorded(&mut c, &seed(), &env, &ChildRef::Generated(i), &pubr).unwrap();
        first_sys.get_or_insert((env.address.child("sys", i).unwrap(), r.envelope.unwrap()));
    }
    let (sys, sys_env) = first_sys.unwrap();
    let r0 = resolve_recorded(&mut c, &seed(), &sys_env, &ChildRef::Generated(0), &pubr).unwrap();
    let world = sys.child("body", 0).unwrap();
    assert!(r0.new);
    let now = UniverseTime::seconds(435_000_000_000_000_000);
    set(
        &mut c,
        kinds::AUTHORED_SET,
        &world,
        facts(&[
            ("roche_m", Value::Float(1.8e7)),
            ("day_s", Value::Float(1.0e7)),
            ("year_s", Value::Float(1.01e7)),
        ]),
        UniverseTime::ORIGIN,
    );
    let moon = world.authored_child("body", 1).unwrap();
    let moon_event = set(
        &mut c,
        kinds::AUTHORED_SET,
        &moon,
        facts(&[("orbit_m", Value::Float(2.0e7))]),
        UniverseTime::ORIGIN,
    );
    set(
        &mut c,
        kinds::OBSERVE_GRANT,
        &world,
        facts(&[("observe.view", Value::Text("scanned".into()))]),
        now,
    );
    let mut clouds = BTreeMap::new();
    clouds.insert(env.address.clone(), env);
    Campaign {
        c,
        clouds,
        now,
        moon,
        moon_event,
    }
}

pub fn run(cp: &Campaign, content: &SiftContent) -> Vec<Hit> {
    let state = cp.c.state_at(cp.now);
    // Tags from the rule pack, then sift.
    let mut tags: BTreeMap<Address, BTreeMap<String, TagState>> = BTreeMap::new();
    for a in state.facts.keys() {
        let s = Subject {
            address: Some(a.clone()),
            facts: state.facts[a].clone(),
            parent: a
                .parent()
                .and_then(|p| state.facts.get(&p).cloned())
                .unwrap_or_default(),
            now_s: cp.now.s,
            ..Default::default()
        };
        let t = content.rules.apply(&s, &BTreeMap::new());
        if !t.is_empty() {
            tags.insert(a.clone(), t);
        }
    }
    let w = World {
        state: &state,
        chronicle: Some(&cp.c),
        tags: &tags,
        clouds: &cp.clouds,
        now: cp.now,
    };
    sift(&content.sifters, &w, &"u:k7".parse().unwrap())
}

pub fn golden() -> serde_json::Value {
    let cp = campaign();
    let content = load(&[pack()]).expect("fixture pack loads");
    let hits = run(&cp, &content);
    let digest = ContentHash::of(&serde_json::to_string(&hits).expect("json")).expect("hash");
    serde_json::json!({
        "hits": hits.len(),
        "hits_digest": digest.hex(),
        "state": cp.c.state_at(cp.now).hash().hex(),
    })
}
