//! C3 fixtures: four stars, placements in them, and the golden.

use crate::{Model, Placement, Placer, Request, Settings, System};
use sse_core::{Address, Seed};
use sse_pack::PackManifest;

pub fn place_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-place/pack.json"))
        .expect("place pack")
}
pub fn physics_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-physics/pack.json"))
        .expect("physics pack")
}
pub fn model() -> Model {
    Model::new(
        &physics_pack(),
        Settings::from_packs(&[place_pack()]).expect("settings"),
    )
    .expect("model")
}
pub fn systems() -> Vec<System> {
    serde_json::from_str(include_str!("../../../tests/fixtures/place/systems.json"))
        .expect("systems")
}
pub fn system(name: &str) -> System {
    systems()
        .into_iter()
        .find(|s| s.name == name)
        .expect("system")
}
pub fn seed() -> Seed {
    Seed::from_text("c3-placement")
}
pub fn address(sys: &str) -> Address {
    format!("u:place/sys:@{sys}/body:@new")
        .parse()
        .expect("address")
}

/// (system, scheme, class, scope) placements in the golden.
/// (system, scheme, class, scope in au).
pub type Case = (&'static str, &'static str, &'static str, Option<(f64, f64)>);
pub const CASES: [Case; 10] = [
    ("sun", "surface", "surface/temperate-ocean", None),
    (
        "sun",
        "surface",
        "surface/temperate-ocean",
        Some((30.0, 40.0)),
    ),
    ("sun", "physical", "gas-giant/ammonia-clouds", None),
    ("k-dwarf", "surface", "surface/cold-desert", None),
    ("k-dwarf", "physical", "water-rich", None),
    ("m-dwarf", "surface", "surface/airless", None),
    ("m-dwarf", "habitability", "habitable/pressure-suit", None),
    ("f-star", "surface", "surface/hot-greenhouse", None),
    ("f-star", "air", "air/carbon-dioxide", None),
    ("sun", "habitability", "habitable/shirt-sleeve", None),
];

pub fn run(
    p: &Placer,
    sys: &str,
    scheme: &str,
    class: &str,
    scope: Option<(f64, f64)>,
    force: bool,
) -> Placement {
    let req = Request {
        scheme: scheme.into(),
        class: class.into(),
        scope_au: scope,
        force,
    };
    p.place(&seed(), &address(sys), &system(sys), &req)
        .expect("placement")
}

pub fn golden() -> serde_json::Value {
    let atlas = sse_sift::scenario::base_atlas();
    let model = model();
    let life = sse_life::NoLifeModel::default();
    let rules = sse_life::scenario::base_rules();
    let p = Placer {
        atlas: &atlas,
        model: &model,
        life: &life,
        life_rules: &rules,
    };
    let cases: Vec<serde_json::Value> = CASES
        .iter()
        .map(|(s, sc, c, scope)| {
            let r = run(&p, s, sc, c, *scope, false);
            serde_json::json!({
                "system": s, "class": c, "outcome": r.outcome, "fit": r.fit, "label": r.label,
                "a_au": r.causes.as_ref().map(|c| c.a_au), "blocking": r.blocking, "interventions": r.interventions,
                "hash": sse_core::ContentHash::of(&r).expect("no NaN").hex(),
            })
        })
        .collect();
    serde_json::json!({ "cases": cases })
}
