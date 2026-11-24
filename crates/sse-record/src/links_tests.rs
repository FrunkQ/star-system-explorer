//! Links between places (space highways): written, replaced, withdrawn,
//! dated, and kept from an audience that does not know of them.

use super::links::*;
use super::*;

fn addr(s: &str) -> Address {
    s.parse().unwrap()
}
fn t(y: i64) -> UniverseTime {
    UniverseTime::years(y)
}
fn pubr() -> Publisher {
    Publisher::new("gm", "record", "1")
}
fn gate() -> LinkDraft {
    LinkDraft {
        root: addr("u:demo"),
        id: "long-gate".into(),
        a: addr("u:demo/sys:tarn"),
        b: addr("u:demo/sys:vesh/body:ring/site:plaza"),
        name: "the Long Gate".into(),
        kind: "gate".into(),
        one_way: false,
        nav: Some(r#"{"rule":{"id":"gate","name":"gate transit","family":"gate","duration":{"form":"fixed","days":1},"messages":{"by":"ships"},"fictional":true}}"#.into()),
        tier: None,
        audience: Audience::everyone(),
        writer: "gm".into(),
        reason: None,
        source: None,
        withdraw: false,
    }
}

#[test]
fn a_link_is_written_dated_replaced_and_withdrawn() {
    let mut ch = Chronicle::new();
    let (link, _) = set_link(&mut ch, &gate(), t(10), &pubr()).unwrap();
    assert_eq!(link.to_string(), "u:demo/link:long-gate");
    assert!(
        links_at(ch.events(), t(9), "gm").is_empty(),
        "not before it was made"
    );
    let now = links_at(ch.events(), t(11), "gm");
    assert_eq!(now.len(), 1);
    assert_eq!(
        (now[0].name.as_str(), now[0].kind.as_str()),
        ("the Long Gate", "gate")
    );
    assert_eq!(now[0].b, addr("u:demo/sys:vesh/body:ring/site:plaza"));
    let mut one_way = gate();
    one_way.one_way = true;
    set_link(&mut ch, &one_way, t(20), &pubr()).unwrap();
    let h = history(ch.events(), t(30));
    assert_eq!(h.len(), 2);
    assert_eq!(h[0].until_s, Some(t(20).s));
    assert!(h[1].one_way);
    let mut gone = gate();
    gone.withdraw = true;
    set_link(&mut ch, &gone, t(40), &pubr()).unwrap();
    assert!(links_at(ch.events(), t(41), "gm").is_empty());
    assert_eq!(
        links_at(ch.events(), t(35), "gm").len(),
        1,
        "rewinding brings it back"
    );
}

#[test]
fn a_secret_link_never_reaches_an_audience_that_does_not_know_it() {
    let mut ch = Chronicle::new();
    let mut secret = gate();
    secret.id = "smugglers-run".into();
    secret.audience = Audience::gm();
    set_link(&mut ch, &secret, t(10), &pubr()).unwrap();
    set_link(&mut ch, &gate(), t(10), &pubr()).unwrap();
    assert_eq!(links_at(ch.events(), t(11), "gm").len(), 2);
    let crew = links_at(ch.events(), t(11), "crew");
    assert_eq!(
        crew.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        vec!["long-gate"]
    );
}

#[test]
fn a_link_needs_two_ends_in_its_universe_and_json_travel_data() {
    let mut ch = Chronicle::new();
    let mut same = gate();
    same.b = same.a.clone();
    assert!(set_link(&mut ch, &same, t(1), &pubr()).is_err());
    let mut away = gate();
    away.b = addr("u:other/sys:x");
    assert!(set_link(&mut ch, &away, t(1), &pubr()).is_err());
    let mut junk = gate();
    junk.nav = Some("not json".into());
    assert!(set_link(&mut ch, &junk, t(1), &pubr()).is_err());
}
