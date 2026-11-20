//! J5 checks: the built-in kinds decide as the chart room does (shared
//! cases), a pack adds a kind with a validity condition, a program seat
//! cannot send outside its may_send or scope, and the GM queue.

use super::intents::*;
use super::*;
use serde_json::json;

fn root() -> Address {
    "u:t".parse().unwrap()
}

fn cases() -> J {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/intents/decide-cases.json"
    );
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn mining_pack() -> J {
    json!({"kinds": [{
        "kind": "refuel",
        "meaning": "take on propellant at a port",
        "fields": [{"name": "address", "type": "address"}, {"name": "tonnes", "type": "number"}],
        "valid": {"if": "all", "of": [
            {"if": "range", "field": "intent.tonnes", "min": 0.001},
            {"if": "tag", "tag": "port"}
        ]}
    }, {
        "kind": "mine",
        "meaning": "fill the hold from a body",
        "fields": [{"name": "address", "type": "address"}]
    }]})
}

#[test]
fn built_ins_decide_as_the_chart_room_does() {
    let reg = IntentRegistry::default();
    let c = cases();
    let all = c["cases"].as_array().unwrap();
    assert!(all.len() >= 10);
    for k in all {
        let seat: Seat = serde_json::from_value(k["seat"].clone()).unwrap();
        let d = decide(&reg, &seat, &k["intent"], None);
        let got = serde_json::to_value(&d).unwrap();
        assert_eq!(got["action"], k["action"], "{}: {d:?}", k["name"]);
    }
    // Malformed intents are refused with the chart room's words.
    let gm = Seat::gm(&reg);
    let bad = |i: J| match decide(&reg, &gm, &i, None) {
        Decision::Refuse { reason } => reason,
        d => panic!("{i} gave {d:?}"),
    };
    assert!(bad(json!({"kind": "teleport"})).contains("unknown intent kind"));
    assert!(bad(
        json!({"kind": "set_value", "address": "u:t/a:1", "field": "Name", "value": 1, "at_yr": 0})
    )
    .contains("lower-case"));
    assert!(
        bad(json!({"kind": "set_value", "address": "u:t/a:1", "field": "name", "at_yr": 0}))
            .contains("value is required")
    );
    assert!(bad(json!({"kind": "grant", "address": "u:t/a:1", "audience": "v", "view": null, "default": true, "at_yr": 0})).contains("cannot be taken back"));
    assert!(bad(json!({"kind": "stack_packs", "packs": [{"id": "x"}]})).contains("sse.pack/1"));
    assert!(bad(json!({"kind": "scan_toggle", "address": "u:t/a:1", "audience": "v", "at_yr": 0, "extra": 1})).contains("unknown field"));
    // The translation is the chart room's intentToRequest.
    let r = to_request(&json!({"kind": "set_value", "address": "u:t/a:1", "field": "name", "value": "Tarn", "at_yr": 2.5}), "gm", 0).unwrap();
    assert_eq!(
        r,
        json!({"op": "set_authored", "edit": {"address": "u:t/a:1", "field": "name", "value": "Tarn", "occurred_at_s": 3 * YEAR_S, "impossible": false}})
    );
    let r = to_request(
        &json!({"kind": "grant", "address": "u:t/a:1", "audience": "v", "view": null, "at_yr": 1}),
        "gm",
        0,
    )
    .unwrap();
    assert_eq!(
        r,
        json!({"op": "grant", "address": "u:t/a:1", "audience": "v", "at_s": YEAR_S})
    );
    // A seat's own journal applies; a note to others asks.
    let vesh: Seat = serde_json::from_value(
        json!({"id": "vesh", "audience": "vesh", "authority": "player", "may_send": ["add_note"]}),
    )
    .unwrap();
    let note = |aud: J| json!({"kind": "add_note", "address": "u:t/a:1", "text": "we were here", "audience": aud});
    assert_eq!(
        decide(&reg, &vesh, &note(json!(["vesh"])), None),
        Decision::Apply
    );
    assert_eq!(
        decide(&reg, &vesh, &note(json!("everyone")), None),
        Decision::AskGm
    );
}

#[test]
fn pack_kinds_validity_and_program_seats() {
    let reg = IntentRegistry::from_packs(&[("test.mining".into(), mining_pack())]).unwrap();
    assert_eq!(reg.get("refuel").unwrap().applied_by, AppliedBy::Record);
    assert_eq!(reg.get("set_value").unwrap().applied_by, AppliedBy::Kernel);
    // Refusals at load.
    for (bad, why) in [
        (
            json!({"kinds": [{"kind": "set_value", "meaning": "x"}]}),
            "already declared by kernel",
        ),
        (
            json!({"kinds": [{"kind": "proposed", "meaning": "x"}]}),
            "reserved",
        ),
        (
            json!({"kinds": [{"kind": "Go-Fast", "meaning": "x"}]}),
            "lowercase",
        ),
        (
            json!({"kinds": [{"kind": "go", "meaning": "x", "subject": "where", "fields": [{"name": "where", "type": "text"}]}]}),
            "declared address field",
        ),
        (
            json!({"kinds": [{"kind": "go", "meaning": "x", "colour": 1}]}),
            "unknown field",
        ),
    ] {
        let e = IntentRegistry::from_packs(&[("p".into(), bad)])
            .unwrap_err()
            .join("; ");
        assert!(e.contains(why), "{e}");
    }

    // A port and an ordinary moon.
    let mut uni = Chronicle::new();
    let record = Chronicle::new();
    let port: Address = "u:t/sys:0/body:1".parse().unwrap();
    let moon: Address = "u:t/sys:0/body:2".parse().unwrap();
    let mut p = Payload::default();
    p.set.insert(
        port.clone(),
        [("tag.port".to_string(), Value::Bool(true))].into(),
    );
    p.set.insert(
        moon.clone(),
        [("body.mass".to_string(), Value::Float(1e20))].into(),
    );
    authored(
        &mut uni,
        &root(),
        UniverseTime::ORIGIN,
        p,
        Publisher::new("gm", "edit", "1"),
    )
    .unwrap();

    let drone: Seat = serde_json::from_value(json!({
        "id": "drone-1", "label": "Drone 1", "audience": "crew", "authority": "player",
        "may_send": ["refuel", "mine"], "program": "mining-drone",
        "auto_approve": [{"intent": "refuel", "scope": ["u:t/sys:0"]}, {"intent": "mine", "scope": ["u:t/sys:0/body:2"]}]
    }))
    .unwrap();
    let go = |kind: &str, a: &Address, t: f64| json!({"kind": kind, "address": a.to_string(), "tonnes": t});
    let run = |i: &J| {
        let v = validity(&reg, i, &uni, &record, &root(), 10);
        decide(&reg, &drone, i, v)
    };
    assert_eq!(run(&go("refuel", &port, 5.0)), Decision::Apply);
    let r = run(&go("refuel", &moon, 5.0));
    assert!(
        matches!(&r, Decision::Refuse { reason } if reason.contains("not valid now")),
        "{r:?}"
    );
    let r = run(&go("refuel", &port, 0.0));
    assert!(
        matches!(&r, Decision::Refuse { reason } if reason.contains("intent.tonnes")),
        "{r:?}"
    );
    let mine = |a: &Address| json!({"kind": "mine", "address": a.to_string()});
    assert_eq!(run(&mine(&moon)), Decision::Apply);
    // Outside its scope, and outside its may_send: refused, never queued.
    let r = run(&mine(&port));
    assert!(
        matches!(&r, Decision::Refuse { reason } if reason.contains("outside program seat Drone 1's scope")),
        "{r:?}"
    );
    let sv = json!({"kind": "set_value", "address": moon.to_string(), "field": "name", "value": "x", "at_yr": 0});
    assert!(matches!(run(&sv), Decision::Refuse { .. }));
    // A person with the same seat but no program is asked about, not refused.
    let mut person = drone.clone();
    person.program = None;
    assert_eq!(decide(&reg, &person, &mine(&port), None), Decision::AskGm);
    // A program never holds GM authority, nor sends session kinds.
    let mut boss = drone.clone();
    boss.authority = Authority::Gm;
    assert!(check_seats(&reg, &[boss])[0].contains("never has GM authority"));
    let mut packer = drone.clone();
    packer.may_send.push("stack_packs".into());
    let r = decide(
        &reg,
        &packer,
        &json!({"kind": "stack_packs", "packs": []}),
        None,
    );
    assert!(
        matches!(&r, Decision::Refuse { reason } if reason.contains("a program cannot send")),
        "{r:?}"
    );
}

#[test]
fn seats_queue_and_answers_fold_from_the_record() {
    let reg = IntentRegistry::from_packs(&[("test.mining".into(), mining_pack())]).unwrap();
    let mut record = Chronicle::new();
    // Default plan: the GM and the table.
    let s = seats(&reg, &record);
    assert_eq!(
        s.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(),
        ["gm", "table"]
    );
    assert!(
        s[0].may_send.contains(&"refuel".to_string()),
        "the GM may send pack kinds"
    );
    let bad: Seat = serde_json::from_value(json!({"id": "x", "audience": "x", "authority": "player", "may_send": ["fly"], "auto_approve": [{"intent": "mine", "scope": []}]})).unwrap();
    let e = set_seats(&reg, &mut record, &root(), &[bad], 0).unwrap_err();
    assert!(
        e.contains("unknown intent fly") && e.contains("applies mine without asking"),
        "{e}"
    );
    let crew: Seat = serde_json::from_value(json!({"id": "crew", "label": "Crew", "audience": "crew", "authority": "player", "may_send": ["mine"], "screens": ["bridge"]})).unwrap();
    set_seats(&reg, &mut record, &root(), std::slice::from_ref(&crew), 0).unwrap();
    let s = seats(&reg, &record);
    assert_eq!(s.len(), 2);
    assert_eq!(
        s[1].rest["screens"],
        json!(["bridge"]),
        "the chart room's own fields ride along"
    );

    let i = json!({"kind": "mine", "address": "u:t/sys:0/body:2"});
    assert_eq!(decide(&reg, &crew, &i, None), Decision::AskGm);
    let id = propose(&reg, &mut record, &root(), &crew, &i, 50).unwrap();
    let p = pending(&record);
    assert_eq!(p.len(), 1);
    assert_eq!(
        (p[0].seat.as_str(), &p[0].intent, p[0].at_s),
        ("crew", &i, 50)
    );
    let (got, ans) = answer(&mut record, &id.hex(), true, 60).unwrap();
    assert!(pending(&record).is_empty());
    assert!(answer(&mut record, &id.hex(), false, 70)
        .unwrap_err()
        .contains("already answered"));
    let w = record_intent(
        &reg,
        &mut record,
        &root(),
        &got.seat,
        &got.intent,
        60,
        vec![ans],
    )
    .unwrap();
    let e = record.events().iter().find(|e| e.id == w).unwrap();
    assert_eq!(e.body.kind, "intent.mine");
    assert_eq!(e.body.causes, vec![ans]);
    assert_eq!(data_text(e, "from"), Some("seat.crew"));
    assert_eq!(
        data_text(e, "payload"),
        Some(r#"{"address":"u:t/sys:0/body:2"}"#)
    );
    // None of it is in the universe; it all replays.
    let again = {
        let mut r2 = Chronicle::new();
        set_seats(&reg, &mut r2, &root(), std::slice::from_ref(&crew), 0).unwrap();
        let id = propose(&reg, &mut r2, &root(), &crew, &i, 50).unwrap();
        let (g, a) = answer(&mut r2, &id.hex(), true, 60).unwrap();
        record_intent(&reg, &mut r2, &root(), &g.seat, &g.intent, 60, vec![a]).unwrap();
        r2.head()
    };
    assert_eq!(again, record.head());
}
