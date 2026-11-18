//! C3 checks (classification design section 12): a requested class is
//! placed and then recognised as that class (round trip) on a fixture set
//! of stars; impossible requests return the blocking reason; forced
//! placements carry the mark; the same result on every device (golden).

use crate::scenario::*;
use crate::*;
use sse_sift::classify::Atlas;

struct Parts {
    atlas: Atlas,
    model: Model,
    life: sse_life::NoLifeModel,
    rules: Vec<sse_life::HabitatRule>,
}
fn parts() -> Parts {
    Parts {
        atlas: sse_sift::scenario::base_atlas(),
        model: model(),
        life: sse_life::NoLifeModel::default(),
        rules: sse_life::scenario::base_rules(),
    }
}
impl Parts {
    fn placer(&self) -> Placer<'_> {
        Placer {
            atlas: &self.atlas,
            model: &self.model,
            life: &self.life,
            life_rules: &self.rules,
        }
    }
}

/// The class read back from the chronicle after commit.
fn read_back(p: &Placer, sys: &str, r: &Placement) -> (Option<String>, Vec<String>, Chronicle) {
    let mut ch = Chronicle::new();
    let at = UniverseTime::years(1);
    commit(&mut ch, &address(sys), at, r).unwrap();
    let state = ch.state_at(at);
    let facts = state.facts.get(&address(sys)).cloned().unwrap_or_default();
    let l = p
        .atlas
        .label(&r.scheme, &sse_sift::predicate::Subject::new(facts), None);
    (l.base, l.modifiers, ch)
}

#[test]
fn requested_classes_are_placed_and_recognised_round_trip() {
    let ps = parts();
    let p = ps.placer();
    for (s, sc, c, scope) in CASES {
        if scope.is_some() {
            continue;
        }
        let r = run(&p, s, sc, c, scope, false);
        assert!(
            matches!(r.outcome, Outcome::Placed | Outcome::NeedsHelp),
            "{s} {c}: {:?} {:?}",
            r.outcome,
            r.blocking
        );
        assert!(r.recognised && r.fit >= 1.0, "{s} {c}");
        assert!(r.blocking.is_empty(), "{s} {c}: {:?}", r.blocking);
        // Causes are authored, results derived, nothing impossible.
        assert_eq!(r.marks.get("body.mass"), Some(&Mark::Authored));
        assert_eq!(
            r.marks.get("planet.equilibrium_temperature"),
            Some(&Mark::Derived)
        );
        assert!(!r.marks.values().any(|m| *m == Mark::AuthoredImpossible));
        // Written to the chronicle and classified again from its state.
        let (base, mods, _) = read_back(&p, s, &r);
        let back = base.as_deref() == Some(c)
            || mods.iter().any(|m| m == c)
            || base
                .as_deref()
                .is_some_and(|b| b.starts_with(&format!("{c}/")));
        assert!(back, "{s} {c}: read back as {base:?} {mods:?}");
    }
}

#[test]
fn impossible_requests_say_what_blocks_them() {
    let ps = parts();
    let p = ps.placer();
    // Too far out for open water.
    let r = run(
        &p,
        "sun",
        "surface",
        "surface/temperate-ocean",
        Some((30.0, 40.0)),
        false,
    );
    assert_eq!(r.outcome, Outcome::ClosestHere);
    assert!(
        r.blocking[0].contains("outside the requested 30 to 40 au"),
        "{:?}",
        r.blocking
    );
    assert_eq!(r.label.as_deref(), Some("surface/cold-desert"));
    // Inside Jupiter's reach.
    let r = run(
        &p,
        "sun",
        "surface",
        "surface/cold-desert",
        Some((5.0, 5.4)),
        false,
    );
    assert_eq!(r.outcome, Outcome::ClosestHere);
    assert!(
        r.blocking[0].contains("too close to Jupiter"),
        "{:?}",
        r.blocking
    );
    // Close enough to be hot, too close to keep the air a greenhouse needs.
    let r = run(
        &p,
        "f-star",
        "surface",
        "surface/hot-greenhouse",
        Some((0.11, 0.16)),
        false,
    );
    assert_eq!(r.outcome, Outcome::ClosestHere);
    assert!(r.blocking[0].contains("loses its air"), "{:?}", r.blocking);
    assert!(
        r.blocking.len() > 1,
        "and names the failing axis: {:?}",
        r.blocking
    );
}

#[test]
fn forced_placements_carry_the_mark() {
    let ps = parts();
    let p = ps.placer();
    let r = run(
        &p,
        "sun",
        "surface",
        "surface/temperate-ocean",
        Some((30.0, 40.0)),
        true,
    );
    assert_eq!(r.outcome, Outcome::Forced);
    let forced: Vec<&String> = r
        .marks
        .iter()
        .filter(|(_, m)| **m == Mark::AuthoredImpossible)
        .map(|(k, _)| k)
        .collect();
    assert!(
        forced.iter().any(|k| *k == "planet.surface_temperature"),
        "{forced:?}"
    );
    assert!(forced.iter().any(|k| *k == "surface.liquid"), "{forced:?}");
    assert!(r.recognised, "forced values make the class: {:?}", r.label);
    let (base, _, ch) = read_back(&p, "sun", &r);
    assert_eq!(base.as_deref(), Some("surface/temperate-ocean"));
    let marked = ch
        .events()
        .iter()
        .filter(|e| e.body.evidence == Evidence::AuthoredImpossible)
        .count();
    assert_eq!(
        marked, 1,
        "one authored-impossible event holds every forced value"
    );
}

#[test]
fn oxygen_and_life_need_a_helping_hand_while_no_life_model_is_installed() {
    let ps = parts();
    let p = ps.placer();
    let r = run(
        &p,
        "sun",
        "habitability",
        "habitable/shirt-sleeve",
        None,
        false,
    );
    assert_eq!(r.outcome, Outcome::NeedsHelp);
    assert!(
        r.interventions[0].contains("oxygenic_photosynthesis"),
        "{:?}",
        r.interventions
    );
    assert!(r.interventions[0].contains("no life model installed"));
    let (_, _, ch) = read_back(&p, "sun", &r);
    let iv: Vec<_> = ch
        .events()
        .iter()
        .filter(|e| e.body.kind == "place.intervention")
        .collect();
    assert_eq!(iv.len(), 1);
    assert_eq!(iv[0].body.evidence, Evidence::Authored);
    // A living world asks the life suite too; an inhabited one needs an author.
    let r = run(&p, "k-dwarf", "physical", "physical/inhabited", None, false);
    assert_eq!(
        r.outcome,
        Outcome::ClosestHere,
        "nothing can make people appear by physics"
    );
    let r = run(&p, "k-dwarf", "physical", "physical/inhabited", None, true);
    assert_eq!(r.outcome, Outcome::Forced);
    assert!(
        r.interventions
            .iter()
            .any(|i| i.contains("authored settlement")),
        "{:?}",
        r.interventions
    );
}

#[test]
fn the_picker_shows_every_class_with_a_reason() {
    let ps = parts();
    let p = ps.placer();
    let sun = system("sun");
    let near = p.preview(&seed(), &address("sun"), &sun, "surface", None);
    let far = p.preview(
        &seed(),
        &address("sun"),
        &sun,
        "surface",
        Some((30.0, 40.0)),
    );
    let bases = ps
        .atlas
        .classes
        .iter()
        .filter(|c| c.scheme == "surface" && c.kind == sse_sift::classify::ClassKind::Base)
        .count();
    assert_eq!((near.len(), far.len()), (bases, bases));
    let get = |v: &[(String, Availability, String)], c: &str| {
        v.iter().find(|x| x.0 == c).cloned().unwrap()
    };
    assert_eq!(
        get(&near, "surface/temperate-ocean").1,
        Availability::Available
    );
    assert_eq!(get(&near, "surface/molten").1, Availability::Available);
    let t = get(&far, "surface/temperate-ocean");
    assert_eq!(t.1, Availability::NotHere);
    assert!(t.2.contains("outside the requested"), "{}", t.2);
    assert_eq!(get(&far, "surface/frozen").1, Availability::Available);
    for (c, a, why) in near.iter().chain(&far) {
        assert!(
            *a == Availability::Available || !why.is_empty(),
            "{c} has no reason"
        );
    }
}

#[test]
fn same_request_same_answer_and_the_seed_matters() {
    let ps = parts();
    let p = ps.placer();
    let a = run(&p, "k-dwarf", "physical", "water-rich", None, false);
    let b = run(&p, "k-dwarf", "physical", "water-rich", None, false);
    assert_eq!(a, b);
    let req = Request {
        scheme: "physical".into(),
        class: "water-rich".into(),
        scope_au: None,
        force: false,
    };
    let c = p
        .place(
            &sse_core::Seed::from_text("other"),
            &address("k-dwarf"),
            &system("k-dwarf"),
            &req,
        )
        .unwrap();
    assert_ne!(a.causes, c.causes);
    assert!(p
        .place(
            &seed(),
            &address("sun"),
            &system("sun"),
            &Request {
                class: "nope".into(),
                ..req
            }
        )
        .is_err());
}

#[test]
fn golden() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/place.json");
    let got = crate::scenario::golden();
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    let want = std::fs::read_to_string(path).expect("run with SSE_BLESS=1 once");
    assert_eq!(
        text, want,
        "tests/golden/place.json moved: bless only on purpose"
    );
}
