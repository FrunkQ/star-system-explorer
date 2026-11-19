//! The golden scenario: the Kestrel system from the PlayerData mockup
//! board, before and after Tarn is scanned. Used by the tests and by the
//! browser harness.

use crate::scene::GM;
use crate::*;
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::{BTreeMap, BTreeSet};

pub const WORLD_JSON: &str = include_str!("../../../tests/fixtures/observe/tarn-world.json");
pub const BEFORE_JSON: &str = include_str!("../../../tests/fixtures/observe/tarn-before.json");
pub const AFTER_JSON: &str = include_str!("../../../tests/fixtures/observe/tarn-after.json");

#[derive(serde::Deserialize)]
pub struct World {
    pub disclosure: Disclosure,
    pub views: Vec<DataView>,
    pub campaign: Campaign,
    pub resolved: BTreeMap<Address, BTreeMap<String, Value>>,
    pub authored: BTreeMap<Address, BTreeMap<String, Value>>,
    pub tags: BTreeMap<Address, BTreeSet<String>>,
    pub grants: BTreeMap<Address, String>,
}

/// One of the three embedded fixtures, by file name.
pub fn read<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let text = match name {
        "tarn-world.json" => WORLD_JSON,
        "tarn-before.json" => BEFORE_JSON,
        "tarn-after.json" => AFTER_JSON,
        other => panic!("no fixture {other}"),
    };
    serde_json::from_str(text).expect("fixture is valid")
}

pub fn gm() -> Publisher {
    Publisher::new("test.gm", "session", "1")
}

pub fn put(c: &mut Chronicle, kind: &str, ev: Evidence, a: &Address, f: BTreeMap<String, Value>) {
    let e = Event::new(EventBody {
        kind: kind.into(),
        subject: vec![a.clone()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: c.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: [(a.clone(), f)].into(),
            ..Default::default()
        },
        evidence: ev,
        publisher: gm(),
    })
    .unwrap();
    c.append(e).unwrap();
}

pub struct Fx {
    pub w: World,
    pub views: BTreeMap<String, DataView>,
    pub c: Chronicle,
    pub now: UniverseTime,
}

pub fn setup() -> Fx {
    let w: World = read("tarn-world.json");
    let mut c = Chronicle::new();
    for (a, f) in &w.resolved {
        put(&mut c, kinds::RESOLVE, Evidence::Resolved, a, f.clone());
    }
    for (a, f) in &w.authored {
        put(
            &mut c,
            kinds::AUTHORED_SET,
            Evidence::Authored,
            a,
            f.clone(),
        );
    }
    for (a, v) in &w.grants {
        put(
            &mut c,
            kinds::OBSERVE_GRANT,
            Evidence::Authored,
            a,
            [("observe.view".to_string(), Value::Text(v.clone()))].into(),
        );
    }
    let views = w.views.iter().map(|v| (v.id.clone(), v.clone())).collect();
    Fx {
        w,
        views,
        c,
        now: UniverseTime::years(4_610_000_000),
    }
}

pub fn scene(fx: &Fx, audience: &str) -> Scene {
    let state = fx.c.state_at(fx.now);
    let fictional = BTreeSet::new();
    let notes = sse_record::notes_at(fx.c.events(), fx.now);
    let inp = SceneInput {
        state: &state,
        chronicle: Some(&fx.c),
        tags: &fx.w.tags,
        fictional: &fictional,
        disclosure: &fx.w.disclosure,
        views: &fx.views,
        campaign: &fx.w.campaign,
        notes: &notes,
    };
    build_scene(&inp, audience, &"u:kestrel/sys:0".parse().unwrap()).unwrap()
}

pub fn tarn() -> Address {
    "u:kestrel/sys:0/body:3".parse().unwrap()
}

pub fn golden() -> serde_json::Value {
    let mut fx = setup();
    let before = scene(&fx, EVERYONE).hash();
    let campaign = fx.w.campaign.clone();
    scan_toggle(&mut fx.c, &tarn(), EVERYONE, &campaign, fx.now, &gm()).expect("scan");
    serde_json::json!({
        "tarn_before": before.hex(),
        "tarn_after": scene(&fx, EVERYONE).hash().hex(),
        "gm": scene(&fx, GM).hash().hex(),
    })
}
