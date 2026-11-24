//! J1 checks (record design section 10): audiences, kept values that fold and
//! explain themselves, the writer rule, unloading a pack, actors, and an
//! empty record leaving state as it was.

use super::*;
use sse_chronicle::kinds as ck;

fn addr(s: &str) -> Address {
    s.parse().unwrap()
}
fn pubr() -> Publisher {
    Publisher::new("gm", "record", "1")
}
fn t(y: i64) -> UniverseTime {
    UniverseTime::years(y)
}

fn reg() -> Registry {
    let pack = serde_json::json!({
        "kept": [
            {"id": "trade.credits", "shape": "number", "unit": "Cr", "range": [-1.0e9, 1.0e12],
             "default": 0, "tier": "obvious", "writers": ["gm", "intent.trade"],
             "describes": "the party's purse"},
            {"id": "politics.standing", "shape": "number", "range": [-10.0, 10.0], "default": 0,
             "tier": "secret", "writers": ["gm", "effect.consequence"], "describes": "how an actor regards the party"},
            {"id": "politics.allegiance", "shape": "ref", "tier": "obvious", "writers": ["gm"],
             "describes": "whom an actor or place answers to"},
            {"id": "politics.status", "shape": "choice", "choices": ["peace", "tension", "war"],
             "tier": "obvious", "writers": ["gm"], "describes": "relations between two actors"}
        ]
    });
    Registry::from_packs(&[("rpg-politics".into(), pack)]).unwrap()
}

fn draft(text: &str, audience: Audience) -> NoteDraft {
    NoteDraft {
        subjects: vec![addr("u:a/sys:1/body:3")],
        text: text.into(),
        tags: vec!["insult".into()],
        by: "gm".into(),
        audience,
        tier: None,
        supersedes: None,
        narrative: false,
        arrives: None,
    }
}

#[test]
fn a_note_for_one_audience_is_not_for_another() {
    let mut ch = Chronicle::new();
    let crew = add_note(
        &mut ch,
        &draft(
            "the crew peed on the statue",
            Audience::Only(vec!["crew".into()]),
        ),
        t(10),
        &pubr(),
    )
    .unwrap();
    let all = add_note(
        &mut ch,
        &draft("a statue stands here", Audience::everyone()),
        t(10),
        &pubr(),
    )
    .unwrap();
    let gm = add_note(
        &mut ch,
        &draft("the statue is a sleeping machine", Audience::gm()),
        t(10),
        &pubr(),
    )
    .unwrap();
    let notes = notes_at(ch.events(), t(11));
    assert_eq!(notes.len(), 3);
    let seen = |aud: &str| -> Vec<String> {
        notes
            .iter()
            .filter(|n| n.visible_to(aud, None))
            .map(|n| n.id.clone())
            .collect()
    };
    assert_eq!(seen("crew").len(), 2);
    assert!(!seen("vesh").contains(&crew.hex()) && seen("vesh").contains(&all.hex()));
    assert!(!seen("crew").contains(&gm.hex()));
    assert_eq!(seen("gm").len(), 3);
    // Not yet written at year 9.
    assert!(notes_at(ch.events(), t(9)).is_empty());
    let _ = (crew, all, gm);
}

#[test]
fn news_arrives_later_and_not_before() {
    let mut ch = Chronicle::new();
    let mut d = draft("the Calla was taken off Tarn", Audience::everyone());
    d.arrives = Some(t(12));
    add_note(&mut ch, &d, t(10), &pubr()).unwrap();
    assert!(notes_at(ch.events(), t(11)).is_empty());
    let n = notes_at(ch.events(), t(12));
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].at_s, t(10).s, "dated when it happened");
    d.arrives = Some(t(9));
    assert!(add_note(&mut ch, &d, t(10), &pubr()).is_err());
}

#[test]
fn notes_never_touch_state() {
    let mut ch = Chronicle::new();
    let before = ch.state_at(t(100)).hash();
    add_note(&mut ch, &draft("x", Audience::everyone()), t(1), &pubr()).unwrap();
    assert_eq!(ch.state_at(t(100)).hash(), before);
}

#[test]
fn tiered_notes_wait_for_the_tier() {
    let mut ch = Chronicle::new();
    let mut d = draft("deep ruins under the ice", Audience::everyone());
    d.tier = Some("surveyed".into());
    add_note(&mut ch, &d, t(1), &pubr()).unwrap();
    let n = &notes_at(ch.events(), t(2))[0];
    let scanned = vec!["obvious".to_string(), "scanned".to_string()];
    let surveyed = vec!["obvious".to_string(), "surveyed".to_string()];
    assert!(!n.visible_to("crew", Some(&scanned)));
    assert!(n.visible_to("crew", Some(&surveyed)));
    assert!(!n.visible_to("crew", None));
}

#[test]
fn corrections_supersede_and_history_is_kept() {
    let mut ch = Chronicle::new();
    let a = add_note(
        &mut ch,
        &draft("we owe the guild 500", Audience::everyone()),
        t(1),
        &pubr(),
    )
    .unwrap();
    let mut fix = draft("we owe the guild 800", Audience::everyone());
    fix.supersedes = Some(a);
    let b = add_note(&mut ch, &fix, t(2), &pubr()).unwrap();
    let live = notes_at(ch.events(), t(3));
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].id, b.hex());
    assert_eq!(live[0].supersedes.as_deref(), Some(a.hex().as_str()));
    // Before the correction the old note stands.
    assert_eq!(notes_at(ch.events(), t(1))[0].id, a.hex());
    // Withdraw: an empty note superseding it.
    let mut w = draft("", Audience::everyone());
    w.supersedes = Some(b);
    add_note(&mut ch, &w, t(4), &pubr()).unwrap();
    assert!(notes_at(ch.events(), t(5)).is_empty());
    assert_eq!(
        ch.events()
            .iter()
            .filter(|e| e.body.kind == kinds::NOTE)
            .count(),
        3
    );
    // Refusals.
    let mut bad = draft("x", Audience::everyone());
    bad.supersedes = Some(ContentHash::of(&1u8).unwrap());
    assert!(matches!(
        add_note(&mut ch, &bad, t(6), &pubr()),
        Err(RecordError::UnknownNote(_))
    ));
    assert!(add_note(&mut ch, &draft("  ", Audience::everyone()), t(6), &pubr()).is_err());
    assert!(add_note(&mut ch, &draft("x", Audience::Only(vec![])), t(6), &pubr()).is_err());
    assert!(add_note(
        &mut ch,
        &draft("x", Audience::Named("crew".into())),
        t(6),
        &pubr()
    )
    .is_err());
    let mut tag = draft("x", Audience::everyone());
    tag.tags = vec!["Bad Tag".into()];
    assert!(add_note(&mut ch, &tag, t(6), &pubr()).is_err());
}

#[test]
fn narrative_notes_are_marked() {
    let mut ch = Chronicle::new();
    let mut d = draft("rumour says the Calla was taken", Audience::everyone());
    d.narrative = true;
    d.by = "minerva".into();
    let id = add_note(&mut ch, &d, t(1), &pubr()).unwrap();
    let e = ch.events().iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.body.evidence, Evidence::Narrative);
    assert!(notes_at(ch.events(), t(2))[0].narrative);
}

fn set(a: &str, kind: &str, v: Option<Value>, writer: &str, reason: &str) -> KeptSet {
    KeptSet {
        address: addr(a),
        kind: kind.into(),
        value: v,
        writer: writer.into(),
        reason: Some(reason.into()),
        causes: vec![],
    }
}

#[test]
fn a_kept_value_folds_from_its_events_and_explains_itself() {
    let r = reg();
    let mut ch = Chronicle::new();
    let party = "u:a/actor:party";
    set_kept(
        &mut ch,
        &r,
        &set(
            party,
            "trade.credits",
            Some(Value::Int(1000)),
            "gm",
            "starting purse",
        ),
        t(1),
        &pubr(),
    )
    .unwrap();
    set_kept(
        &mut ch,
        &r,
        &set(
            party,
            "trade.credits",
            Some(Value::Int(1400)),
            "intent.trade",
            "sold ore at Tarn",
        ),
        t(2),
        &pubr(),
    )
    .unwrap();
    set_kept(
        &mut ch,
        &r,
        &set(
            party,
            "trade.credits",
            Some(Value::Int(1386)),
            "gm",
            "docking fee",
        ),
        t(3),
        &pubr(),
    )
    .unwrap();
    // The fold reads it as an ordinary fact.
    assert_eq!(
        ch.state_at(t(3)).get(&addr(party), "trade.credits"),
        Some(&Value::Int(1386))
    );
    assert_eq!(
        ch.state_at(t(2)).get(&addr(party), "trade.credits"),
        Some(&Value::Int(1400))
    );
    // Why is it 1386? Walk the events.
    let k = kept_at(&ch, &r, &addr(party), t(10));
    assert_eq!(k.len(), 1);
    assert_eq!(k[0].value, Value::Int(1386));
    assert_eq!(k[0].kept_by.as_deref(), Some("rpg-politics"));
    let h = &k[0].history;
    assert_eq!(h.len(), 3);
    assert_eq!(h[1].writer, "intent.trade");
    assert_eq!(h[1].reason.as_deref(), Some("sold ore at Tarn"));
    assert_eq!(h[1].delta, Some(400.0));
    assert_eq!(h[2].delta, Some(-14.0));
    assert_eq!(h[0].delta, None);
    // At year 2 the history stops there.
    assert_eq!(kept_at(&ch, &r, &addr(party), t(2))[0].history.len(), 2);
    // Clearing leaves no value but keeps the history in the chronicle.
    set_kept(
        &mut ch,
        &r,
        &set(party, "trade.credits", None, "gm", "account closed"),
        t(4),
        &pubr(),
    )
    .unwrap();
    assert!(kept_at(&ch, &r, &addr(party), t(10)).is_empty());
    assert_eq!(ch.state_at(t(10)).get(&addr(party), "trade.credits"), None);
}

#[test]
fn an_unlisted_writer_is_refused_and_values_must_fit() {
    let r = reg();
    let mut ch = Chronicle::new();
    let a = "u:a/actor:vesh";
    let e = set_kept(
        &mut ch,
        &r,
        &set(
            a,
            "politics.standing",
            Some(Value::Int(-2)),
            "intent.trade",
            "x",
        ),
        t(1),
        &pubr(),
    );
    assert!(matches!(e, Err(RecordError::WriterRefused { .. })), "{e:?}");
    assert!(matches!(
        set_kept(
            &mut ch,
            &r,
            &set(a, "politics.standing", Some(Value::Int(-20)), "gm", "x"),
            t(1),
            &pubr()
        ),
        Err(RecordError::BadValue { .. })
    ));
    assert!(set_kept(
        &mut ch,
        &r,
        &set(
            a,
            "politics.status",
            Some(Value::Text("feud".into())),
            "gm",
            "x"
        ),
        t(1),
        &pubr()
    )
    .is_err());
    assert!(set_kept(
        &mut ch,
        &r,
        &set(
            a,
            "politics.allegiance",
            Some(Value::Text("not an address".into())),
            "gm",
            "x"
        ),
        t(1),
        &pubr()
    )
    .is_err());
    assert!(set_kept(
        &mut ch,
        &r,
        &set(
            a,
            "trade.credits",
            Some(Value::Text("lots".into())),
            "gm",
            "x"
        ),
        t(1),
        &pubr()
    )
    .is_err());
    assert!(matches!(
        set_kept(
            &mut ch,
            &r,
            &set(a, "x4.metal", Some(Value::Int(1)), "gm", "x"),
            t(1),
            &pubr()
        ),
        Err(RecordError::UnknownKind(_))
    ));
    // misc: anything, GM only.
    set_kept(
        &mut ch,
        &r,
        &set(
            a,
            "misc.favourite_drink",
            Some(Value::Text("tea".into())),
            "gm",
            "x",
        ),
        t(1),
        &pubr(),
    )
    .unwrap();
    assert!(set_kept(
        &mut ch,
        &r,
        &set(a, "misc.x", Some(Value::Int(1)), "crew", "x"),
        t(1),
        &pubr()
    )
    .is_err());
    assert!(set_kept(
        &mut ch,
        &r,
        &set(a, "misc.", Some(Value::Int(1)), "gm", "x"),
        t(1),
        &pubr()
    )
    .is_err());
    // Nothing refused reached the chronicle.
    assert_eq!(ch.len(), 1);
}

#[test]
fn relations_are_kept_refs_between_actors() {
    let r = reg();
    let mut ch = Chronicle::new();
    let root = addr("u:a");
    let vesh = add_actor(
        &mut ch,
        &root,
        None,
        "vesh",
        "Vesh Hegemony",
        "empire",
        t(0),
        &pubr(),
    )
    .unwrap();
    let fleet = add_actor(
        &mut ch,
        &root,
        Some(&vesh),
        "third-fleet",
        "Third Fleet",
        "fleet",
        t(0),
        &pubr(),
    )
    .unwrap();
    assert_eq!(fleet.to_string(), "u:a/actor:vesh/actor:third-fleet");
    set_kept(
        &mut ch,
        &r,
        &KeptSet {
            address: addr("u:a/sys:1/body:3"),
            kind: "politics.allegiance".into(),
            value: Some(Value::Text(vesh.to_string())),
            writer: "gm".into(),
            reason: None,
            causes: vec![],
        },
        t(1),
        &pubr(),
    )
    .unwrap();
    let list = actors(&ch, &root, t(2));
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].1, "Vesh Hegemony");
    assert_eq!(list[1].2, "fleet");
    // Refusals.
    assert!(add_actor(
        &mut ch,
        &root,
        None,
        "vesh",
        "again",
        "empire",
        t(1),
        &pubr()
    )
    .is_err());
    assert!(add_actor(
        &mut ch,
        &root,
        Some(&addr("u:a/sys:1")),
        "x",
        "X",
        "guild",
        t(1),
        &pubr()
    )
    .is_err());
    assert!(add_actor(
        &mut ch,
        &root,
        Some(&addr("u:a/actor:nobody")),
        "x",
        "X",
        "guild",
        t(1),
        &pubr()
    )
    .is_err());
    assert!(add_actor(
        &mut ch,
        &root,
        None,
        "bad key!",
        "X",
        "guild",
        t(1),
        &pubr()
    )
    .is_err());
}

#[test]
fn unloading_the_pack_leaves_values_readable() {
    let r = reg();
    let mut ch = Chronicle::new();
    let a = addr("u:a/actor:party");
    set_kept(
        &mut ch,
        &r,
        &set(
            "u:a/actor:party",
            "trade.credits",
            Some(Value::Int(5)),
            "gm",
            "x",
        ),
        t(1),
        &pubr(),
    )
    .unwrap();
    set_kept(
        &mut ch,
        &r,
        &set(
            "u:a/actor:party",
            "misc.mood",
            Some(Value::Text("grim".into())),
            "gm",
            "x",
        ),
        t(1),
        &pubr(),
    )
    .unwrap();
    let gone = Registry::default();
    let k = kept_at(&ch, &gone, &a, t(2));
    assert_eq!(k.len(), 2);
    let credits = k.iter().find(|x| x.kind == "trade.credits").unwrap();
    assert_eq!(credits.value, Value::Int(5));
    assert_eq!(credits.kept_by, None);
    assert_eq!(
        credits.orphan.as_deref(),
        Some("kept by a pack that is not loaded")
    );
    assert_eq!(
        k.iter().find(|x| x.kind == "misc.mood").unwrap().orphan,
        None
    );
    // But nothing new can be written to it without its keeper.
    assert!(set_kept(
        &mut ch,
        &gone,
        &set(
            "u:a/actor:party",
            "trade.credits",
            Some(Value::Int(6)),
            "gm",
            "x"
        ),
        t(2),
        &pubr()
    )
    .is_err());
}

#[test]
fn one_keeper_per_kind_and_declarations_are_checked() {
    let k = |id: &str| serde_json::json!({"kept": [{"id": id, "shape": "number", "tier": "obvious", "writers": ["gm"], "describes": "d"}]});
    let e = Registry::from_packs(&[
        ("a".into(), k("trade.credits")),
        ("b".into(), k("trade.credits")),
    ])
    .unwrap_err();
    assert!(e[0].contains("already kept by a"), "{e:?}");
    assert!(Registry::from_packs(&[("a".into(), k("credits"))]).is_err());
    assert!(Registry::from_packs(&[("a".into(), k("misc.x"))]).is_err());
    let bad = serde_json::json!({"kept": [
        {"id": "a.b", "shape": "choice", "tier": "obvious", "writers": ["gm"], "describes": "d"},
        {"id": "a.c", "shape": "number", "range": [2.0, 1.0], "tier": "obvious", "writers": ["gm"], "describes": "d"},
        {"id": "a.d", "shape": "flag", "default": 3, "tier": "obvious", "writers": ["gm"], "describes": "d"},
        {"id": "a.e", "shape": "flag", "tier": "obvious", "writers": [], "describes": "d"}
    ]});
    assert_eq!(
        Registry::from_packs(&[("p".into(), bad)])
            .unwrap_err()
            .len(),
        4
    );
    // Disclosure: each kind at its tier, misc at the secret rung.
    let d = reg().disclosure_fields("secret");
    assert_eq!(d["trade.credits"], "obvious");
    assert_eq!(d["politics.standing"], "secret");
    assert_eq!(d["misc.*"], "secret");
}

#[test]
fn an_empty_record_changes_nothing() {
    // A universe with facts and no record events folds as before, and the
    // record readers find nothing.
    let mut ch = Chronicle::new();
    let a = addr("u:a/sys:1");
    ch.append(
        Event::new(EventBody {
            kind: ck::AUTHORED_SET.into(),
            subject: vec![a.clone()],
            occurred_at: t(0),
            activated_at: t(0),
            recognised_at: ch.next_stamp(),
            causes: vec![],
            payload: Payload {
                set: [(
                    a.clone(),
                    [("name".to_string(), Value::Text("Tarn".into()))].into(),
                )]
                .into(),
                ..Default::default()
            },
            evidence: Evidence::Authored,
            publisher: pubr(),
        })
        .unwrap(),
    )
    .unwrap();
    let h = ch.state_at(t(1)).hash();
    assert!(notes_at(ch.events(), t(1)).is_empty());
    assert!(kept_at(&ch, &reg(), &a, t(1)).is_empty());
    assert!(actors(&ch, &addr("u:a"), t(1)).is_empty());
    assert_eq!(ch.state_at(t(1)).hash(), h);
}

/// Track MC: the base manifest pack's kinds load, and a person aboard moves
/// from one ship to another as one dated change whose history is kept.
#[test]
fn the_manifest_pack_moves_people_between_ships() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-manifest/pack.json")).unwrap();
    let r = Registry::from_packs(&[(
        "sse.base-manifest".into(),
        pack["provides"]["other"]["record"].clone(),
    )])
    .unwrap();
    let mut ch = Chronicle::new();
    let root = addr("u:a");
    let ada = add_actor(&mut ch, &root, None, "ada", "Ada", "person", t(0), &pubr()).unwrap();
    let a = ada.to_string();
    let on = |ship: &str| Some(Value::Text(ship.into()));
    set_kept(
        &mut ch,
        &r,
        &set(&a, "aboard.on", on("u:a/sys:1/ship:1"), "gm", "signed on"),
        t(1),
        &pubr(),
    )
    .unwrap();
    set_kept(
        &mut ch,
        &r,
        &set(
            &a,
            "aboard.state",
            Some(Value::Text("cold_sleep".into())),
            "gm",
            "for the crossing",
        ),
        t(1),
        &pubr(),
    )
    .unwrap();
    set_kept(
        &mut ch,
        &r,
        &set(&a, "aboard.on", on("u:a/sys:2/ship:7"), "gm", "transferred"),
        t(3),
        &pubr(),
    )
    .unwrap();
    // A state the pack does not list is refused; so is a writer it does not name.
    assert!(set_kept(
        &mut ch,
        &r,
        &set(
            &a,
            "aboard.state",
            Some(Value::Text("asleep".into())),
            "gm",
            ""
        ),
        t(3),
        &pubr()
    )
    .is_err());
    assert!(set_kept(
        &mut ch,
        &r,
        &set(
            &a,
            "aboard.role",
            Some(Value::Text("pilot".into())),
            "effect.x",
            ""
        ),
        t(3),
        &pubr()
    )
    .is_err());
    let kept = kept_at(&ch, &r, &ada, t(4));
    let on = kept.iter().find(|k| k.kind == "aboard.on").unwrap();
    assert_eq!(on.value, Value::Text("u:a/sys:2/ship:7".into()));
    assert_eq!(on.history.len(), 2);
    assert_eq!(on.history[0].reason.as_deref(), Some("signed on"));
    // Before the transfer she was on the first ship.
    let then = kept_at(&ch, &r, &ada, t(2));
    assert_eq!(
        then.iter().find(|k| k.kind == "aboard.on").unwrap().value,
        Value::Text("u:a/sys:1/ship:1".into())
    );
    assert_eq!(actors(&ch, &root, t(4)).len(), 1);
}
