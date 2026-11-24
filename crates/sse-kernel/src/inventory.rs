//! `read_inventory` (TG11, MP8b): how much of each resource a world holds,
//! in kilograms, read off its composition (MP6) at Survey. A resource kind
//! is pack data (`provides.other.resources`): an id, a name, the matter it
//! is (an element id or `h2o` on the sheet; none for a made good) and its
//! reach (crust, sea, air, surface or whole). Any element id or `h2o` is a
//! kind by itself, so a resource nobody declared still has an inventory.
//! No dice: the same world always holds the same amounts. What is mined
//! out (`extracted`) and what is brought in (`stock`) are kept values on
//! top, owned by the record (the Traveller pack thread).

use crate::adopt::worlds_at;
use crate::err;
use crate::protocol::Response;
use serde::{Deserialize, Serialize};
use sse_chronicle::Chronicle;
use sse_core::Address;
use sse_formation::pipeline::SystemRun;
use sse_surface::composition::{compose, Composition, Level, Reservoirs, Settings, Sheet};
use std::collections::BTreeMap;

/// Where a resource can be taken from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reach {
    Crust,
    Sea,
    Air,
    /// Crust, sea and air.
    #[default]
    Surface,
    /// The whole body, core and mantle included.
    Whole,
}

impl Reach {
    fn of(self, r: &Reservoirs) -> f64 {
        match self {
            Reach::Crust => r.crust_kg,
            Reach::Sea => r.sea_kg,
            Reach::Air => r.air_kg,
            Reach::Surface => r.crust_kg + r.sea_kg + r.air_kg,
            Reach::Whole => r.total(),
        }
    }
}

/// A resource kind (pack data).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kind {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// An element id or `h2o`; none for a made good (stock only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matter: Option<String>,
    #[serde(default)]
    pub reach: Reach,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    #[serde(default)]
    kinds: Vec<Kind>,
}

/// The stack's resource kinds by id (a later pack's kind of the same id wins).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Kinds {
    pub kinds: BTreeMap<String, (String, Kind)>,
}

impl Kinds {
    pub fn from_packs(found: &[(String, serde_json::Value)]) -> Result<Self, Vec<String>> {
        let mut out = Kinds::default();
        let mut problems = vec![];
        for (pack, v) in found {
            let b: Block = match serde_json::from_value(v.clone()) {
                Ok(b) => b,
                Err(e) => {
                    problems.push(format!("{pack}: resources: {e}"));
                    continue;
                }
            };
            for k in b.kinds {
                let ok = !k.id.is_empty()
                    && k.id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
                if !ok {
                    problems.push(format!(
                        "{pack}: resource kind {:?}: ids are lower case letters, digits and dashes",
                        k.id
                    ));
                    continue;
                }
                out.kinds.insert(k.id.clone(), (pack.clone(), k));
            }
        }
        if problems.is_empty() {
            Ok(out)
        } else {
            Err(problems)
        }
    }
}

/// One kind's amount on one world.
fn amount(c: &Composition, k: &Kind) -> serde_json::Value {
    let mass = c.bulk.mass_kg.max(f64::MIN_POSITIVE);
    let base = serde_json::json!({
        "id": k.id,
        "name": k.name,
        "matter": k.matter,
        "reach": k.reach,
    });
    let mut o = base.as_object().cloned().unwrap_or_default();
    let (kg, by, note): (f64, serde_json::Value, Option<String>) = match k.matter.as_deref() {
        None => (
            0.0,
            serde_json::Value::Null,
            Some("a made good: nothing natural, only stock".into()),
        ),
        Some("h2o") => {
            let b = &c.bulk;
            let sea = b.surface_water_kg;
            let kg = match k.reach {
                Reach::Sea | Reach::Surface => sea,
                Reach::Whole => b.water_kg,
                Reach::Crust | Reach::Air => 0.0,
            };
            (
                kg,
                serde_json::json!({"sea_kg": sea, "interior_kg": (b.water_kg - sea).max(0.0)}),
                None,
            )
        }
        Some(id) => match c.elements.iter().find(|e| e.id == id) {
            Some(e) => {
                let r = e.reservoirs.clone().unwrap_or_default();
                (
                    k.reach.of(&r),
                    serde_json::to_value(&r).unwrap_or_default(),
                    None,
                )
            }
            None => (
                0.0,
                serde_json::Value::Null,
                Some(format!(
                    "{id} is neither an element on the sheet nor water: not counted yet"
                )),
            ),
        },
    };
    o.insert("kg".into(), kg.into());
    o.insert("per_kg".into(), (kg / mass).into());
    o.insert("by_reservoir".into(), by);
    if let Some(n) = note {
        o.insert("note".into(), n.into());
    }
    serde_json::Value::Object(o)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn read(
    ch: &Chronicle,
    system: &Address,
    body: Option<&str>,
    kinds: Option<&[String]>,
    at_s: Option<i64>,
    run: Option<&SystemRun>,
    sheet: &sse_reference::Reference,
    declared: &Kinds,
) -> Result<Response, Response> {
    let rows = Sheet::read(sheet);
    let is_matter = |id: &str| id == "h2o" || rows.elements.iter().any(|e| e.id == id);
    let wanted: Vec<Kind> = match kinds {
        Some(ids) => {
            let mut out = vec![];
            for id in ids {
                if let Some((_, k)) = declared.kinds.get(id) {
                    out.push(k.clone());
                } else if is_matter(id) {
                    out.push(Kind {
                        id: id.clone(),
                        name: None,
                        matter: Some(id.clone()),
                        reach: Reach::Surface,
                        about: None,
                    });
                } else {
                    return Err(err(
                        "unknown_resource",
                        format!(
                            "{id:?} is not a resource kind of this stack, an element id on the sheet or h2o"
                        ),
                    ));
                }
            }
            out
        }
        None => declared.kinds.values().map(|(_, k)| k.clone()).collect(),
    };
    if wanted.is_empty() {
        return Err(err(
            "no_kinds",
            "name kinds: the stack declares no resource kinds (provides.other.resources)",
        ));
    }
    let want: Option<Address> = match body {
        Some(b) => Some(b.parse().map_err(|e| err("bad_address", format!("{e}")))?),
        None => None,
    };
    let ss = sse_surface::scenario::settings();
    let w = worlds_at(ch, system, at_s, run, &ss)?;
    let settings = Settings::default();
    let ratio = crate::composition::star_ratio(run, sheet)?;
    let mut out = vec![];
    for b in &w.bodies {
        if want.as_ref().is_some_and(|a| *a != b.address) {
            continue;
        }
        let c = compose(
            &rows,
            &crate::composition::bulk_of(b, run),
            &ratio,
            Level::Survey,
            &settings,
        );
        let mut stand_ins = vec![
            "natural amounts only: what has been mined out and what has been brought in are kept values on top".to_string(),
            "deposits are part of the crust: survey_surface's holdings say how much of it is ore".to_string(),
        ];
        if b.makeup.as_ref().is_some_and(|m| m.gas > 0.0) || b.deep_envelope {
            stand_ins.push("a gas envelope's hydrogen and helium are not counted yet".into());
        }
        out.push(serde_json::json!({
            "body": b.address.to_string(),
            "label": b.label,
            "mass_kg": c.bulk.mass_kg,
            "source": c.bulk.source,
            "kinds": wanted.iter().map(|k| amount(&c, k)).collect::<Vec<_>>(),
            "stand_ins": stand_ins,
        }));
    }
    if let (Some(a), true) = (&want, out.is_empty()) {
        return Err(err(
            "unknown_body",
            format!("{a} is not a world of {system} with a mass and a radius"),
        ));
    }
    Ok(Response::InventoryRead { bodies: out })
}
