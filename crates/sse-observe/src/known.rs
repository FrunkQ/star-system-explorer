//! "Known to" (track SV, secrets and visibility design 3 to 5, SV1): the
//! three landmarks every thing stands at for an audience (GM only, Heard of,
//! Known), read off the data view in force there; one-field exceptions per
//! thing; and one intent that sets either for many things at once as ONE
//! dated `observe.grant` event, so a scoped reveal is one undo.
//!
//! The rung is the data view (the pack's ladder of views, design 3). The
//! landmarks are three views: `gm_only` (the built-in [`GM_ONLY_VIEW`],
//! which shows nothing, unless the campaign names another), `heard_of` (the
//! lowest view that shows something) and `known` (the view that shows the
//! most tiers). A campaign may name any of them (`campaign.landmarks`).

use crate::view::{grant_field, Campaign, DataView, TagFilter, TagMode, UnknownStyle};
use serde::{Deserialize, Serialize};
use sse_chronicle::{
    kinds, Chronicle, ChronicleError, Event, EventBody, Evidence, Payload, Publisher, State,
    UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

/// The view that shows an audience nothing: the thing is not on their device.
pub const GM_ONLY_VIEW: &str = "gm-only";

/// Where a thing (or one fact on it) stands for an audience.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Landmark {
    GmOnly,
    HeardOf,
    Known,
}

impl Landmark {
    pub fn id(self) -> &'static str {
        match self {
            Landmark::GmOnly => "gm_only",
            Landmark::HeardOf => "heard_of",
            Landmark::Known => "known",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "gm_only" => Some(Landmark::GmOnly),
            "heard_of" => Some(Landmark::HeardOf),
            "known" => Some(Landmark::Known),
            _ => None,
        }
    }
}

/// The views a campaign names for its landmarks; any left out are found.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct LandmarkViews {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gm_only: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known: Option<String>,
}

/// The built-in view that shows nothing.
pub fn gm_only_view() -> DataView {
    DataView {
        id: GM_ONLY_VIEW.into(),
        label: "GM only".into(),
        show_tiers: vec![],
        tags: TagFilter {
            mode: TagMode::None,
            list: vec![],
        },
        overrides: vec![],
        unknown_style: UnknownStyle::Omit,
        show_honesty: false,
        description: "not known to exist".into(),
    }
}

fn shows_nothing(v: &DataView) -> bool {
    v.show_tiers.is_empty() && !v.overrides.iter().any(|o| o.show)
}

fn reach(v: &DataView) -> (usize, usize) {
    (
        v.show_tiers.len(),
        v.overrides.iter().filter(|o| o.show).count(),
    )
}

/// The three landmark views, resolved against the campaign's views.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Landmarks {
    pub gm_only: String,
    pub heard_of: String,
    pub known: String,
}

impl Landmarks {
    /// `views` in pack order. A named view must exist (checked at load).
    pub fn resolve(views: &[DataView], campaign: &Campaign) -> Landmarks {
        let named = campaign.landmarks.clone().unwrap_or_default();
        let seen: Vec<&DataView> = views.iter().filter(|v| !shows_nothing(v)).collect();
        let lowest = seen
            .iter()
            .fold(None::<&DataView>, |b, v| match b {
                Some(x) if reach(x) <= reach(v) => Some(x),
                _ => Some(v),
            })
            .map(|v| v.id.clone());
        let highest = seen
            .iter()
            .fold(None::<&DataView>, |b, v| match b {
                Some(x) if reach(x) >= reach(v) => Some(x),
                _ => Some(v),
            })
            .map(|v| v.id.clone());
        let fallback = campaign.default_view.clone();
        Landmarks {
            gm_only: named.gm_only.unwrap_or_else(|| GM_ONLY_VIEW.into()),
            heard_of: named
                .heard_of
                .or(lowest)
                .unwrap_or_else(|| fallback.clone()),
            known: named.known.or(highest).unwrap_or(fallback),
        }
    }

    /// The landmark a view stands at.
    pub fn of(&self, v: &DataView) -> Landmark {
        if v.id == self.gm_only || shows_nothing(v) {
            Landmark::GmOnly
        } else if v.id == self.known {
            Landmark::Known
        } else {
            Landmark::HeardOf
        }
    }

    pub fn view(&self, l: Landmark) -> &str {
        match l {
            Landmark::GmOnly => &self.gm_only,
            Landmark::HeardOf => &self.heard_of,
            Landmark::Known => &self.known,
        }
    }
}

/// The fact a one-field exception is kept in, on the thing itself
/// (internal, so never shown as a fact).
pub fn except_field(audience: &str, field: &str) -> String {
    format!("observe.except.{audience}.{field}")
}

/// A thing's one-field exceptions for `audience`: field to landmark.
pub fn exceptions(state: &State, a: &Address, audience: &str) -> BTreeMap<String, Landmark> {
    let p = format!("observe.except.{audience}.");
    state
        .facts
        .get(a)
        .into_iter()
        .flatten()
        .filter_map(|(f, v)| {
            let field = f.strip_prefix(&p)?;
            match v {
                Value::Text(t) => Landmark::parse(t).map(|l| (field.to_string(), l)),
                _ => None,
            }
        })
        .collect()
}

/// One change of a `set_known`: at `address`, the thing's own landmark
/// (`field` absent) or one field's exception. `None` takes the grant or the
/// exception away.
#[derive(Clone, PartialEq, Debug)]
pub struct KnownChange {
    pub address: Address,
    pub field: Option<String>,
    pub to: Option<Landmark>,
}

fn opt_text(v: Option<&Value>) -> Value {
    match v {
        Some(Value::Text(t)) => Value::Text(t.clone()),
        _ => Value::Null,
    }
}

/// Write `changes` for `audience` as ONE `observe.grant` event dated `at`,
/// reaching the audience at `from` (the record's activated_at): later for news
/// that waits on the campaign clock, earlier for "they have known since").
/// Each address's prior value is kept in the event, so [`undo_known`] can
/// put it back. Things already at the asked landmark are left alone.
/// Returns `None` when nothing would change.
#[allow(clippy::too_many_arguments)]
pub fn set_known(
    chronicle: &mut Chronicle,
    changes: &[KnownChange],
    audience: &str,
    marks: &Landmarks,
    views: &BTreeMap<String, DataView>,
    campaign: &Campaign,
    at: UniverseTime,
    from: UniverseTime,
    publisher: &Publisher,
) -> Result<Option<ContentHash>, ChronicleError> {
    let state = chronicle.state_at(from);
    let mut set: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
    let mut unset: BTreeMap<Address, Vec<String>> = BTreeMap::new();
    let mut prior = Vec::new();
    let mut subject = Vec::new();
    for c in changes {
        let key = match &c.field {
            Some(f) => except_field(audience, f),
            None => grant_field(audience),
        };
        let before = state.get(&c.address, &key);
        let want: Option<Value> = match (&c.field, c.to) {
            (_, None) => None,
            (Some(_), Some(l)) => Some(Value::Text(l.id().into())),
            (None, Some(l)) => {
                let now = crate::view::effective_view(&state, &c.address, audience, campaign);
                let here = views.get(&now).map(|v| marks.of(v));
                let here = if now == GM_ONLY_VIEW {
                    Some(Landmark::GmOnly)
                } else {
                    here
                };
                if here == Some(l) && before.is_some() {
                    continue;
                }
                Some(Value::Text(marks.view(l).into()))
            }
        };
        if want.as_ref() == before {
            continue;
        }
        prior.push(Value::Map(
            [
                ("address".to_string(), Value::Text(c.address.to_string())),
                ("key".to_string(), Value::Text(key.clone())),
                ("was".to_string(), opt_text(before)),
            ]
            .into(),
        ));
        match want {
            Some(v) => {
                set.entry(c.address.clone()).or_default().insert(key, v);
            }
            None => unset.entry(c.address.clone()).or_default().push(key),
        }
        if !subject.contains(&c.address) {
            subject.push(c.address.clone());
        }
    }
    if subject.is_empty() {
        return Ok(None);
    }
    subject.sort();
    write(
        chronicle,
        subject,
        set,
        unset,
        audience,
        prior,
        None,
        at.min(from),
        from,
        publisher,
    )
    .map(Some)
}

#[allow(clippy::too_many_arguments)]
fn write(
    chronicle: &mut Chronicle,
    subject: Vec<Address>,
    set: BTreeMap<Address, BTreeMap<String, Value>>,
    unset: BTreeMap<Address, Vec<String>>,
    audience: &str,
    prior: Vec<Value>,
    undoes: Option<ContentHash>,
    occurred: UniverseTime,
    from: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, ChronicleError> {
    let mut data = BTreeMap::new();
    data.insert("audience".to_string(), Value::Text(audience.into()));
    data.insert("known".to_string(), Value::Array(prior));
    if let Some(u) = undoes {
        data.insert("undoes".to_string(), Value::Text(u.hex()));
    }
    let e = Event::new(EventBody {
        kind: kinds::OBSERVE_GRANT.into(),
        subject,
        occurred_at: occurred,
        activated_at: from,
        recognised_at: chronicle.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            unset,
            data: Some(Value::Map(data)),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    chronicle.append(e)
}

/// Why an undo is refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum UndoError {
    NoSuchChange,
    Chronicle(String),
}

/// Put back what a `set_known` event changed, as one more event at `at`,
/// or when the original reaches the audience if that is later (so an undo
/// before a future-dated reveal arrives means it never arrives).
pub fn undo_known(
    chronicle: &mut Chronicle,
    id: &ContentHash,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, UndoError> {
    let e = chronicle
        .events()
        .iter()
        .find(|e| &e.id == id && e.body.kind == kinds::OBSERVE_GRANT)
        .ok_or(UndoError::NoSuchChange)?;
    let Some(Value::Map(d)) = &e.body.payload.data else {
        return Err(UndoError::NoSuchChange);
    };
    let (Some(Value::Text(audience)), Some(Value::Array(known))) =
        (d.get("audience"), d.get("known"))
    else {
        return Err(UndoError::NoSuchChange);
    };
    let audience = audience.clone();
    let from = e.body.activated_at;
    let mut set: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
    let mut unset: BTreeMap<Address, Vec<String>> = BTreeMap::new();
    let mut prior = Vec::new();
    let mut subject = Vec::new();
    for k in known {
        let Value::Map(k) = k else { continue };
        let (Some(Value::Text(a)), Some(Value::Text(key))) = (k.get("address"), k.get("key"))
        else {
            continue;
        };
        let Ok(a) = a.parse::<Address>() else {
            continue;
        };
        let now = e.body.payload.set.get(&a).and_then(|m| m.get(key));
        prior.push(Value::Map(
            [
                ("address".to_string(), Value::Text(a.to_string())),
                ("key".to_string(), Value::Text(key.clone())),
                ("was".to_string(), opt_text(now)),
            ]
            .into(),
        ));
        match k.get("was") {
            Some(Value::Text(v)) => {
                set.entry(a.clone())
                    .or_default()
                    .insert(key.clone(), Value::Text(v.clone()));
            }
            _ => unset.entry(a.clone()).or_default().push(key.clone()),
        }
        if !subject.contains(&a) {
            subject.push(a);
        }
    }
    subject.sort();
    let id = *id;
    write(
        chronicle,
        subject,
        set,
        unset,
        &audience,
        prior,
        Some(id),
        at,
        at.max(from),
        publisher,
    )
    .map_err(|e| UndoError::Chronicle(format!("{e}")))
}
