//! The intent registry and seats in the kernel (record design 6 and 9,
//! Care 3; track J5).
//!
//! Every change anyone makes is an intent. The UI used to hold the list of
//! intent kinds as a closed TypeScript union; programs run in the kernel and
//! packs add kinds, so the list is now a **registry the kernel knows**: the
//! built-in kinds declared here (the five the chart room already sends, plus
//! `add_note`), and pack kinds from `provides.other.intents`.
//!
//! A **seat** is a place at the table: whose knowledge it shows, what it may
//! send (`may_send`) and what applies without asking (`auto_approve`). A seat
//! may be driven by a program (J6) instead of a person; a program seat can
//! never send outside its `may_send` or its auto-approve scope.
//!
//! [`decide`] is the one approval rule, the same as the chart room's
//! `decide` in `packages/ui-contract/src/seat.ts` for the built-in kinds.
//! A pack kind may carry a validity condition (the K8 condition language,
//! reading the subject's facts and kept values, the payload as
//! `intent.<field>`), checked for every seat.
//!
//! What an applied intent does:
//! - built-in kinds the kernel can apply (`set_value`, `scan_toggle`,
//!   `grant`, `add_note`) become the same kernel request the chart room's
//!   `intentToRequest` builds ([`to_request`]);
//! - `adopt_physics` and `stack_packs` need the session's pack stack, so the
//!   session applies them;
//! - a pack kind is written to the record as `intent.<kind>`, where J4's
//!   `on_intent` rules take it up.
//!
//! The seat plan, proposals and answers are GM-only record events
//! (`intents.seats`, `intents.proposed`, `intents.decided`): a fold, so it
//! replays, and nothing here reaches a seat (8A).

use crate::{data_text, json_value, subjects, text};
use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use sse_sift::predicate::Predicate;
use std::collections::{BTreeMap, BTreeSet};

/// Record event kinds (plural `intents.`, so they never match `on_intent`).
pub mod kinds {
    pub const SEATS: &str = "intents.seats";
    pub const PROPOSED: &str = "intents.proposed";
    pub const DECIDED: &str = "intents.decided";
}

/// Seconds in a Julian year: the built-in kinds carry `at_yr` as the chart room's time ruler does.
pub const YEAR_S: i64 = sse_core::sheet::YR_JULIAN as i64;

/// Kind names a pack may not take (record event names would collide).
const RESERVED: [&str; 3] = ["proposed", "decided", "seats"];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Number,
    Bool,
    Address,
    List,
    Any,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDoc {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: FieldType,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub required: bool,
}

fn yes() -> bool {
    true
}
fn is_true(b: &bool) -> bool {
    *b
}

/// Who carries out an applied intent.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedBy {
    /// The kernel runs the matching request.
    Kernel,
    /// The session (it holds the pack stack).
    Session,
    /// Written to the record as `intent.<kind>` for rules to take up.
    Record,
}

/// An intent kind, built in or declared by a pack (`provides.other.intents.kinds`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindDoc {
    pub kind: String,
    pub meaning: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldDoc>,
    /// The field naming the subject (an address). Default `address`; a kind
    /// with no such field is about the universe as a whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Must hold for the intent to be accepted from any seat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid: Option<Predicate>,
    /// Never applies without the GM, whatever a seat's auto-approve says.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub never_auto: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Kind {
    #[serde(flatten)]
    pub doc: KindDoc,
    /// `kernel` for built-ins, else the declaring pack.
    pub pack: String,
    pub applied_by: AppliedBy,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntentsPack {
    #[serde(default)]
    kinds: Vec<KindDoc>,
}

fn field(name: &str, ty: FieldType, required: bool) -> FieldDoc {
    FieldDoc {
        name: name.into(),
        ty,
        required,
    }
}

fn builtin(
    kind: &str,
    meaning: &str,
    fields: Vec<FieldDoc>,
    never_auto: bool,
    by: AppliedBy,
) -> Kind {
    let has_address = fields.iter().any(|f| f.name == "address");
    Kind {
        doc: KindDoc {
            kind: kind.into(),
            meaning: meaning.into(),
            fields,
            subject: has_address.then(|| "address".to_string()),
            valid: None,
            never_auto,
        },
        pack: "kernel".into(),
        applied_by: by,
    }
}

/// The built-in kinds, as the chart room sends them (`intent.ts`), plus
/// `add_note` and `set_campaign_now`.
pub fn builtins() -> Vec<Kind> {
    use AppliedBy::*;
    use FieldType::*;
    vec![
        builtin(
            "set_value",
            "set one field of one subject, as of a universe time",
            vec![
                field("address", Address, true),
                field("field", Text, true),
                field("value", Any, true),
                field("at_yr", Number, true),
                field("impossible", Bool, false),
            ],
            false,
            Kernel,
        ),
        builtin(
            "adopt_physics",
            "adopt a campaign physics pack over the base stack (null: back to the base physics)",
            vec![field("pack", Any, true), field("requests", List, true)],
            true,
            Session,
        ),
        builtin(
            "stack_packs",
            "set the packs stacked above the ones the session opened with",
            vec![field("packs", List, true)],
            true,
            Session,
        ),
        builtin(
            "scan_toggle",
            "give an audience the next data view up for one subject, or take it back",
            vec![
                field("address", Address, true),
                field("audience", Text, true),
                field("at_yr", Number, true),
            ],
            true,
            Kernel,
        ),
        builtin(
            "grant",
            "grant an audience a data view of one subject and everything under it (view null: take it back)",
            vec![
                field("address", Address, true),
                field("audience", Text, true),
                field("view", Any, true),
                field("default", Bool, false),
                field("at_yr", Number, true),
            ],
            true,
            Kernel,
        ),
        builtin(
            "set_known",
            "set what an audience knows of a thing (gm_only, heard_of, known), or of one field on it; scope this, children or a list; one undo",
            vec![
                field("address", Address, true),
                field("audience", Text, true),
                field("landmark", Any, true),
                field("field", Text, false),
                field("scope", Any, false),
                field("from_yr", Number, false),
                field("at_yr", Number, true),
            ],
            true,
            Kernel,
        ),
        builtin(
            "undo_known",
            "put back what one set_known changed",
            vec![field("undo", Text, true), field("at_yr", Number, true)],
            true,
            Kernel,
        ),
        builtin(
            "add_note",
            "write a note on the record (a seat's own audience: its journal, applied without asking)",
            vec![
                field("address", Address, true),
                field("text", Text, true),
                field("tags", List, false),
                field("audience", Any, false),
            ],
            false,
            Kernel,
        ),
        builtin(
            "set_campaign_now",
            "move the campaign's now (universe seconds), so every replica's clock agrees",
            vec![field("now_s", Number, true)],
            true,
            Kernel,
        ),
    ]
}

/// The built-in kinds plus every pack's, bottom to top.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct IntentRegistry {
    pub kinds: Vec<Kind>,
}

impl Default for IntentRegistry {
    fn default() -> Self {
        IntentRegistry { kinds: builtins() }
    }
}

impl IntentRegistry {
    /// Read every pack's `provides.other.intents` (bottom to top). A kind
    /// declared twice, or with a built-in's name, is refused.
    pub fn from_packs(packs: &[(String, J)]) -> Result<Self, Vec<String>> {
        let mut reg = IntentRegistry::default();
        let mut problems = vec![];
        for (pack, v) in packs {
            let p: IntentsPack = match serde_json::from_value(v.clone()) {
                Ok(p) => p,
                Err(e) => {
                    problems.push(format!("{pack}: intents: {e}"));
                    continue;
                }
            };
            for d in p.kinds {
                if !crate::ident_ok(&d.kind) || d.kind.contains('-') || d.kind.contains('.') {
                    problems.push(format!(
                        "{pack}: intent {:?}: kinds are lowercase letters, digits and _",
                        d.kind
                    ));
                }
                if RESERVED.contains(&d.kind.as_str()) {
                    problems.push(format!("{pack}: intent {} is a reserved name", d.kind));
                }
                if let Some(k) = reg.get(&d.kind) {
                    problems.push(format!(
                        "{pack}: intent {} is already declared by {}",
                        d.kind, k.pack
                    ));
                }
                let mut seen = BTreeSet::new();
                for f in &d.fields {
                    if !seen.insert(f.name.as_str()) || f.name == "kind" {
                        problems.push(format!(
                            "{pack}: intent {}: field {:?} twice or reserved",
                            d.kind, f.name
                        ));
                    }
                }
                let subj = d.subject.as_deref().unwrap_or("address");
                let declared = d.fields.iter().find(|f| f.name == subj).map(|f| f.ty);
                if d.subject.is_some() && declared != Some(FieldType::Address) {
                    problems.push(format!(
                        "{pack}: intent {}: subject {subj:?} must be a declared address field",
                        d.kind
                    ));
                }
                let mut doc = d;
                if doc.subject.is_none() && declared == Some(FieldType::Address) {
                    doc.subject = Some("address".into());
                }
                reg.kinds.push(Kind {
                    doc,
                    pack: pack.clone(),
                    applied_by: AppliedBy::Record,
                });
            }
        }
        if problems.is_empty() {
            Ok(reg)
        } else {
            Err(problems)
        }
    }

    pub fn get(&self, kind: &str) -> Option<&Kind> {
        self.kinds.iter().find(|k| k.doc.kind == kind)
    }

    /// Every built-in kind name (what a GM seat may send by default).
    pub fn names(&self) -> Vec<String> {
        self.kinds.iter().map(|k| k.doc.kind.clone()).collect()
    }

    /// Why an intent is malformed for its kind, or None.
    pub fn check(&self, intent: &J) -> Result<&Kind, String> {
        let o = intent.as_object().ok_or("an intent is an object")?;
        let kind = o
            .get("kind")
            .and_then(J::as_str)
            .ok_or("an intent needs a kind")?;
        let k = self
            .get(kind)
            .ok_or_else(|| format!("unknown intent kind {kind}"))?;
        for f in &k.doc.fields {
            match o.get(&f.name) {
                None | Some(J::Null) if f.required && f.ty != FieldType::Any => {
                    return Err(format!("{kind}: {} is required", f.name))
                }
                None if f.required => return Err(format!("{kind}: {} is required", f.name)),
                None | Some(J::Null) => {}
                Some(v) => {
                    let ok = match f.ty {
                        FieldType::Text => v.as_str().map(|s| !s.is_empty()).unwrap_or(false),
                        FieldType::Number => v.as_f64().map(f64::is_finite).unwrap_or(false),
                        FieldType::Bool => v.is_boolean(),
                        FieldType::Address => v
                            .as_str()
                            .map(|s| s.parse::<Address>().is_ok())
                            .unwrap_or(false),
                        FieldType::List => v.is_array(),
                        FieldType::Any => true,
                    };
                    if !ok {
                        return Err(format!("{kind}: {} must be {:?}", f.name, f.ty).to_lowercase());
                    }
                }
            }
        }
        for key in o.keys() {
            if key != "kind" && !k.doc.fields.iter().any(|f| &f.name == key) {
                return Err(format!("{kind}: unknown field {key}"));
            }
        }
        // The chart room's own checks on the built-ins (`checkIntent`).
        let audience_ok = |s: &str| {
            let b = s.as_bytes();
            !b.is_empty()
                && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
                && b.iter()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"._-".contains(c))
        };
        match kind {
            "set_value" => {
                let f = o["field"].as_str().unwrap_or("");
                let b = f.as_bytes();
                if !(b[0].is_ascii_lowercase()
                    && b.iter().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || b"._-".contains(c)
                    }))
                {
                    return Err("field must be a lower-case name".into());
                }
            }
            "scan_toggle" | "grant" => {
                if !audience_ok(o["audience"].as_str().unwrap_or("")) {
                    return Err("audience must be a lower-case name".into());
                }
                if kind == "grant" {
                    let view = &o["view"];
                    if !(view.is_null() || view.as_str().map(|s| !s.is_empty()).unwrap_or(false)) {
                        return Err(
                            "view must be a data view id, or null to take the grant back".into(),
                        );
                    }
                    if o.get("default") == Some(&J::Bool(true)) && view.is_null() {
                        return Err("a starting view cannot be taken back, only replaced".into());
                    }
                }
            }
            "set_known" => {
                if !audience_ok(o["audience"].as_str().unwrap_or("")) {
                    return Err("audience must be a lower-case name".into());
                }
                let l = &o["landmark"];
                if !(l.is_null() || matches!(l.as_str(), Some("gm_only" | "heard_of" | "known"))) {
                    return Err("landmark must be gm_only, heard_of, known or null".into());
                }
                let sc = o.get("scope").unwrap_or(&J::Null);
                let ok = sc.is_null()
                    || matches!(sc.as_str(), Some("this" | "children"))
                    || sc["list"]
                        .as_array()
                        .is_some_and(|a| a.iter().all(J::is_string));
                if !ok {
                    return Err("scope must be this, children or {list: [addresses]}".into());
                }
            }
            "set_campaign_now" => {
                let t = o["now_s"].as_f64().unwrap_or(f64::NAN);
                if !(t.is_finite() && sse_core::math::floor(t) == t && t.abs() < 9.0e15) {
                    return Err("now_s must be a whole number of seconds".into());
                }
            }
            "stack_packs" => {
                for p in o["packs"].as_array().into_iter().flatten() {
                    if !(p["id"].is_string() && p["format"] == "sse.pack/1") {
                        return Err("each pack must be an sse.pack/1 document".into());
                    }
                }
            }
            "adopt_physics" => {
                let p = &o["pack"];
                if !p.is_null() && !(p.is_object() && p["id"].is_string()) {
                    return Err("physics pack must be a pack document".into());
                }
                if !p.is_null() && p["format"] != "sse.pack/1" {
                    return Err("physics pack must be sse.pack/1".into());
                }
            }
            _ => {}
        }
        Ok(k)
    }

    /// The subject an intent is about, if its kind names one.
    pub fn subject(&self, intent: &J) -> Option<Address> {
        let k = self.get(intent["kind"].as_str()?)?;
        intent[k.doc.subject.as_deref()?].as_str()?.parse().ok()
    }
}

/// `auto_approve` entry of a seat (as `seat.ts`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AutoApprove {
    pub intent: String,
    /// Addresses (and everything inside them) where the intent applies without asking.
    pub scope: Vec<String>,
    /// For set_value: only these fields apply without asking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    Gm,
    Player,
}

/// A seat (as `seat.ts`, plus `program`). Unknown UI fields (screens, lens,
/// sealed) are carried through untouched.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Seat {
    pub id: String,
    #[serde(default)]
    pub label: String,
    pub audience: String,
    pub authority: Authority,
    #[serde(default)]
    pub may_send: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub auto_approve: Vec<AutoApprove>,
    /// The program driving this seat (J6); absent for a person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, J>,
}

impl Seat {
    pub fn gm(reg: &IntentRegistry) -> Seat {
        Seat {
            id: "gm".into(),
            label: "Game master".into(),
            audience: "all".into(),
            authority: Authority::Gm,
            may_send: reg.names(),
            auto_approve: vec![],
            program: None,
            rest: BTreeMap::new(),
        }
    }
    /// The shared table: sees `everyone`, may ask the GM to set a value.
    pub fn table() -> Seat {
        Seat {
            id: "table".into(),
            label: "Table".into(),
            audience: "everyone".into(),
            authority: Authority::Player,
            may_send: vec!["set_value".into()],
            auto_approve: vec![],
            program: None,
            rest: BTreeMap::new(),
        }
    }
}

/// `inner` is `outer` or lies inside it (address segments, not string prefixes).
pub fn within(inner: &str, outer: &str) -> bool {
    inner == outer
        || inner
            .strip_prefix(outer)
            .map(|r| r.starts_with('/'))
            .unwrap_or(false)
}

/// The outcome of the approval rule.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Decision {
    Apply,
    AskGm,
    Refuse { reason: String },
}

fn refuse(r: impl Into<String>) -> Decision {
    Decision::Refuse { reason: r.into() }
}

/// The note audience of an `add_note` intent is exactly the seat's own.
fn own_audience(seat: &Seat, intent: &J) -> bool {
    match &intent["audience"] {
        J::String(s) => *s == seat.audience,
        J::Array(a) => a.len() == 1 && a[0].as_str() == Some(seat.audience.as_str()),
        _ => false,
    }
}

/// The approval rule for one intent from one seat. `valid` is the kind's
/// validity condition already evaluated (None when it has none).
pub fn decide(
    reg: &IntentRegistry,
    seat: &Seat,
    intent: &J,
    valid: Option<Result<(), String>>,
) -> Decision {
    let k = match reg.check(intent) {
        Ok(k) => k,
        Err(e) => return refuse(e),
    };
    let kind = k.doc.kind.as_str();
    if !seat.may_send.iter().any(|m| m == kind) {
        return refuse(format!("seat {} may not send {kind}", label(seat)));
    }
    if let Some(Err(why)) = valid {
        return refuse(format!("not valid now: {why}"));
    }
    if seat.authority == Authority::Gm {
        return Decision::Apply;
    }
    if kind == "set_value" {
        let f = intent["field"].as_str().unwrap_or("");
        if f == "observe" || f.starts_with("observe.") {
            return refuse("what players know is set by the game master");
        }
    }
    if k.applied_by == AppliedBy::Session && seat.program.is_some() {
        return refuse(format!("a program cannot send {kind}"));
    }
    let subject = reg.subject(intent).map(|a| a.to_string());
    let in_scope = |a: &AutoApprove| {
        a.intent == kind
            && match &subject {
                Some(s) => a.scope.iter().any(|o| within(s, o)),
                None => a.scope.is_empty(),
            }
    };
    if seat.program.is_some() && !seat.auto_approve.iter().any(in_scope) {
        return refuse(format!(
            "{kind} on {} is outside program seat {}'s scope",
            subject.as_deref().unwrap_or("the universe"),
            label(seat)
        ));
    }
    if k.doc.never_auto {
        return Decision::AskGm;
    }
    if kind == "add_note" && own_audience(seat, intent) {
        return Decision::Apply;
    }
    let ok = seat.auto_approve.iter().any(|a| {
        in_scope(a)
            && match &a.fields {
                None => true,
                Some(fs) => {
                    kind == "set_value"
                        && fs
                            .iter()
                            .any(|f| Some(f.as_str()) == intent["field"].as_str())
                }
            }
    });
    if ok {
        Decision::Apply
    } else {
        Decision::AskGm
    }
}

fn label(seat: &Seat) -> &str {
    if seat.label.is_empty() {
        &seat.id
    } else {
        &seat.label
    }
}

/// Evaluate a pack kind's validity condition: the subject's facts and kept
/// values at `at_s`, with the payload as `intent.<field>`.
pub fn validity(
    reg: &IntentRegistry,
    intent: &J,
    universe: &Chronicle,
    record: &Chronicle,
    root: &Address,
    at_s: i64,
) -> Option<Result<(), String>> {
    let k = reg.get(intent["kind"].as_str()?)?;
    let cond = k.doc.valid.as_ref()?;
    let at = UniverseTime::seconds(at_s);
    let world = subjects(universe, record, root, at);
    let mut s = reg
        .subject(intent)
        .and_then(|a| world.get(&a).cloned())
        .unwrap_or_default();
    s.now_s = at_s;
    for (key, v) in intent.as_object().into_iter().flatten() {
        if key == "kind" {
            continue;
        }
        if let Ok(v) = json_value(v) {
            s.facts.insert(format!("intent.{key}"), v);
        }
    }
    let ev = cond.eval(&s, false);
    Some(if ev.holds {
        Ok(())
    } else {
        let read: Vec<String> = ev
            .read
            .iter()
            .map(|r| {
                format!(
                    "{} = {}",
                    r.field,
                    serde_json::to_string(&r.value).unwrap_or_default()
                )
            })
            .collect();
        Err(if read.is_empty() {
            format!("{}'s condition does not hold for this subject", k.doc.kind)
        } else {
            read.join(", ")
        })
    })
}

/// The kernel request (as JSON) an applied built-in intent becomes: the
/// same translation as the chart room's `intentToRequest`. `by` names the
/// sending seat for notes. None for session and record kinds.
pub fn to_request(intent: &J, by: &str, at_s: i64) -> Option<J> {
    // Math.round, as the chart room's yearsToSeconds.
    let secs = |yr: &J| sse_core::math::floor(yr.as_f64().unwrap_or(0.0) + 0.5) as i64 * YEAR_S;
    let o = intent.as_object()?;
    Some(match o.get("kind")?.as_str()? {
        "set_value" => serde_json::json!({
            "op": "set_authored",
            "edit": {
                "address": o["address"],
                "field": o["field"],
                "value": o["value"],
                "occurred_at_s": secs(&o["at_yr"]),
                "impossible": o.get("impossible").and_then(J::as_bool).unwrap_or(false),
            }
        }),
        "scan_toggle" => serde_json::json!({
            "op": "scan_toggle", "address": o["address"], "audience": o["audience"], "at_s": secs(&o["at_yr"])
        }),
        "grant" => {
            let mut r = serde_json::json!({
                "op": "grant", "address": o["address"], "audience": o["audience"], "at_s": secs(&o["at_yr"])
            });
            if let Some(v) = o.get("view").and_then(J::as_str) {
                r["view"] = J::String(v.into());
            }
            if o.get("default") == Some(&J::Bool(true)) {
                r["default"] = J::Bool(true);
            }
            r
        }
        "set_known" => {
            let mut r = serde_json::json!({
                "op": "set_known", "address": o["address"], "audience": o["audience"],
                "landmark": o["landmark"], "at_s": secs(&o["at_yr"])
            });
            for k in ["field", "scope"] {
                if let Some(v) = o.get(k).filter(|v| !v.is_null()) {
                    r[k] = v.clone();
                }
            }
            if let Some(v) = o.get("from_yr").filter(|v| !v.is_null()) {
                r["from_s"] = secs(v).into();
            }
            r
        }
        "undo_known" => serde_json::json!({
            "op": "undo_known", "undo": o["undo"], "at_s": secs(&o["at_yr"])
        }),
        "set_campaign_now" => serde_json::json!({
            "op": "set_campaign_now", "at_s": o["now_s"].as_f64().unwrap_or(0.0) as i64
        }),
        "add_note" => {
            let mut r = serde_json::json!({
                "op": "add_note", "address": o["address"], "text": o["text"], "by": by, "at_s": at_s
            });
            for k in ["tags", "audience"] {
                if let Some(v) = o.get(k).filter(|v| !v.is_null()) {
                    r[k] = v.clone();
                }
            }
            r
        }
        _ => return None,
    })
}

fn write(
    record: &mut Chronicle,
    kind: &str,
    subject: &Address,
    at_s: i64,
    causes: Vec<ContentHash>,
    data: BTreeMap<String, Value>,
    by: &str,
) -> Result<ContentHash, String> {
    let at = UniverseTime::seconds(at_s);
    let e = Event::new(EventBody {
        kind: kind.into(),
        subject: vec![subject.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: record.next_stamp(),
        causes,
        payload: Payload {
            data: Some(Value::Map(data)),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("seat", by, "1"),
    })
    .map_err(|e| e.to_string())?;
    record.append(e).map_err(|e| e.to_string())
}

/// Store the seat plan (GM only; the latest plan wins). Checked against the registry.
pub fn set_seats(
    reg: &IntentRegistry,
    record: &mut Chronicle,
    root: &Address,
    seats: &[Seat],
    at_s: i64,
) -> Result<ContentHash, String> {
    let problems = check_seats(reg, seats);
    if !problems.is_empty() {
        return Err(problems.join("; "));
    }
    let mut d = BTreeMap::new();
    d.insert(
        "seats".into(),
        text(&serde_json::to_string(seats).expect("plain")),
    );
    write(record, kinds::SEATS, root, at_s, vec![], d, "gm")
}

/// Problems with a seat plan: unknown kinds, auto-approve without may_send, duplicate ids.
pub fn check_seats(reg: &IntentRegistry, seats: &[Seat]) -> Vec<String> {
    let mut out = vec![];
    let mut ids = BTreeSet::new();
    for s in seats {
        if !ids.insert(s.id.as_str()) {
            out.push(format!("seat {} twice", s.id));
        }
        for k in &s.may_send {
            if reg.get(k).is_none() {
                out.push(format!("seat {}: unknown intent {k}", label(s)));
            }
        }
        for a in &s.auto_approve {
            if !s.may_send.contains(&a.intent) {
                out.push(format!(
                    "seat {}: applies {} without asking but may not send it",
                    label(s),
                    a.intent
                ));
            }
        }
        if s.program.is_some() && s.authority == Authority::Gm {
            out.push(format!(
                "seat {}: a program never has GM authority",
                label(s)
            ));
        }
    }
    out
}

/// The seat plan in force: the latest `intents.seats`, else the default
/// (the GM and the shared table). A GM seat is always present.
pub fn seats(reg: &IntentRegistry, record: &Chronicle) -> Vec<Seat> {
    let mut s: Vec<Seat> = record
        .events()
        .iter()
        .rev()
        .find(|e| e.body.kind == kinds::SEATS)
        .and_then(|e| data_text(e, "seats"))
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_else(|| vec![Seat::table()]);
    if !s.iter().any(|x| x.id == "gm") {
        s.insert(0, Seat::gm(reg));
    }
    s
}

/// An intent waiting for the GM.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pending {
    pub id: String,
    pub seat: String,
    pub intent: J,
    pub at_s: i64,
}

/// Record that an intent waits for the GM.
pub fn propose(
    reg: &IntentRegistry,
    record: &mut Chronicle,
    root: &Address,
    seat: &Seat,
    intent: &J,
    at_s: i64,
) -> Result<ContentHash, String> {
    let subject = reg.subject(intent).unwrap_or_else(|| root.clone());
    let mut d = BTreeMap::new();
    d.insert("seat".into(), text(&seat.id));
    d.insert("intent".into(), text(&intent.to_string()));
    write(record, kinds::PROPOSED, &subject, at_s, vec![], d, &seat.id)
}

/// Intents waiting for the GM, oldest first.
pub fn pending(record: &Chronicle) -> Vec<Pending> {
    let decided: BTreeSet<&str> = record
        .events()
        .iter()
        .filter(|e| e.body.kind == kinds::DECIDED)
        .filter_map(|e| data_text(e, "proposal"))
        .collect();
    record
        .events()
        .iter()
        .filter(|e| e.body.kind == kinds::PROPOSED)
        .filter(|e| !decided.contains(e.id.hex().as_str()))
        .filter_map(|e| {
            Some(Pending {
                id: e.id.hex(),
                seat: data_text(e, "seat")?.to_string(),
                intent: serde_json::from_str(data_text(e, "intent")?).ok()?,
                at_s: e.body.occurred_at.s,
            })
        })
        .collect()
}

/// Record the GM's answer. Returns the proposal (to apply on a yes).
pub fn answer(
    record: &mut Chronicle,
    proposal: &str,
    approve: bool,
    at_s: i64,
) -> Result<(Pending, ContentHash), String> {
    let p = pending(record)
        .into_iter()
        .find(|p| p.id == proposal)
        .ok_or_else(|| format!("no intent {proposal} is waiting (unknown or already answered)"))?;
    let cause = record
        .events()
        .iter()
        .find(|e| e.id.hex() == proposal)
        .map(|e| (e.id, e.body.subject[0].clone()))
        .expect("pending came from here");
    let mut d = BTreeMap::new();
    d.insert("proposal".into(), text(proposal));
    d.insert("approve".into(), Value::Bool(approve));
    let id = write(
        record,
        kinds::DECIDED,
        &cause.1,
        at_s.max(p.at_s),
        vec![cause.0],
        d,
        "gm",
    )?;
    Ok((p, id))
}

/// Write an applied pack intent to the record as `intent.<kind>`, where
/// `on_intent` rules take it up.
pub fn record_intent(
    reg: &IntentRegistry,
    record: &mut Chronicle,
    root: &Address,
    seat: &str,
    intent: &J,
    at_s: i64,
    causes: Vec<ContentHash>,
) -> Result<ContentHash, String> {
    let kind = intent["kind"].as_str().ok_or("no kind")?;
    let subject = reg.subject(intent).unwrap_or_else(|| root.clone());
    let mut payload = intent.clone();
    if let Some(o) = payload.as_object_mut() {
        o.remove("kind");
    }
    let mut d = BTreeMap::new();
    d.insert("from".into(), text(&format!("seat.{seat}")));
    d.insert("payload".into(), text(&payload.to_string()));
    write(
        record,
        &format!("{}{kind}", crate::kinds::INTENT_PREFIX),
        &subject,
        at_s,
        causes,
        d,
        seat,
    )
}
