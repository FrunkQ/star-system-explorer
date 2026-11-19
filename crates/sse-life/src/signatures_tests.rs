//! L1d checks: signatures by tier, the scripted false positive and false
//! negative, data-view integration, sifters and hooks.

use crate::scenario::{observables, signature_cases};
use crate::sift::{epochs, hooks, view, HookInputs, SiftParams};
use crate::signatures::{disclosure, facts, signatures, SignatureParams};

fn case(name: &str) -> crate::signatures::Signatures {
    let (_, b, h, o) = signature_cases().into_iter().find(|c| c.0 == name).unwrap();
    signatures(b.as_ref(), h.as_ref(), &o, &SignatureParams::default())
}

#[test]
fn earth_reads_as_alive_at_every_tier() {
    let s = case("earth");
    println!(
        "earth: o2 {:.3}, ch4 {:.2} ppm, edge {:?}, confidence {:.3}, colour {:?}, traces {:?}",
        s.o2,
        s.ch4_ppm,
        s.edge_nm,
        s.confidence,
        s.colour,
        s.traces
            .iter()
            .map(|t| (t.kind.as_str(), t.class.as_str()))
            .collect::<Vec<_>>()
    );
    let ids: Vec<&str> = s.signals.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, ["o2_ch4", "red_edge"]);
    assert!((0.1..0.3).contains(&s.o2), "{}", s.o2);
    assert!((0.5..5.0).contains(&s.ch4_ppm), "{}", s.ch4_ppm);
    assert!(s.candidate && s.confidence > 0.99);
    assert!(s.colour.is_some() && s.seasonal);
    assert!((690.0..710.0).contains(&s.edge_nm.unwrap()));
    assert!(s.summary.as_deref().unwrap().contains("guilds"));
    assert!(!s.guild_cards.is_empty());
    let class = |k: &str| s.traces.iter().find(|t| t.kind == k).unwrap().class.clone();
    assert_eq!(class("fossils"), "abundant");
    assert_eq!(class("isotopes"), "rare");
}

#[test]
fn archean_earth_shows_methane_without_oxygen() {
    let s = case("earth-archean");
    println!(
        "archean: o2 {:.2e}, ch4 {:.1} ppm, confidence {:.3}",
        s.o2, s.ch4_ppm, s.confidence
    );
    assert!(s.o2 < 1.0e-3);
    assert_eq!(s.signals[0].id, "ch4");
    assert!(s.candidate && s.confidence < 0.8);
    assert!(s.colour.is_none(), "no land plants, no colour from afar");
}

#[test]
fn scripted_false_positive_and_false_negative() {
    // A lifeless red-dwarf desert with oxygen left by water loss: a candidate,
    // with a modest confidence and a reason that names the trap.
    let fp = case("red-dwarf-desert");
    assert!(!fp.alive && !fp.ever_alive);
    assert!(fp.candidate, "players see a candidate");
    assert!((0.5..0.75).contains(&fp.confidence), "{}", fp.confidence);
    assert_eq!(fp.signals[0].id, "o2");
    assert!(fp.signals[0].why.contains("water loss"));
    assert!(fp.summary.is_none(), "a survey finds no biosphere");
    // Europa: alive under ice, and the scan reads empty, without lowering the odds.
    let fneg = case("europa");
    assert!(fneg.alive && !fneg.candidate);
    assert_eq!(fneg.signals[0].id, "nothing");
    assert!(
        (fneg.confidence - 0.5).abs() < 1e-12,
        "a blind scan says nothing either way"
    );
    assert!(fneg.blind.as_deref().unwrap().contains("ice shell"));
    assert!(
        fneg.summary.is_some(),
        "a survey that reaches the ocean finds it"
    );
    // An Earth-like world where nothing shows lowers the odds.
    let mut o = observables("earth");
    o.abiotic_ch4_ppm = 0.0;
    let empty = signatures(None, None, &o, &SignatureParams::default());
    assert!(!empty.candidate && empty.confidence < 0.5);
    // Hooks follow.
    let sp = SiftParams::default();
    let inp = HookInputs {
        habitable_until_myr: None,
        o2_fraction: fp.o2,
        now_myr: 6000.0,
    };
    let ids = |s: &crate::signatures::Signatures,
               b: Option<&crate::biosphere::Biosphere>|
     -> Vec<String> {
        hooks(b, None, s, &inp, &sp)
            .into_iter()
            .map(|h| h.id)
            .collect()
    };
    assert_eq!(ids(&fp, None), ["false_positive"]);
    let eu = crate::scenario::europa_biosphere();
    let hk = ids(&fneg, Some(&eu));
    assert!(hk.contains(&"hidden_life".to_string()), "{hk:?}");
}

#[test]
fn data_views_show_each_tier_only() {
    use sse_chronicle::{
        kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, SessionStamp,
        UniverseTime,
    };
    use sse_core::Address;
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/packs/tarn-observe.json"
    ))
    .unwrap();
    let mut obs = pack["provides"]["other"]["observe"].clone();
    for (k, v) in disclosure() {
        obs["disclosure"]["fields"][k] = v.into();
    }
    let setup = sse_observe::ObserveSetup::from_stack(&[("tarn-observe".into(), obs)])
        .unwrap()
        .unwrap();
    let views = setup.views_by_id();
    let mut c = Chronicle::new();
    let mut put = |name: &str, s: &crate::signatures::Signatures, n: u64| {
        let a: Address = format!("u:tarn/sys:0/body:{n}").parse().unwrap();
        let row = facts(s)
            .into_iter()
            .map(|(k, v)| (k, serde_json::from_value(v).unwrap()))
            .collect();
        let mut set = std::collections::BTreeMap::new();
        set.insert(a.clone(), row);
        c.append(
            Event::new(EventBody {
                kind: kinds::RESOLVE.into(),
                subject: vec![a],
                occurred_at: UniverseTime::ORIGIN,
                activated_at: UniverseTime::ORIGIN,
                recognised_at: SessionStamp(n + 1),
                causes: vec![],
                payload: Payload {
                    set,
                    ..Default::default()
                },
                evidence: Evidence::Resolved,
                publisher: Publisher::new("sse.life", name, "1"),
            })
            .unwrap(),
        )
        .unwrap();
    };
    put("earth", &case("earth"), 1);
    put("red-dwarf-desert", &case("red-dwarf-desert"), 2);
    let st = c.state_at(UniverseTime::seconds(i64::MAX));
    let tags = Default::default();
    let fictional = Default::default();
    let inp = sse_observe::SceneInput {
        state: &st,
        chronicle: Some(&c),
        tags: &tags,
        fictional: &fictional,
        disclosure: &setup.disclosure,
        views: &views,
        campaign: &setup.campaign,
        notes: &[],
    };
    let scope: Address = "u:tarn".parse().unwrap();
    let shown = |lens: &str| -> Vec<Vec<String>> {
        let sc = sse_observe::build_scene_through(&inp, "players", &scope, Some(lens)).unwrap();
        sc.subjects
            .iter()
            .map(|s| {
                s.facts
                    .iter()
                    .filter(|f| f.value.is_some() || f.band.is_some())
                    .map(|f| f.field.clone())
                    .collect()
            })
            .collect()
    };
    for (lens, see, hide) in [
        (
            "long-range",
            "life.obvious.colour",
            "life.signature.candidate",
        ),
        (
            "scanned",
            "life.signature.confidence",
            "life.survey.summary",
        ),
        ("survey", "life.survey.summary", "gm.life.alive"),
    ] {
        let s = shown(lens);
        assert!(s[0].iter().any(|f| f == see), "{lens}: {:?}", s[0]);
        assert!(
            s.iter().all(|x| !x.iter().any(|f| f == hide)),
            "{lens} hides {hide}"
        );
        assert!(
            s.iter().all(|x| !x.iter().any(|f| f.starts_with("gm."))),
            "{lens}: no truth"
        );
    }
    // The false positive shows its candidate in the scanned view.
    let s = shown("scanned");
    assert!(s[1].iter().any(|f| f == "life.signature.candidate"));
    // The GM sees the truth.
    let gm = sse_observe::build_scene(&inp, "gm", &scope).unwrap();
    assert!(gm.subjects[1]
        .facts
        .iter()
        .any(|f| f.field == "gm.life.alive"));
}

#[test]
fn sifters_name_contiguous_epochs_with_reasons() {
    let (_, b, h, o) = signature_cases()
        .into_iter()
        .find(|c| c.0 == "earth")
        .unwrap();
    let h = h.unwrap();
    let e = epochs(&h, o.age_myr, &SiftParams::default());
    let names: Vec<&str> = e.iter().map(|x| x.name.as_str()).collect();
    println!(
        "{:?}",
        e.iter()
            .map(|x| (x.name.as_str(), x.from_myr, x.to_myr, x.major))
            .collect::<Vec<_>>()
    );
    assert_eq!(names[0], "The barren age");
    assert!(names.contains(&"The age of green mats"));
    for w in e.windows(2) {
        assert_eq!(w[0].to_myr, w[1].from_myr, "contiguous");
    }
    assert_eq!(e.last().unwrap().to_myr, o.age_myr);
    assert!(e.iter().any(|x| x.major));
    assert!(e
        .iter()
        .all(|x| !x.why.physics.is_empty() && !x.why.history.is_empty()));
    // A world one innovation short of animals, and a doomed biosphere.
    let sp = SiftParams::default();
    let s = case("earth");
    let mut short = b.clone().unwrap();
    short
        .capabilities
        .retain(|c| c != "large_bodies" && c != "hard_parts");
    let inp = HookInputs {
        habitable_until_myr: Some(4800.0),
        o2_fraction: 0.02,
        now_myr: 4570.0,
    };
    let ids: Vec<String> = hooks(Some(&short), Some(&h), &s, &inp, &sp)
        .into_iter()
        .map(|x| x.id)
        .collect();
    assert!(
        ids.contains(&"one_short_of_animals".to_string()) && ids.contains(&"doomed".to_string()),
        "{ids:?}"
    );
    // A dead Earth still tells its story.
    let dead = case("dead-earth");
    let ids: Vec<String> = hooks(None, Some(&h), &dead, &inp, &sp)
        .into_iter()
        .map(|x| x.id)
        .collect();
    assert!(ids.contains(&"dead_world_record".to_string()), "{ids:?}");
    // The view gathers it all.
    let v = view("u:sol/sys:0/body:3", b.as_ref(), Some(&h), &s, &inp, &sp);
    assert_eq!(v.lenses.len(), 4);
    assert!(!v.timeline.is_empty() && !v.epochs.is_empty());
}

#[test]
fn golden() {
    let g = crate::scenario::signatures_golden();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/life-signatures.json"
    );
    let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn pack_matches_the_defaults() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-life/pack.json")).unwrap();
    let o = &pack["provides"]["other"];
    let mut p: SignatureParams = serde_json::from_value(o["life_signatures"].clone()).unwrap();
    assert!(!p.sources.is_empty());
    p.sources.clear();
    assert_eq!(p, SignatureParams::default());
    let d: std::collections::BTreeMap<String, String> =
        serde_json::from_value(o["life_signatures"]["disclosure"].clone()).unwrap();
    assert_eq!(d, disclosure());
    let s: SiftParams = serde_json::from_value(o["life_sift"].clone()).unwrap();
    assert_eq!(s, SiftParams::default());
}
