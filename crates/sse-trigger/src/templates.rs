//! Templates, `create` and `retire` (record design section 8, track J7).
//!
//! A pack declares **templates** (ship, unit, structure, module, station,
//! faction, ...): the authored facts an object made from it starts with, its
//! kept values with their starting values, the tags it carries and the
//! programs it may run. `create` makes an authored object from one, in the
//! authored namespace; `retire` takes one out of play. Traveller's stock
//! ships become the first use (Care 4: no Traveller special).
//!
//! An object made from a template carries everything as its own facts and
//! kept values (`template.id`, `template.pack`, `template.version`,
//! `template.programs`, its tags), so **removing the pack leaves the object
//! valid**: nothing is read back from the template after creation. Its kept
//! values stay readable as any orphaned kept value does (J1).
//!
//! Universe side: one `template.create` event (authored, published by the
//! pack, template and version), and `template.retire` (sets `retired`).
//! Record side: one `record.set` per kept value, written by
//! `template.<id>`, which the kept kind must list as a writer.

use crate::json_value;
use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use sse_record::{KeptSet, Registry};
use std::collections::BTreeMap;

pub mod kinds {
    pub const CREATE: &str = "template.create";
    pub const RETIRE: &str = "template.retire";
}

/// Facts every object made from a template carries.
pub mod fields {
    pub const ID: &str = "template.id";
    pub const PACK: &str = "template.pack";
    pub const VERSION: &str = "template.version";
    pub const PROGRAMS: &str = "template.programs";
    pub const RETIRED: &str = "retired";
    pub const NAME: &str = "name";
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateDoc {
    pub id: String,
    /// The address segment kind of objects made from it (`ship`, `station`, `faction`, ...).
    pub kind: String,
    pub label: String,
    /// Authored universe facts it starts with (e.g. `sensor.range_m`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub facts: BTreeMap<String, J>,
    /// Kept values it starts with (record side), by kept kind.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kept: BTreeMap<String, J>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Programs an object made from it may run (J6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub programs: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub explain: String,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Template {
    pub pack: String,
    pub version: String,
    #[serde(flatten)]
    pub doc: TemplateDoc,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplatesPack {
    #[serde(default)]
    templates: Vec<TemplateDoc>,
}

/// The stack's templates, bottom to top.
#[derive(Clone, PartialEq, Debug, Default, Serialize)]
pub struct Templates {
    pub templates: Vec<Template>,
}

fn writer(id: &str) -> String {
    format!("template.{id}")
}

impl Templates {
    /// Read every pack's `provides.other.templates`. A template id declared
    /// twice is refused, and so is a fact or tag the kernel would not take.
    pub fn from_packs(packs: &[(String, String, J)]) -> Result<Self, Vec<String>> {
        let mut out = Templates::default();
        let mut problems = vec![];
        for (pack, version, v) in packs {
            let p: TemplatesPack = match serde_json::from_value(v.clone()) {
                Ok(p) => p,
                Err(e) => {
                    problems.push(format!("{pack}: templates: {e}"));
                    continue;
                }
            };
            for d in p.templates {
                let at = format!("{pack}: template {}", d.id);
                if !crate::ident_ok(&d.id) {
                    problems.push(format!("{at}: ids are lowercase letters, digits and _ - ."));
                }
                if !crate::ident_ok(&d.kind) || d.kind.contains(['.', '-']) {
                    problems.push(format!(
                        "{at}: kind {:?} must be a lowercase word (it becomes an address segment)",
                        d.kind
                    ));
                }
                if let Some(t) = out.get(&d.id) {
                    problems.push(format!("{at} is already declared by {}", t.pack));
                }
                if d.label.trim().is_empty() {
                    problems.push(format!("{at}: needs a label"));
                }
                for (f, v) in &d.facts {
                    if f.starts_with("template.")
                        || f == fields::RETIRED
                        || f.starts_with(crate::TAG_FIELD)
                    {
                        problems.push(format!(
                            "{at}: fact {f} is set by create itself (use tags for tags)"
                        ));
                    }
                    if json_value(v).is_err() {
                        problems.push(format!("{at}: fact {f} is not a plain value"));
                    }
                }
                for t in &d.tags {
                    if !crate::ident_ok(t) {
                        problems.push(format!("{at}: tag {t:?} must be lowercase"));
                    }
                }
                out.templates.push(Template {
                    pack: pack.clone(),
                    version: version.clone(),
                    doc: d,
                });
            }
        }
        if problems.is_empty() {
            Ok(out)
        } else {
            Err(problems)
        }
    }

    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.doc.id == id)
    }

    /// Against the stack's kept kinds: each starting kept value exists, fits
    /// its kind, and the kind lists `template.<id>` as a writer.
    pub fn check(&self, reg: &Registry) -> Vec<String> {
        let mut out = vec![];
        for t in &self.templates {
            for (kind, v) in &t.doc.kept {
                let v = match json_value(v) {
                    Ok(v) => v,
                    Err(e) => {
                        out.push(format!("template {}: kept {kind}: {e}", t.doc.id));
                        continue;
                    }
                };
                if let Err(e) = reg.check(kind, Some(&v), &writer(&t.doc.id)) {
                    out.push(format!("template {}: kept {kind}: {e}", t.doc.id));
                }
            }
        }
        out
    }
}

/// What a `create` wrote.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Created {
    pub address: String,
    /// The `template.create` universe event.
    pub event: String,
    /// The kept values' `record.set` events.
    pub kept: Vec<String>,
}

/// The address a new object would take: `<under>/<kind>:<key>`, the key
/// given or the next free number among that kind under `under`.
pub fn next_address(
    universe: &Chronicle,
    t: &Template,
    under: &Address,
    key: Option<&str>,
) -> Result<Address, String> {
    let st = universe.state_at(UniverseTime::seconds(i64::MAX));
    let taken = |a: &Address| st.facts.contains_key(a);
    match key {
        Some(k) => {
            let a = under
                .child_key(&t.doc.kind, k)
                .map_err(|e| format!("{k:?}: {e}"))?;
            if taken(&a) {
                return Err(format!("{a} already exists"));
            }
            Ok(a)
        }
        None => {
            for n in 1..=100_000u32 {
                let a = under
                    .child_key(&t.doc.kind, &n.to_string())
                    .map_err(|e| e.to_string())?;
                if !taken(&a) {
                    return Ok(a);
                }
            }
            Err(format!("no free {} address under {under}", t.doc.kind))
        }
    }
}

/// Make an authored object from a template at `at`. `causes` (record
/// events, e.g. a rule's firing) are cited by the kept values only: the
/// universe event cites nothing secret.
#[allow(clippy::too_many_arguments)]
pub fn create(
    universe: &mut Chronicle,
    record: &mut Chronicle,
    reg: &Registry,
    root: &Address,
    t: &Template,
    under: &Address,
    key: Option<&str>,
    name: Option<&str>,
    at: UniverseTime,
    causes: Vec<ContentHash>,
) -> Result<(Created, ContentHash), String> {
    if !root.contains(under) {
        return Err(format!("{under} is not in universe {root}"));
    }
    let problems = Templates {
        templates: vec![t.clone()],
    }
    .check(reg);
    if !problems.is_empty() {
        return Err(problems.join("; "));
    }
    let a = next_address(universe, t, under, key)?;
    let mut row: BTreeMap<String, Value> = BTreeMap::new();
    for (f, v) in &t.doc.facts {
        row.insert(f.clone(), json_value(v)?);
    }
    for tag in &t.doc.tags {
        row.insert(format!("{}{tag}", crate::TAG_FIELD), Value::Bool(true));
    }
    let txt = |s: &str| Value::Text(s.into());
    row.insert(fields::ID.into(), txt(&t.doc.id));
    row.insert(fields::PACK.into(), txt(&t.pack));
    row.insert(fields::VERSION.into(), txt(&t.version));
    row.insert(
        fields::PROGRAMS.into(),
        Value::Array(t.doc.programs.iter().map(|p| txt(p)).collect()),
    );
    row.insert(
        fields::NAME.into(),
        txt(name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or(&t.doc.label)),
    );
    let publisher = Publisher::new(&t.pack, &t.doc.id, &t.version);
    let e = Event::new(EventBody {
        kind: kinds::CREATE.into(),
        subject: vec![a.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: universe.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: [(a.clone(), row)].into(),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })
    .map_err(|e| e.to_string())?;
    let id = universe.append(e).map_err(|e| e.to_string())?;
    let mut kept = vec![];
    for (kind, v) in &t.doc.kept {
        let k = sse_record::set_kept(
            record,
            reg,
            &KeptSet {
                address: a.clone(),
                kind: kind.clone(),
                value: Some(json_value(v)?),
                writer: writer(&t.doc.id),
                reason: Some(format!("made from template {}", t.doc.id)),
                causes: causes.clone(),
            },
            at,
            &publisher,
        )
        .map_err(|e| e.to_string())?;
        kept.push(k.hex());
    }
    Ok((
        Created {
            address: a.to_string(),
            event: id.hex(),
            kept,
        },
        id,
    ))
}

/// Take an object made from a template out of play (`retired` = true). Its
/// facts, kept values and history stay; nothing is deleted.
pub fn retire(
    universe: &mut Chronicle,
    a: &Address,
    reason: Option<&str>,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, String> {
    let st = universe.state_at(at);
    if st.get(a, fields::ID).is_none() && st.get(a, crate::split::FROM).is_none() {
        return Err(format!(
            "{a} was not made from a template or a split (only those retire)"
        ));
    }
    if st.get(a, fields::RETIRED) == Some(&Value::Bool(true)) {
        return Err(format!("{a} is already retired"));
    }
    let mut data = BTreeMap::new();
    if let Some(r) = reason {
        data.insert("reason".to_string(), Value::Text(r.into()));
    }
    let e = Event::new(EventBody {
        kind: kinds::RETIRE.into(),
        subject: vec![a.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: universe.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: [(
                a.clone(),
                [(fields::RETIRED.to_string(), Value::Bool(true))].into(),
            )]
            .into(),
            data: (!data.is_empty()).then_some(Value::Map(data)),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })
    .map_err(|e| e.to_string())?;
    universe.append(e).map_err(|e| e.to_string())
}

/// Whether the object at `a` may run `program`: its template listed it, it
/// is not retired. Read from the object's own facts, so it holds with the
/// pack removed.
pub fn may_run(
    universe: &Chronicle,
    a: &Address,
    program: &str,
    at: UniverseTime,
) -> Result<(), String> {
    let st = universe.state_at(at);
    let Some(Value::Array(ps)) = st.get(a, fields::PROGRAMS) else {
        return Err(format!(
            "{a} was not made from a template, so it runs no programs"
        ));
    };
    if st.get(a, fields::RETIRED) == Some(&Value::Bool(true)) {
        return Err(format!("{a} is retired"));
    }
    if ps.iter().any(|p| *p == Value::Text(program.into())) {
        Ok(())
    } else {
        Err(format!("{a}'s template does not list program {program}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn made_objects_run_only_their_programs_and_outlive_the_pack() {
        let pack = json!({"templates": [{"id": "drone", "kind": "drone", "label": "Mining drone",
            "facts": {"sensor.range_m": 1.0e7}, "kept": {"ship.cargo_t": 0}, "tags": ["ship"], "programs": ["mining-drone"]}]});
        let t = Templates::from_packs(&[("test.mine".into(), "0.1.0".into(), pack)]).unwrap();
        let kept = json!({"kept": [{"id": "ship.cargo_t", "shape": "number", "tier": "scanned",
            "writers": ["gm", "template.drone"], "describes": "cargo"}]});
        let reg = Registry::from_packs(&[("test.mine".into(), kept)]).unwrap();
        assert!(t.check(&reg).is_empty());
        assert!(t.check(&Registry::default())[0].contains("ship.cargo_t"));
        let (mut u, mut r) = (Chronicle::new(), Chronicle::new());
        let root: Address = "u:m".parse().unwrap();
        let at = UniverseTime::seconds(5);
        let (c, _) = create(
            &mut u,
            &mut r,
            &reg,
            &root,
            &t.templates[0],
            &root,
            Some("d7"),
            None,
            at,
            vec![],
        )
        .unwrap();
        assert_eq!(c.address, "u:m/drone:d7");
        let a: Address = c.address.parse().unwrap();
        // Before it exists it runs nothing; after, only what its template listed.
        assert!(may_run(&u, &a, "mining-drone", UniverseTime::seconds(4)).is_err());
        assert!(may_run(&u, &a, "mining-drone", at).is_ok());
        assert!(may_run(&u, &a, "pirate", at)
            .unwrap_err()
            .contains("does not list"));
        let st = u.state_at(at);
        assert_eq!(
            st.get(&a, "name"),
            Some(&Value::Text("Mining drone".into()))
        );
        assert_eq!(st.get(&a, "tag.ship"), Some(&Value::Bool(true)));
        assert_eq!(r.state_at(at).get(&a, "ship.cargo_t"), Some(&Value::Int(0)));
        // A second one takes a number; the facts are its own, so no pack is needed to read them.
        let (c2, _) = create(
            &mut u,
            &mut r,
            &reg,
            &root,
            &t.templates[0],
            &root,
            None,
            Some("Ore 2"),
            at,
            vec![],
        )
        .unwrap();
        assert_eq!(c2.address, "u:m/drone:1");
        // Retire: kept, history intact, no programs.
        retire(
            &mut u,
            &a,
            Some("lost"),
            UniverseTime::seconds(9),
            &Publisher::new("gm", "retire", "1"),
        )
        .unwrap();
        assert!(may_run(&u, &a, "mining-drone", UniverseTime::seconds(9))
            .unwrap_err()
            .contains("retired"));
        assert!(
            may_run(&u, &a, "mining-drone", UniverseTime::seconds(8)).is_ok(),
            "the past is unchanged"
        );
        // Refusals at load.
        for (bad, why) in [
            (
                json!({"templates": [{"id": "x", "kind": "Ship", "label": "X"}]}),
                "lowercase word",
            ),
            (
                json!({"templates": [{"id": "x", "kind": "ship", "label": "X", "facts": {"retired": true}}]}),
                "set by create",
            ),
            (
                json!({"templates": [{"id": "x", "kind": "ship", "label": ""}]}),
                "needs a label",
            ),
            (
                json!({"templates": [{"id": "x", "kind": "ship", "label": "X", "hull": 1}]}),
                "unknown field",
            ),
        ] {
            let e = Templates::from_packs(&[("p".into(), "1".into(), bad)])
                .unwrap_err()
                .join("; ");
            assert!(e.contains(why), "{e}");
        }
    }
}
