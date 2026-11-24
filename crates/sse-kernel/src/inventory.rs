//! `read_inventory` (TG11, MP8b): how much of each resource a world holds,
//! in kilograms, read off its composition (MP6) at Survey. A resource kind
//! is pack data (`provides.other.resources`): an id, a name, the matter it
//! is (an element id, an isotope id such as `he3`, or `h2o` on the sheet;
//! none for a made good) and its
//! reach (crust, sea, air, surface or whole). Any element id or `h2o` is a
//! kind by itself, so a resource nobody declared still has an inventory.
//! No dice: the same world always holds the same amounts. What is mined
//! out (`extracted`) and what is brought in (`stock`) are kept values on
//! top (record-logic design 2.4, X3j): every resource kind gets the two kept
//! kinds `resource.<kind>.extracted` and `resource.<kind>.stock`, on any
//! address, and what a place holds is natural - extracted + stock. Goods
//! move only by `transfer`, which conserves them.

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
    /// A giant's hydrogen and helium envelope (gas skimming).
    Envelope,
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
            Reach::Envelope => r.envelope_kg,
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

/// The record's kept kind for what has been taken from a place's natural
/// amount of `kind`, and for what has been put or made there.
pub fn extracted_kind(kind: &str) -> String {
    format!("resource.{kind}.extracted")
}
pub fn stock_kind(kind: &str) -> String {
    format!("resource.{kind}.stock")
}

/// The two kept kinds every resource kind gets (record-logic design 2.4).
/// Writers: the GM and `transfer`; a pack's own effects name them to join.
pub fn kept_kinds(kinds: &Kinds) -> Vec<sse_record::KeptKind> {
    let mut out = vec![];
    for (id, (_, k)) in &kinds.kinds {
        let name = k.name.clone().unwrap_or_else(|| id.clone());
        for (kept, describes) in [
            (
                extracted_kind(id),
                format!("{name} taken from this place's natural amount, kg"),
            ),
            (stock_kind(id), format!("{name} put or made here, kg")),
        ] {
            out.push(sse_record::KeptKind {
                id: kept,
                shape: sse_record::Shape::Number,
                unit: Some("kg".into()),
                range: Some((0.0, f64::MAX)),
                choices: vec![],
                default: None,
                tier: "surveyed".into(),
                writers: vec!["gm".into(), "transfer".into()],
                describes,
            });
        }
    }
    out
}

/// A kept number in kilograms (absent: 0).
pub fn kept_kg(state: &sse_chronicle::State, a: &Address, kind: &str) -> f64 {
    match state.get(a, kind) {
        Some(sse_core::cbor::Value::Float(f)) => *f,
        Some(sse_core::cbor::Value::Int(i)) => *i as f64,
        _ => 0.0,
    }
}

/// Natural, extracted, stock and held kilograms of `kind` at `a`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Held {
    pub natural_kg: f64,
    pub extracted_kg: f64,
    pub stock_kg: f64,
}
impl Held {
    pub fn at(state: &sse_chronicle::State, a: &Address, kind: &str, natural_kg: f64) -> Held {
        Held {
            natural_kg,
            extracted_kg: kept_kg(state, a, &extracted_kind(kind)),
            stock_kg: kept_kg(state, a, &stock_kind(kind)),
        }
    }
    pub fn held_kg(&self) -> f64 {
        (self.natural_kg - self.extracted_kg).max(0.0) + self.stock_kg
    }
    /// Take `kg` away: stock first, then the natural amount. The new
    /// (extracted, stock), or None when the place holds less.
    pub fn take(&self, kg: f64) -> Option<(f64, f64)> {
        if kg > self.held_kg() {
            return None;
        }
        let from_stock = kg.min(self.stock_kg);
        Some((
            self.extracted_kg + (kg - from_stock),
            self.stock_kg - from_stock,
        ))
    }
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"natural_kg": self.natural_kg, "extracted_kg": self.extracted_kg,
            "stock_kg": self.stock_kg, "held_kg": self.held_kg()})
    }
}

/// An isotope resource: its element and its mass share of that element
/// (the sheet's protosolar atom fraction), or why it cannot be counted.
type Isotope = Option<Result<(String, f64), String>>;

fn amount(c: &Composition, k: &Kind, iso: &Isotope) -> serde_json::Value {
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
                Reach::Crust | Reach::Air | Reach::Envelope => 0.0,
            };
            (
                kg,
                serde_json::json!({"sea_kg": sea, "interior_kg": (b.water_kg - sea).max(0.0)}),
                None,
            )
        }
        Some(_) if iso.is_some() => match iso {
            Some(Ok((el, share))) => {
                let r = c
                    .elements
                    .iter()
                    .find(|e| e.id == *el)
                    .and_then(|e| e.reservoirs.clone())
                    .unwrap_or_default();
                let scaled = Reservoirs {
                    core_kg: r.core_kg * share,
                    mantle_kg: r.mantle_kg * share,
                    crust_kg: r.crust_kg * share,
                    sea_kg: r.sea_kg * share,
                    air_kg: r.air_kg * share,
                    envelope_kg: r.envelope_kg * share,
                };
                (
                    k.reach.of(&scaled),
                    serde_json::to_value(&scaled).unwrap_or_default(),
                    Some(format!(
                        "{share:.3e} of the world's {el} by mass, at the birth cloud's isotope share"
                    )),
                )
            }
            Some(Err(why)) => (0.0, serde_json::Value::Null, Some(why.clone())),
            None => unreachable!(),
        },
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
    kept: &sse_chronicle::State,
) -> Result<Response, Response> {
    let rows = Sheet::read(sheet);
    let is_isotope = |id: &str| sheet.row("isotopes", id).is_some();
    let is_matter =
        |id: &str| id == "h2o" || rows.elements.iter().any(|e| e.id == id) || is_isotope(id);
    let isotope_of = |k: &Kind| -> Isotope {
        let id = k.matter.as_deref()?;
        if !is_isotope(id) {
            return None;
        }
        Some(rows.isotope_mass_share(sheet, id).ok_or_else(|| {
            format!("the sheet does not carry {id}'s protosolar share yet: not counted")
        }))
    };
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
        let z = w
            .spec
            .as_ref()
            .map(|s| s.z)
            .unwrap_or(crate::adopt::DEFAULT_Z);
        let c = compose(
            &rows,
            &crate::composition::bulk_of(b, run, z),
            &ratio,
            Level::Survey,
            &settings,
        );
        let stand_ins = vec![
            "deposits are part of the crust: survey_surface's holdings say how much of it is ore"
                .to_string(),
        ];
        let kinds: Vec<serde_json::Value> = wanted
            .iter()
            .map(|k| {
                let mut v = amount(&c, k, &isotope_of(k));
                let natural = v["kg"].as_f64().unwrap_or(0.0);
                let h = Held::at(kept, &b.address, &k.id, natural);
                if let (Some(o), serde_json::Value::Object(extra)) = (v.as_object_mut(), h.json()) {
                    o.extend(extra);
                }
                v
            })
            .collect();
        out.push(serde_json::json!({
            "body": b.address.to_string(),
            "label": b.label,
            "mass_kg": c.bulk.mass_kg,
            "source": c.bulk.source,
            "kinds": kinds,
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
