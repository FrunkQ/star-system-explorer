//! Holdings (track OV, territory overlays design 4.3 to 4.5): who holds,
//! claims or influences a place, on a map layer, from a date.
//!
//! A holding is one `record.hold` event about a place and an answer (an
//! actor's address, or any short key a layer lists). It carries a status,
//! an optional strength and extent, who asserts it, and a tier and audience
//! like any kept value. Nothing is edited in place: a later event in the same
//! slot replaces it, and a withdrawal ends it. Holdings live in the event's
//! `data` only, never in the fold's facts, so no physics flow can read them.
//!
//! Reading a layer has three rules, the same at every scale:
//! - **inherit down**: a holding covers everything inside its place, and a
//!   more specific holding wins inside its own place;
//! - **roll up**: a place with differently held parts takes the answer that
//!   holds most of it by the layer's weight, or reads as divided when no
//!   answer reaches the divided share (0.6);
//! - **reach**: a place no address rule covers takes the nearest held place
//!   within the layer's reach (or the holding's own reach), so borders between
//!   stars run midway and the void stays bare.

use crate::{field, kinds, Audience, RecordError, GM_WRITER};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

/// Below this share of a place, no answer holds it: it is divided (ruling 5).
pub const DIVIDED_BELOW: f64 = 0.6;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Control: one holder per place per layer at a time.
    Holds,
    /// An assertion by `by`; any number may overlap.
    Claims,
    /// A soft presence with a strength.
    Influences,
}

impl Status {
    fn word(self) -> &'static str {
        match self {
            Status::Holds => "holds",
            Status::Claims => "claims",
            Status::Influences => "influences",
        }
    }
    fn parse(s: &str) -> Option<Status> {
        match s {
            "holds" => Some(Status::Holds),
            "claims" => Some(Status::Claims),
            "influences" => Some(Status::Influences),
            _ => None,
        }
    }
}

/// A distance with its unit, as the sheet's units name it (`pc`, `au`, `km`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Distance {
    pub value: f64,
    pub unit: String,
}

impl Distance {
    /// In metres, or an error for a unit that is not a length.
    pub fn metres(&self) -> Result<f64, String> {
        let u = sse_core::Units::standard();
        let unit = u
            .get(&self.unit)
            .map_err(|_| format!("unknown unit {:?}", self.unit))?;
        if unit.dim != sse_core::Dimension::LENGTH {
            return Err(format!("{:?} is not a length", self.unit));
        }
        if !self.value.is_finite() || self.value < 0.0 {
            return Err("a distance is a finite number, 0 or more".into());
        }
        Ok(self.value * unit.scale)
    }
}

/// Where a holding reaches when its place alone is not enough (design 4.3).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Extent {
    /// Everything within a distance of the place.
    Reach { reach: Distance },
    /// Between two distances from the place.
    Band { band: [Distance; 2] },
    /// A drawn shape in the place's own frame (`body-latlon`: x longitude
    /// east, y latitude north, degrees; `galaxy-plane-pc`, ...).
    Outline {
        outline: Vec<[f64; 2]>,
        frame: String,
    },
}

/// The frames an outline may be drawn in (the overlay contract's).
pub const FRAMES: [&str; 5] = [
    "galaxy-plane-pc",
    "cluster-pc",
    "system-au",
    "body-latlon",
    "web-mpc",
];

impl Extent {
    fn check(&self) -> Result<(), String> {
        match self {
            Extent::Reach { reach } => reach.metres().map(|_| ()),
            Extent::Band { band } => {
                let (a, b) = (band[0].metres()?, band[1].metres()?);
                if a >= b {
                    return Err("a band's inner edge must be inside its outer edge".into());
                }
                Ok(())
            }
            Extent::Outline { outline, frame } => {
                if !FRAMES.contains(&frame.as_str()) {
                    return Err(format!("frame {frame:?}: one of {}", FRAMES.join(", ")));
                }
                if outline.len() < 3 || outline.iter().flatten().any(|v| !v.is_finite()) {
                    return Err("an outline needs at least three finite points".into());
                }
                Ok(())
            }
        }
    }
}

/// A holding to write.
#[derive(Clone, PartialEq, Debug)]
pub struct HoldDraft {
    pub place: Address,
    /// The map layer (an overlay id).
    pub layer: String,
    /// The answer: an actor's address, or a tag, choice or band key.
    pub key: String,
    pub status: Status,
    pub strength: Option<f64>,
    pub extent: Option<Extent>,
    /// Who asserts it (a claim's claimant); absent means it is simply so.
    pub by: Option<String>,
    pub tier: Option<String>,
    pub audience: Audience,
    /// The seat or rule that wrote it.
    pub writer: String,
    pub reason: Option<String>,
    /// The relation record or rule the holding comes from, for "Why?".
    pub source: Option<String>,
    /// End the holding in this slot instead of starting one.
    pub withdraw: bool,
}

/// A holding as the record reads it back (the overlay contract's `Holding`
/// before the rules: `via` is `held`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Holding {
    pub place: Address,
    pub layer: String,
    pub key: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extent: Option<Extent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub since_s: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_s: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    pub audience: Audience,
    pub writer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The event that wrote it.
    pub event: String,
}

impl Holding {
    /// The slot a later event replaces: one holder per place, one claim per
    /// claimant, one influence per answer.
    fn slot(&self) -> (Address, String, Status, String) {
        let who = match self.status {
            Status::Holds => String::new(),
            Status::Claims => self.by.clone().unwrap_or_else(|| self.key.clone()),
            Status::Influences => self.key.clone(),
        };
        (self.place.clone(), self.layer.clone(), self.status, who)
    }
    /// May this audience see it? The GM sees all; others need the audience
    /// and an untiered or `obvious` holding (data views refine this in OV9).
    pub fn visible_to(&self, audience: &str) -> bool {
        audience == GM_WRITER
            || (self.audience.includes(audience)
                && self.tier.as_deref().is_none_or(|t| t == "obvious"))
    }
}

fn text(v: &str) -> Value {
    Value::Text(v.into())
}

/// Write a holding (or withdraw one): one `record.hold` event.
pub fn hold(
    ch: &mut Chronicle,
    d: &HoldDraft,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, RecordError> {
    let bad = |w: String| RecordError::BadValue {
        kind: "holding".into(),
        why: w,
    };
    if d.layer.is_empty() || !crate::ident_ok(&d.layer) {
        return Err(bad(format!("layer {:?}: a lowercase overlay id", d.layer)));
    }
    if d.key.trim().is_empty() {
        return Err(bad("a holding needs an answer (key)".into()));
    }
    if d.writer.is_empty() {
        return Err(bad("no writer".into()));
    }
    if let Some(s) = d.strength {
        if !(0.0..=1.0).contains(&s) {
            return Err(bad("strength is 0 to 1".into()));
        }
    }
    if d.status == Status::Claims && d.by.is_none() && !d.key.contains("actor:") {
        return Err(bad("a claim names who makes it (by)".into()));
    }
    if let Some(e) = &d.extent {
        e.check().map_err(bad)?;
    }
    let audience = d.audience.normal().map_err(bad)?;
    let mut m = BTreeMap::new();
    m.insert("layer".to_string(), text(&d.layer));
    m.insert("key".into(), text(&d.key));
    m.insert("status".into(), text(d.status.word()));
    if let Some(s) = d.strength {
        m.insert("strength".into(), Value::Float(s));
    }
    if let Some(e) = &d.extent {
        m.insert(
            "extent".into(),
            text(&serde_json::to_string(e).expect("plain")),
        );
    }
    if let Some(b) = &d.by {
        m.insert("by".into(), text(b));
    }
    if let Some(t) = &d.tier {
        m.insert("tier".into(), text(t));
    }
    m.insert("audience".into(), audience.to_value());
    m.insert("writer".into(), text(&d.writer));
    if let Some(r) = &d.reason {
        m.insert("reason".into(), text(r));
    }
    if let Some(s) = &d.source {
        m.insert("source".into(), text(s));
    }
    if d.withdraw {
        m.insert("withdraw".into(), Value::Bool(true));
    }
    let mut subject = vec![d.place.clone()];
    if let Ok(a) = d.key.parse::<Address>() {
        subject.push(a);
    }
    let e = Event::new(EventBody {
        kind: kinds::HOLD.into(),
        subject,
        occurred_at: at,
        activated_at: at,
        recognised_at: ch.next_stamp(),
        causes: vec![],
        payload: Payload {
            data: Some(Value::Map(m)),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    Ok(ch.append(e)?)
}

/// Read one `record.hold` event: the holding, and whether it withdraws.
fn holding_of(e: &Event) -> Option<(Holding, bool)> {
    if e.body.kind != kinds::HOLD {
        return None;
    }
    let Some(Value::Map(m)) = &e.body.payload.data else {
        return None;
    };
    let h = Holding {
        place: e.body.subject.first()?.clone(),
        layer: field(m, "layer")?.to_string(),
        key: field(m, "key")?.to_string(),
        status: Status::parse(field(m, "status")?)?,
        strength: match m.get("strength") {
            Some(Value::Float(f)) => Some(*f),
            _ => None,
        },
        extent: field(m, "extent").and_then(|j| serde_json::from_str(j).ok()),
        by: field(m, "by").map(str::to_string),
        since_s: e.body.occurred_at.s,
        until_s: None,
        tier: field(m, "tier").map(str::to_string),
        audience: m
            .get("audience")
            .and_then(Audience::from_value)
            .unwrap_or_else(Audience::gm),
        writer: field(m, "writer").unwrap_or("").to_string(),
        reason: field(m, "reason").map(str::to_string),
        source: field(m, "source").map(str::to_string),
        event: e.id.hex(),
    };
    Some((h, matches!(m.get("withdraw"), Some(Value::Bool(true)))))
}

/// Every holding on `layer` that started by `at`, in fold order, with
/// `until_s` on those a later event (also by `at`) replaced or withdrew.
pub fn history<'a>(
    events: impl IntoIterator<Item = &'a Event>,
    layer: &str,
    at: UniverseTime,
) -> Vec<Holding> {
    let mut sel: Vec<&Event> = events
        .into_iter()
        .filter(|e| e.body.kind == kinds::HOLD && e.body.activated_at <= at)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut out: Vec<Holding> = vec![];
    let mut live: BTreeMap<(Address, String, Status, String), usize> = BTreeMap::new();
    for e in sel {
        let Some((h, withdraw)) = holding_of(e) else {
            continue;
        };
        if h.layer != layer {
            continue;
        }
        let slot = h.slot();
        if let Some(i) = live.remove(&slot) {
            out[i].until_s = Some(h.since_s);
        }
        if !withdraw {
            live.insert(slot, out.len());
            out.push(h);
        }
    }
    out
}

/// The holdings on `layer` in force at `at`.
pub fn holdings_at<'a>(
    events: impl IntoIterator<Item = &'a Event>,
    layer: &str,
    at: UniverseTime,
) -> Vec<Holding> {
    history(events, layer, at)
        .into_iter()
        .filter(|h| h.until_s.is_none())
        .collect()
}

/// Whose map is read (design 8): an audience, and optionally a power whose
/// claims are drawn as if held.
#[derive(Clone, PartialEq, Debug)]
pub struct View {
    pub audience: String,
    pub according_to: Option<String>,
}

impl View {
    pub fn gm() -> Self {
        View {
            audience: GM_WRITER.into(),
            according_to: None,
        }
    }
}

/// A layer's rules for reading (from its pack entry, design 4.1).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct LayerRules {
    /// Between places: how far a held place colours space (metres); none
    /// means places colour only themselves.
    pub reach_m: Option<f64>,
    /// The kept value that weighs a place when rolling up (`population`);
    /// none counts places.
    pub weigh_by: Option<String>,
}

/// The holdings a view sees on a layer, as control: `holds`, plus, for a
/// power's own map, its claims as if held. Claims and influences are kept
/// apart for the "also" lines.
pub struct Seen {
    pub holds: Vec<Holding>,
    pub others: Vec<Holding>,
}

impl Seen {
    pub fn of(live: &[Holding], view: &View) -> Seen {
        let mut holds: BTreeMap<Address, Holding> = BTreeMap::new();
        let mut others = vec![];
        for h in live.iter().filter(|h| h.visible_to(&view.audience)) {
            if h.status == Status::Holds {
                holds.entry(h.place.clone()).or_insert_with(|| h.clone());
            } else {
                others.push(h.clone());
            }
        }
        if let Some(p) = &view.according_to {
            for c in others.iter().filter(|c| {
                c.status == Status::Claims && c.by.as_deref().unwrap_or(&c.key) == p.as_str()
            }) {
                let mut h = c.clone();
                h.status = Status::Holds;
                holds.insert(h.place.clone(), h);
            }
        }
        Seen {
            holds: holds.into_values().collect(),
            others,
        }
    }
}

/// How an answer was reached (the contract's `via`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Via {
    Held,
    Inherited,
    RolledUp,
    Rule,
}

/// One part of a divided place.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Share {
    pub key: String,
    pub share: f64,
}

/// The answer at a place (the overlay contract's `Holding`, design 4.5's
/// `who_holds`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Answer {
    pub place: Address,
    pub layer: String,
    pub key: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extent: Option<Extent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub since_s: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_s: Option<i64>,
    pub via: Via,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub divided: Vec<Share>,
    /// The holding's own place, when it is not this one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Claims and influences on the place itself or what holds it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also: Vec<Holding>,
    pub why: String,
}

fn answer(h: &Holding, place: &Address, via: Via, why: String) -> Answer {
    Answer {
        place: place.clone(),
        layer: h.layer.clone(),
        key: h.key.clone(),
        status: h.status,
        strength: h.strength,
        extent: h.extent.clone(),
        by: h.by.clone(),
        since_s: h.since_s,
        until_s: h.until_s,
        via,
        divided: vec![],
        from: (h.place != *place).then(|| h.place.clone()),
        source: h.source.clone(),
        also: vec![],
        why,
    }
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    sse_core::math::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
}

/// What the reading rules need from the world: where a place is (metres,
/// in one frame), and how much it weighs.
pub trait World {
    fn position(&self, place: &Address) -> Option<[f64; 3]>;
    fn weight(&self, place: &Address, kind: &str) -> Option<f64>;
}

/// A world with no positions and no weights: address rules only.
pub struct NoWorld;
impl World for NoWorld {
    fn position(&self, _: &Address) -> Option<[f64; 3]> {
        None
    }
    fn weight(&self, _: &Address, _: &str) -> Option<f64> {
        None
    }
}

fn fmt_m(m: f64) -> String {
    let pc = sse_core::sheet::PC;
    let au = sse_core::sheet::AU;
    if m >= 0.01 * pc {
        format!("{:.1} pc", m / pc)
    } else if m >= 0.01 * au {
        format!("{:.2} au", m / au)
    } else {
        format!("{:.0} km", m / 1000.0)
    }
}

/// The answer from address and reach rules alone, before rolling up:
/// the deepest holding on the place or above it, then a holding's own
/// extent, then the layer's reach.
fn direct(place: &Address, seen: &Seen, rules: &LayerRules, world: &dyn World) -> Option<Answer> {
    // Inherit down: the deepest held place that contains this one.
    if let Some(h) = seen
        .holds
        .iter()
        .filter(|h| h.place.contains(place) && h.extent.is_none())
        .max_by_key(|h| h.place.depth())
    {
        if h.place == *place {
            let why = format!("{} holds {} since {} s", h.key, place, h.since_s);
            return Some(answer(h, place, Via::Held, why));
        }
        let why = format!(
            "{} holds {}, which contains {} (a holding covers everything inside its place)",
            h.key, h.place, place
        );
        return Some(answer(h, place, Via::Inherited, why));
    }
    // A holding's own extent, then the layer's reach: nearest wins, so a
    // border between two held places runs midway.
    let here = world.position(place)?;
    let mut best: Option<(f64, &Holding, String)> = None;
    for h in &seen.holds {
        let Some(at) = world.position(&h.place) else {
            continue;
        };
        let d = dist(here, at);
        let reach = match &h.extent {
            Some(Extent::Reach { reach }) => reach.metres().ok().map(|r| (0.0, r)),
            Some(Extent::Band { band }) => match (band[0].metres(), band[1].metres()) {
                (Ok(a), Ok(b)) => Some((a, b)),
                _ => None,
            },
            Some(Extent::Outline { .. }) => None,
            None => rules.reach_m.map(|r| (0.0, r)),
        };
        let Some((lo, hi)) = reach else {
            continue;
        };
        if d < lo || d > hi {
            continue;
        }
        let better = match &best {
            None => true,
            Some((bd, bh, _)) => d < *bd || (d == *bd && h.place < bh.place),
        };
        if better {
            let how = match &h.extent {
                Some(Extent::Band { .. }) => {
                    format!("inside the band {} to {} round it", fmt_m(lo), fmt_m(hi))
                }
                Some(_) => format!("within its reach of {}", fmt_m(hi)),
                None => format!("within the layer's reach of {}", fmt_m(hi)),
            };
            best = Some((d, h, how));
        }
    }
    let (d, h, how) = best?;
    let why = format!(
        "{} holds {}, {} away and the nearest held place, {how}",
        h.key,
        h.place,
        fmt_m(d)
    );
    Some(answer(h, place, Via::Rule, why))
}

/// Who holds `place` on a layer, with "Why?" (design 4.4 and 4.5). Places
/// whose parts are held by others roll up by weight, and read as divided
/// below the divided share.
pub fn who_holds(
    place: &Address,
    live: &[Holding],
    view: &View,
    rules: &LayerRules,
    world: &dyn World,
) -> Option<Answer> {
    let seen = Seen::of(live, view);
    let here = direct(place, &seen, rules, world);
    // Held parts strictly inside the place: the top-level ones count, each
    // with its own weight; what they leave is the place's own answer's.
    let mut parts: Vec<&Holding> = seen
        .holds
        .iter()
        .filter(|h| place.contains(&h.place) && h.place != *place)
        .collect();
    parts.sort_by(|a, b| a.place.cmp(&b.place));
    let mut out = if parts.is_empty() {
        here
    } else {
        Some(roll_up(place, here, &parts, &seen, rules, world))
    }?;
    // Claims and influences on this place or on what holds it.
    let anchor = out.from.clone().unwrap_or_else(|| place.clone());
    out.also = seen
        .others
        .iter()
        .filter(|o| o.place.contains(place) || o.place == anchor)
        .cloned()
        .collect();
    Some(out)
}

fn roll_up(
    place: &Address,
    here: Option<Answer>,
    parts: &[&Holding],
    seen: &Seen,
    rules: &LayerRules,
    world: &dyn World,
) -> Answer {
    let weigh = |a: &Address| -> Option<f64> {
        rules
            .weigh_by
            .as_deref()
            .and_then(|k| world.weight(a, k))
            .filter(|w| w.is_finite() && *w >= 0.0)
    };
    // Each held part's exclusive weight: its own (or one, counted) less
    // what its own held parts take.
    let mut shares: BTreeMap<String, f64> = BTreeMap::new();
    let mut inner = 0.0;
    for p in parts {
        let below: f64 = parts
            .iter()
            .filter(|q| p.place.contains(&q.place) && q.place != p.place)
            .filter(|q| {
                // Only the parts directly under p (no held place between).
                !parts.iter().any(|r| {
                    r.place != p.place
                        && r.place != q.place
                        && p.place.contains(&r.place)
                        && r.place.contains(&q.place)
                })
            })
            .map(|q| weigh(&q.place).unwrap_or(1.0))
            .sum();
        let own = weigh(&p.place).unwrap_or(1.0);
        let top = !parts
            .iter()
            .any(|r| r.place != p.place && r.place.contains(&p.place));
        if top {
            inner += own;
        }
        let w = match weigh(&p.place) {
            Some(x) => (x - below).max(0.0),
            None => 1.0,
        };
        *shares.entry(p.key.clone()).or_default() += w;
    }
    if let Some(h) = &here {
        let rest = match weigh(place) {
            Some(x) => (x - inner).max(0.0),
            None => 1.0,
        };
        *shares.entry(h.key.clone()).or_default() += rest;
    }
    let total: f64 = shares.values().sum();
    let mut list: Vec<Share> = shares
        .into_iter()
        .map(|(key, w)| Share {
            key,
            share: if total > 0.0 { w / total } else { 0.0 },
        })
        .collect();
    list.sort_by(|a, b| b.share.total_cmp(&a.share).then(a.key.cmp(&b.key)));
    let top = list[0].clone();
    let by_what = match &rules.weigh_by {
        Some(k) => format!("by {k}"),
        None => "by count of held places".to_string(),
    };
    // The answer's record: the place's own holding when it is the top
    // answer, else the top answer's outermost held part.
    let own = here.as_ref().filter(|h| h.key == top.key).and_then(|h| {
        let at = h.from.clone().unwrap_or_else(|| place.clone());
        seen.holds.iter().find(|x| x.place == at && x.key == h.key)
    });
    let rec = own
        .or_else(|| {
            parts
                .iter()
                .copied()
                .filter(|p| p.key == top.key)
                .min_by(|a, b| {
                    a.place
                        .depth()
                        .cmp(&b.place.depth())
                        .then(a.place.cmp(&b.place))
                })
        })
        .expect("the top share has a holding");
    let mut a = answer(rec, place, Via::RolledUp, String::new());
    if top.share < DIVIDED_BELOW {
        a.why = format!(
            "divided: no one holds {} of {place} {by_what}; largest {} at {:.0} %",
            pct(DIVIDED_BELOW),
            top.key,
            top.share * 100.0
        );
        a.divided = list;
    } else {
        a.why = format!(
            "{} holds {:.0} % of {place} {by_what}, across {} held part{}",
            top.key,
            top.share * 100.0,
            parts.len(),
            if parts.len() == 1 { "" } else { "s" }
        );
    }
    a
}

fn pct(x: f64) -> String {
    format!("{:.0} %", x * 100.0)
}

/// The answers for every place a view sees held inside `bounds` on a layer,
/// and for `bounds` itself, in address order.
pub fn holdings_in(
    bounds: &Address,
    live: &[Holding],
    view: &View,
    rules: &LayerRules,
    world: &dyn World,
) -> Vec<Answer> {
    let seen = Seen::of(live, view);
    let mut places: Vec<Address> = seen
        .holds
        .iter()
        .chain(seen.others.iter())
        .map(|h| h.place.clone())
        .filter(|p| bounds.contains(p))
        .collect();
    places.push(bounds.clone());
    places.sort();
    places.dedup();
    places
        .iter()
        .filter_map(|p| who_holds(p, live, view, rules, world))
        .collect()
}

/// Is the point inside the outline (even-odd rule, in the outline's frame)?
pub fn inside_outline(outline: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut inside = false;
    let n = outline.len();
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (outline[i], outline[j]);
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let x = a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
            if p[0] < x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Who holds a point drawn on `on` (a body in `body-latlon`, a galaxy in
/// `galaxy-plane-pc`): the outlines drawn on that place first, the most
/// recent on top, then the place's own answer.
pub fn who_holds_point(
    on: &Address,
    frame: &str,
    p: [f64; 2],
    live: &[Holding],
    view: &View,
    rules: &LayerRules,
    world: &dyn World,
) -> Option<Answer> {
    let seen = Seen::of(live, view);
    let hit = seen
        .holds
        .iter()
        .filter(|h| h.place == *on)
        .filter(|h| match &h.extent {
            Some(Extent::Outline { outline, frame: f }) => f == frame && inside_outline(outline, p),
            _ => false,
        })
        .max_by_key(|h| h.since_s);
    if let Some(h) = hit {
        let why = format!(
            "{} holds the outline drawn on {on} that this point lies inside",
            h.key
        );
        return Some(answer(h, on, Via::Held, why));
    }
    let outlined = |h: &Holding| matches!(h.extent, Some(Extent::Outline { .. }));
    let rest: Vec<Holding> = live.iter().filter(|h| !outlined(h)).cloned().collect();
    who_holds(on, &rest, view, rules, world)
}
