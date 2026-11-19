//! Tiers, disclosure, data views and grants (H6.7, H8.7 audiences).

use serde::{Deserialize, Serialize};
use sse_chronicle::{
    kinds, Chronicle, ChronicleError, Event, EventBody, Evidence, Payload, Publisher, State,
    UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::{BTreeMap, BTreeSet};

/// The audience that always exists (H8.7).
pub const EVERYONE: &str = "everyone";

/// An ordered ladder of disclosure tiers, lowest first. The last rung is GM-only.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ladder {
    /// (tier id, text shown in place of a value at this tier).
    pub rungs: Vec<(String, String)>,
}
impl Default for Ladder {
    fn default() -> Self {
        let r = |a: &str, b: &str| (a.to_string(), b.to_string());
        Ladder {
            rungs: vec![
                r("obvious", "unknown"),
                r("scanned", "not yet scanned"),
                r("surveyed", "survey required"),
                r("secret", ""),
            ],
        }
    }
}
impl Ladder {
    pub fn rank(&self, tier: &str) -> Option<usize> {
        self.rungs.iter().position(|(t, _)| t == tier)
    }
    pub fn secret(&self) -> &str {
        &self.rungs.last().expect("ladder has rungs").0
    }
    pub fn placeholder(&self, tier: &str) -> &str {
        self.rungs
            .iter()
            .find(|(t, _)| t == tier)
            .map_or("", |(_, p)| p.as_str())
    }
}

/// Which tier every field and tag sits at. Field keys are exact names or
/// prefixes ending in `*` (`interior.*`); the longest match wins.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Disclosure {
    #[serde(default)]
    pub ladder: Ladder,
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    /// Tier of anything not listed. Default: the GM-only rung, so a new fact
    /// never leaks by omission.
    #[serde(default)]
    pub default_tier: Option<String>,
    /// Fields never shown to anyone as facts (bookkeeping: `resolve.*`, `observe.*`).
    #[serde(default = "internal")]
    pub internal: Vec<String>,
}
fn internal() -> Vec<String> {
    vec![
        "resolve.*".into(),
        "observe.*".into(),
        "query.*".into(),
        "budget.*".into(),
    ]
}

fn matches(pattern: &str, field: &str) -> Option<usize> {
    match pattern.strip_suffix('*') {
        Some(p) => field.starts_with(p).then_some(p.len()),
        None => (pattern == field).then_some(pattern.len() + 1),
    }
}

impl Disclosure {
    pub fn field_tier(&self, field: &str) -> &str {
        self.fields
            .iter()
            .filter_map(|(p, t)| matches(p, field).map(|n| (n, t)))
            .max_by_key(|(n, _)| *n)
            .map(|(_, t)| t.as_str())
            .unwrap_or_else(|| self.default_tier.as_deref().unwrap_or(self.ladder.secret()))
    }
    pub fn tag_tier(&self, tag: &str) -> &str {
        self.tags
            .get(tag)
            .map(|s| s.as_str())
            .unwrap_or_else(|| self.default_tier.as_deref().unwrap_or(self.ladder.secret()))
    }
    pub fn is_internal(&self, field: &str) -> bool {
        self.internal.iter().any(|p| matches(p, field).is_some())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TagMode {
    All,
    None,
    Listed,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TagFilter {
    pub mode: TagMode,
    #[serde(default)]
    pub list: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "kebab-case")]
pub enum Target {
    Field(String),
    Tag(String),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "as", rename_all = "kebab-case")]
pub enum Precision {
    Exact,
    /// To `digits` significant figures (default 2).
    Rounded {
        #[serde(default = "two")]
        digits: u8,
    },
    /// A range: the value rounded to `step`, then plus or minus `half_width`.
    Band {
        step: f64,
        half_width: f64,
    },
    Hidden,
}
fn two() -> u8 {
    2
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Override {
    pub target: Target,
    pub show: bool,
    #[serde(default)]
    pub precision: Option<Precision>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnknownStyle {
    Omit,
    /// Show the next rung's fields as placeholders ("not yet scanned").
    Placeholder,
}

/// H6.7 `DataView`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DataView {
    pub id: String,
    pub label: String,
    pub show_tiers: Vec<String>,
    pub tags: TagFilter,
    #[serde(default)]
    pub overrides: Vec<Override>,
    pub unknown_style: UnknownStyle,
    /// Honesty marks (authored, impossible, fictional) on player views: off by
    /// default (H8.6). The GM scene always has them.
    #[serde(default)]
    pub show_honesty: bool,
    #[serde(default)]
    pub description: String,
}

/// The campaign's everyday setup: the default view and the Scanned button's pair.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Campaign {
    pub default_view: String,
    pub before: String,
    pub after: String,
    /// Tags that grant a view (the 4X case: whatever carries an empire's
    /// discovery tag is shown to that empire). Checked after explicit grants
    /// at each level, so a grant on the body beats a tag on it, and a tag on a
    /// planet beats a grant on its star.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag_grants: Vec<TagGrant>,
}

/// "Bodies tagged `tag` are shown to `audience` through view `view`."
/// `audience` may be `*` (every audience but the GM's), and `tag` may name the
/// audience as `{audience}`: one rule `* / discovered:{audience} / scanned`
/// covers every empire.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TagGrant {
    pub audience: String,
    pub tag: String,
    pub view: String,
}

impl TagGrant {
    /// The tag this rule looks for when serving `audience`, if it applies to it.
    pub fn tag_for(&self, audience: &str) -> Option<String> {
        (self.audience == "*" || self.audience == audience)
            .then(|| self.tag.replace("{audience}", audience))
    }
}

/// No tags: for callers that have none to hand (the view then follows grants only).
pub fn no_tags() -> &'static BTreeMap<Address, BTreeSet<String>> {
    static EMPTY: BTreeMap<Address, BTreeSet<String>> = BTreeMap::new();
    &EMPTY
}

fn grant_field(audience: &str) -> String {
    if audience == EVERYONE {
        "observe.view".into()
    } else {
        format!("observe.view.{audience}")
    }
}

/// The view in force for `audience` at `addr`: its own grant, else the
/// nearest ancestor's (moons follow their planet), else the audience's
/// session default, else the campaign default. Tags are not consulted; see
/// `effective_view_tagged`.
pub fn effective_view(
    state: &State,
    addr: &Address,
    audience: &str,
    campaign: &Campaign,
) -> String {
    effective_view_tagged(state, no_tags(), addr, audience, campaign)
}

/// As `effective_view`, with the campaign's tag grants: walking up from
/// `addr`, at each level an explicit grant wins, then a tag grant matching a
/// tag on that body (first rule in campaign order).
pub fn effective_view_tagged(
    state: &State,
    tags: &BTreeMap<Address, BTreeSet<String>>,
    addr: &Address,
    audience: &str,
    campaign: &Campaign,
) -> String {
    let f = grant_field(audience);
    let wanted: Vec<(String, &str)> = campaign
        .tag_grants
        .iter()
        .filter_map(|g| g.tag_for(audience).map(|t| (t, g.view.as_str())))
        .collect();
    let mut cur = Some(addr.clone());
    while let Some(a) = cur {
        if let Some(Value::Text(v)) = state.get(&a, &f) {
            return v.clone();
        }
        if let Some(have) = tags.get(&a) {
            if let Some((_, v)) = wanted.iter().find(|(t, _)| have.contains(t)) {
                return (*v).to_string();
            }
        }
        cur = a.parent();
    }
    let root = addr.segments()[0].clone();
    let root = Address::universe(root.key()).expect("valid root");
    match state.get(&root, &format!("{f}.default")) {
        Some(Value::Text(v)) => v.clone(),
        _ => campaign.default_view.clone(),
    }
}

/// Grant `audience` a view of `addr` (and, by inheritance, of everything
/// under it), or with `None` take the grant back so the body falls back to
/// its parent, tags and defaults. With `default`, `addr` must be the universe
/// root and the grant is the audience's starting view for everything. One
/// `observe.grant` event either way: history shows when they learned it.
pub fn grant(
    chronicle: &mut Chronicle,
    addr: &Address,
    audience: &str,
    view: Option<&str>,
    default: bool,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, ChronicleError> {
    let field = if default {
        format!("{}.default", grant_field(audience))
    } else {
        grant_field(audience)
    };
    let mut payload = Payload {
        data: Some(Value::Map(
            [
                ("audience".to_string(), Value::Text(audience.into())),
                (
                    "view".to_string(),
                    view.map_or(Value::Null, |v| Value::Text(v.into())),
                ),
            ]
            .into(),
        )),
        ..Default::default()
    };
    match view {
        Some(v) => {
            payload
                .set
                .insert(addr.clone(), [(field, Value::Text(v.into()))].into());
        }
        None => {
            payload.unset.insert(addr.clone(), vec![field]);
        }
    }
    let e = Event::new(EventBody {
        kind: kinds::OBSERVE_GRANT.into(),
        subject: vec![addr.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: chronicle.next_stamp(),
        causes: vec![],
        payload,
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    chronicle.append(e)
}

/// The Scanned button: flip `addr` between the campaign's before and after
/// views for `audience`. Recorded as an `observe.grant` event, so "when did
/// the players learn this" is history. Pressing it again undoes.
pub fn scan_toggle(
    chronicle: &mut Chronicle,
    addr: &Address,
    audience: &str,
    campaign: &Campaign,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<(ContentHash, String), ChronicleError> {
    let state = chronicle.state_at(at);
    let now = effective_view(&state, addr, audience, campaign);
    let next = if now == campaign.after {
        campaign.before.clone()
    } else {
        campaign.after.clone()
    };
    let mut fields = BTreeMap::new();
    fields.insert(grant_field(audience), Value::Text(next.clone()));
    let mut set = BTreeMap::new();
    set.insert(addr.clone(), fields);
    let e = Event::new(EventBody {
        kind: kinds::OBSERVE_GRANT.into(),
        subject: vec![addr.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: chronicle.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            data: Some(Value::Map(
                [("audience".to_string(), Value::Text(audience.into()))].into(),
            )),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    Ok((chronicle.append(e)?, next))
}

/// Secret-tier fields an authored event sets. If every field it sets is
/// secret, the GM does not send the event to players at all (H6.7: GM plot
/// notes stay off player machines); `true` in the second slot says so.
pub fn withheld(e: &Event, d: &Disclosure) -> (Vec<(Address, String)>, bool) {
    if !matches!(
        e.body.evidence,
        Evidence::Authored | Evidence::AuthoredImpossible
    ) {
        return (Vec::new(), false);
    }
    let secret = d.ladder.secret();
    let mut hidden = Vec::new();
    let mut total = 0;
    for (a, fields) in &e.body.payload.set {
        for f in fields.keys() {
            total += 1;
            if d.field_tier(f) == secret {
                hidden.push((a.clone(), f.clone()));
            }
        }
    }
    let all = total > 0 && hidden.len() == total;
    (hidden, all)
}
