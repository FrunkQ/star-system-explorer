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

// ---- C1: the SSE 3 classic scheme and a warming Earth ----

/// The SSE 3 classic scheme pack (`packs/sse3-classic`).
pub fn classic_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/sse3-classic/pack.json"))
        .expect("classic pack")
}
pub fn classic_atlas() -> crate::Atlas {
    crate::pack::load(&[classic_pack()])
        .expect("classic pack loads")
        .atlas
}
/// SSE4's shipped atlas (C2): physical, surface, air, habitability.
pub fn base_atlas_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-atlas/pack.json"))
        .expect("base atlas pack")
}
pub fn base_atlas() -> crate::Atlas {
    crate::pack::load(&[base_atlas_pack()])
        .expect("base atlas loads")
        .atlas
}
fn solar() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/classify/solar-system.json"
    ))
    .expect("solar fixture")
}
fn body_facts(v: &serde_json::Value) -> BTreeMap<String, Value> {
    serde_json::from_value(v.clone()).expect("facts")
}

fn authored(ch: &mut Chronicle, a: &Address, t_gyr: i64, field: &str, v: Value) {
    let mut f = BTreeMap::new();
    f.insert(field.to_string(), v);
    let mut set = BTreeMap::new();
    set.insert(a.clone(), f);
    let t = UniverseTime::years(t_gyr * 1_000_000_000);
    let e = Event::new(EventBody {
        kind: "authored.set".into(),
        subject: vec![a.clone()],
        occurred_at: t,
        activated_at: t,
        recognised_at: ch.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("gm", "edit", "1"),
    })
    .expect("event");
    ch.append(e).expect("append");
}

/// An Earth that warms, as an ageing run would leave it: a brighter Sun,
/// the oceans go, then the crust melts. Times are every 1 Gyr from 0 to 4.
pub fn warming_earth() -> (Chronicle, Address, Vec<UniverseTime>) {
    let a: Address = "u:sol/sys:0/body:2".parse().expect("address");
    let mut ch = Chronicle::new();
    let sol = solar();
    let earth = sol["bodies"]
        .as_array()
        .expect("bodies")
        .iter()
        .find(|b| b["name"] == "Earth")
        .expect("Earth");
    for (k, v) in body_facts(&earth["facts"]) {
        authored(&mut ch, &a, 0, &k, v);
    }
    authored(
        &mut ch,
        &a,
        1,
        "planet.surface_temperature",
        Value::Float(330.0),
    );
    authored(
        &mut ch,
        &a,
        2,
        "hydrosphere.liquidCoverage",
        Value::Float(0.0),
    );
    authored(&mut ch, &a, 2, "hydrosphere.coverage", Value::Float(0.0));
    authored(&mut ch, &a, 2, "surface.liquid_fraction", Value::Float(0.0));
    authored(&mut ch, &a, 2, "surface.liquid", Value::Text("none".into()));
    authored(
        &mut ch,
        &a,
        2,
        "planet.surface_temperature",
        Value::Float(600.0),
    );
    authored(
        &mut ch,
        &a,
        3,
        "planet.surface_temperature",
        Value::Float(1500.0),
    );
    let times = (0..=4)
        .map(|g| UniverseTime::years(g * 1_000_000_000))
        .collect();
    (ch, a, times)
}

/// C1 golden: classic labels of the Solar System and the warming Earth's
/// relabel events (ids and the chronicle head), identical on every device.
pub fn classify_golden() -> serde_json::Value {
    let atlas = classic_atlas();
    let labels: Vec<serde_json::Value> = solar()["bodies"]
        .as_array()
        .expect("bodies")
        .iter()
        .map(|b| {
            let l = atlas.label("sse3-classic", &Subject::new(body_facts(&b["facts"])), None);
            serde_json::json!([b["name"], l.classic.map(|w| w.classes), l.fit])
        })
        .collect();
    let (mut ch, a, times) = warming_earth();
    let rl = crate::relabel::record(&mut ch, &atlas, "sse3-classic", &a, &times).expect("relabel");
    // C2: the base atlas over the same bodies, and the warming Earth's
    // surface relabels on a fresh copy of the chronicle.
    let base = base_atlas();
    let schemes = ["physical", "surface", "air", "habitability"];
    let base_labels: Vec<serde_json::Value> = solar()["bodies"]
        .as_array()
        .expect("bodies")
        .iter()
        .map(|b| {
            let s = Subject::new(body_facts(&b["facts"]));
            let ls: Vec<serde_json::Value> = schemes
                .iter()
                .map(|sc| {
                    let l = base.label(sc, &s, None);
                    serde_json::json!([l.base, l.modifiers, l.fit])
                })
                .collect();
            serde_json::json!([b["name"], ls])
        })
        .collect();
    let (mut ch2, a2, times2) = warming_earth();
    let surface =
        crate::relabel::record(&mut ch2, &base, "surface", &a2, &times2).expect("relabel");
    serde_json::json!({
        "base_atlas": base.content_hash().hex(),
        "base_solar_system": base_labels,
        "base_surface_relabels": surface.iter().map(|r| serde_json::json!([r.at.s, r.from, r.to, r.event.hex()])).collect::<Vec<_>>(),
        "atlas": atlas.content_hash().hex(),
        "solar_system": labels,
        "relabels": rl.iter().map(|r| serde_json::json!([r.at.s, r.from, r.to, r.event.hex()])).collect::<Vec<_>>(),
        "chronicle_head": ch.head().hex(),
    })
}
