//! Scene building: plain, serialisable data made from state after the data
//! view's filter (H8.1). Views see only this; they never read kernel state.

use crate::view::{
    effective_view_tagged, Campaign, DataView, Disclosure, Precision, TagMode, Target, UnknownStyle,
};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Evidence, State};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::{Address, ContentHash};
use std::collections::{BTreeMap, BTreeSet};

/// The audience with all knowledge: the GM's seat (H8.8).
pub const GM: &str = "gm";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SceneFact {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    /// A shown range instead of a value (band precision).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band: Option<(f64, f64)>,
    /// Text shown in place of a value the audience has not been given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Honesty marks: `authored`, `impossible`, `fictional`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SceneSubject {
    pub address: Address,
    /// The data view applied (`gm` for the GM seat).
    pub view: String,
    pub facts: Vec<SceneFact>,
    pub tags: Vec<String>,
    /// Record notes at or linked to this address that the audience may see
    /// (J1). Left out when none, so scenes without notes hash as before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<SceneNote>,
    /// Something undisclosed here (tags design TG5): at least one tag sits
    /// one rung above what the view shows. One neutral marker per object,
    /// however many tags are hidden: no id, colour or count. Tags further
    /// up, and GM-only tags, give nothing.
    #[serde(default, skip_serializing_if = "is_false")]
    pub undisclosed: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// The authored fact that moves one tag on one object to another tier
/// (`observe.tag_tier.<tag>`, a tier name): the per-object override. It is
/// an `observe.*` field, so no scene shows it.
pub const TAG_TIER_PREFIX: &str = "observe.tag_tier.";

/// A note as a scene shows it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SceneNote {
    /// The note's event id, on the GM's scene only: a player's scene never
    /// carries record event ids (record design 8A).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub at_s: i64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub by: String,
    /// Linked addresses beyond this one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Address>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub audience: String,
    pub scope: Address,
    pub subjects: Vec<SceneSubject>,
}
impl Scene {
    pub fn hash(&self) -> ContentHash {
        ContentHash::of(self).expect("scenes hold no NaN")
    }
}

/// Everything a scene is built from. All read-only.
pub struct SceneInput<'a> {
    pub state: &'a State,
    /// For honesty marks (who set each fact); optional.
    pub chronicle: Option<&'a Chronicle>,
    pub tags: &'a BTreeMap<Address, BTreeSet<String>>,
    /// Fields published downstream of a fictional pack (K5 labels).
    pub fictional: &'a BTreeSet<String>,
    pub disclosure: &'a Disclosure,
    pub views: &'a BTreeMap<String, DataView>,
    pub campaign: &'a Campaign,
    /// The live record notes at the scene's time (`sse_record::notes_at`);
    /// empty for none. Each is shown at every address it names, to the
    /// audiences it is for, where their view shows its tier.
    pub notes: &'a [sse_record::Note],
}

fn sig_round(x: f64, digits: u8) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let e = m::floor(m::log10(x.abs())) + 1.0 - digits as f64;
    let p = m::pow(10.0, e);
    m::round(x / p) * p
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

fn marks_for(inp: &SceneInput, a: &Address, field: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(c) = inp.chronicle {
        if let Some(e) = c.events().iter().rev().find(|e| {
            e.body
                .payload
                .set
                .get(a)
                .is_some_and(|f| f.contains_key(field))
        }) {
            match e.body.evidence {
                Evidence::Authored => out.push("authored".to_string()),
                Evidence::AuthoredImpossible => out.push("impossible".to_string()),
                _ => {}
            }
        }
    }
    if inp.fictional.contains(field) {
        out.push("fictional".into());
    }
    out
}

/// The GM's own view: every tier, every tag, marks always on.
fn gm_view(d: &Disclosure) -> DataView {
    DataView {
        id: GM.into(),
        label: "Everything".into(),
        show_tiers: d.ladder.rungs.iter().map(|(t, _)| t.clone()).collect(),
        tags: crate::view::TagFilter {
            mode: TagMode::All,
            list: vec![],
        },
        overrides: vec![],
        unknown_style: UnknownStyle::Omit,
        show_honesty: true,
        description: String::new(),
    }
}

fn subject(inp: &SceneInput, a: &Address, v: &DataView, audience: &str) -> Option<SceneSubject> {
    let d = inp.disclosure;
    let shown_rank = v.show_tiers.iter().filter_map(|t| d.ladder.rank(t)).max();
    let next_rank = shown_rank.map_or(0, |r| r + 1);
    let secret_rank = d.ladder.rungs.len() - 1;
    let field_override = |f: &str| {
        v.overrides
            .iter()
            .find(|o| o.target == Target::Field(f.into()))
    };
    let mut facts = Vec::new();
    let mut real = 0;
    for (f, val) in inp.state.facts.get(a).into_iter().flatten() {
        if d.is_internal(f) {
            continue;
        }
        let tier = d.field_tier(f);
        let ov = field_override(f);
        let precision = ov
            .and_then(|o| o.precision.clone())
            .unwrap_or(Precision::Exact);
        let shown = match ov {
            Some(o) if !o.show => false,
            Some(o) if o.show => true,
            _ => v.show_tiers.iter().any(|t| t == tier),
        } && precision != Precision::Hidden;
        let marks = if v.show_honesty {
            marks_for(inp, a, f)
        } else {
            Vec::new()
        };
        if shown {
            real += 1;
            let (value, band) = match (&precision, num(val)) {
                (Precision::Rounded { digits }, Some(x)) => {
                    (Some(Value::Float(sig_round(x, *digits))), None)
                }
                (Precision::Band { step, half_width }, Some(x)) => {
                    let c = m::round(x / step) * step;
                    (None, Some((c - half_width, c + half_width)))
                }
                _ => (Some(val.clone()), None),
            };
            facts.push(SceneFact {
                field: f.clone(),
                value,
                band,
                placeholder: None,
                marks,
            });
        } else if v.unknown_style == UnknownStyle::Placeholder
            && d.ladder.rank(tier) == Some(next_rank)
            && next_rank < secret_rank
            && ov.is_none_or(|o| o.show)
        {
            facts.push(SceneFact {
                field: f.clone(),
                value: None,
                band: None,
                placeholder: Some(d.ladder.placeholder(tier).into()),
                marks: Vec::new(),
            });
        }
    }
    // Tags: the one redaction point (TG5). A tag sits at a tier exactly as a
    // fact does: its declared tier, or this object's own override.
    let tag_tier = |t: &str| -> &str {
        match inp.state.get(a, &format!("{TAG_TIER_PREFIX}{t}")) {
            Some(Value::Text(x)) if d.ladder.rank(x).is_some() => x.as_str(),
            _ => d.tag_tier(t),
        }
    };
    // Shown, held back one rung (the marker), or nothing.
    let tag_seen = |t: &String| -> (bool, bool) {
        let ov = v
            .overrides
            .iter()
            .find(|o| o.target == Target::Tag(t.clone()));
        if let Some(o) = ov {
            return (o.show, false);
        }
        let listed = match v.tags.mode {
            TagMode::All => true,
            TagMode::None => false,
            TagMode::Listed => v.tags.list.contains(t),
        };
        let tier = tag_tier(t);
        if v.show_tiers.iter().any(|x| x == tier) {
            return (listed, false);
        }
        let next = v.unknown_style == UnknownStyle::Placeholder
            && d.ladder.rank(tier) == Some(next_rank)
            && next_rank < secret_rank;
        (false, listed && next)
    };
    let mut undisclosed = false;
    let tags: Vec<String> = inp
        .tags
        .get(a)
        .into_iter()
        .flatten()
        .filter(|t| {
            let (shown, held) = tag_seen(t);
            undisclosed |= held;
            shown
        })
        .cloned()
        .collect();
    let notes: Vec<SceneNote> = inp
        .notes
        .iter()
        .filter(|n| n.about(a) && n.visible_to(audience, Some(&v.show_tiers)))
        .map(|n| SceneNote {
            id: (audience == GM).then(|| n.id.clone()),
            at_s: n.at_s,
            text: n.text.clone(),
            // A note's tags pass the same filter as the subject's where the
            // tag has a tier (declared, or this object's override). A tag
            // with no tier anywhere, such as one a seat typed on its own
            // note, goes where the note goes.
            tags: n
                .tags
                .iter()
                .filter(|t| {
                    let tiered = d.tags.contains_key(*t)
                        || inp.state.get(a, &format!("{TAG_TIER_PREFIX}{t}")).is_some();
                    !tiered || tag_seen(t).0
                })
                .cloned()
                .collect(),
            by: n.by.clone(),
            links: n.subjects.iter().filter(|s| *s != a).cloned().collect(),
        })
        .collect();
    // Nothing shown: the audience does not learn the subject exists.
    (real > 0 || !tags.is_empty() || !notes.is_empty()).then(|| SceneSubject {
        address: a.clone(),
        view: v.id.clone(),
        facts,
        tags,
        notes,
        undisclosed,
    })
}

/// Build the scene for `audience` over everything at or under `scope`.
/// `audience == "gm"` gives the GM's all-knowledge scene with honesty marks.
/// Subjects come in address order; facts in field order.
pub fn build_scene(inp: &SceneInput, audience: &str, scope: &Address) -> Result<Scene, String> {
    build_scene_through(inp, audience, scope, None)
}

/// As `build_scene`, but with `lens` set every subject is shown through that
/// one data view, whatever the grants say (the GM looking through a player
/// lens, or a preview of what a view would show).
pub fn build_scene_through(
    inp: &SceneInput,
    audience: &str,
    scope: &Address,
    lens: Option<&str>,
) -> Result<Scene, String> {
    let gm = gm_view(inp.disclosure);
    let forced = match lens {
        Some(id) => Some(
            inp.views
                .get(id)
                .ok_or_else(|| format!("data view {id:?} is not defined"))?,
        ),
        None => None,
    };
    let mut subjects = Vec::new();
    // Every address with facts, plus any a note names (an actor or place
    // known only from the record).
    let mut addrs: BTreeSet<&Address> = inp.state.facts.keys().collect();
    addrs.extend(inp.notes.iter().flat_map(|n| n.subjects.iter()));
    for a in addrs.into_iter().filter(|a| scope.contains(a)) {
        let v = if let Some(f) = forced {
            f
        } else if audience == GM {
            &gm
        } else {
            let id = effective_view_tagged(inp.state, inp.tags, a, audience, inp.campaign);
            inp.views
                .get(&id)
                .ok_or_else(|| format!("{a}: data view {id:?} is not defined"))?
        };
        subjects.extend(subject(inp, a, v, audience));
    }
    Ok(Scene {
        audience: audience.into(),
        scope: scope.clone(),
        subjects,
    })
}

/// What changed between two scenes (the payload of a `subscribe` stream).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct SceneDelta {
    pub upsert: Vec<SceneSubject>,
    pub remove: Vec<Address>,
}

pub fn diff(old: &Scene, new: &Scene) -> SceneDelta {
    let o: BTreeMap<&Address, &SceneSubject> =
        old.subjects.iter().map(|s| (&s.address, s)).collect();
    let n: BTreeMap<&Address, &SceneSubject> =
        new.subjects.iter().map(|s| (&s.address, s)).collect();
    SceneDelta {
        upsert: n
            .iter()
            .filter(|(a, s)| o.get(*a) != Some(*s))
            .map(|(_, s)| (*s).clone())
            .collect(),
        remove: o
            .keys()
            .filter(|a| !n.contains_key(*a))
            .map(|a| (*a).clone())
            .collect(),
    }
}
