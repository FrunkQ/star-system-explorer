//! OV3, territory on map layers (territory overlays design 4 and 11): the
//! kernel's side of `hold`, `who_holds` and `holdings_in`.
//!
//! A layer is a pack's `provides.other.overlays[]` entry with `areas`; the
//! kernel reads its `by` (only `{relation: "holds"}` is read here; tag
//! groups, choices, facts, knowledge and reach sources come with OV10), its
//! reach between stars (`fill.between_stars.reach`) and its `weigh_by`.
//! Holdings live in the campaign record (`sse_record::holdings`); the world
//! gives the rules positions (drawn star systems, at the date asked, in the
//! galactic frame) and weights (a kept number named by `weigh_by`).

use crate::{err, Response};
use sse_core::cbor::Value;
use sse_core::Address;
use sse_record::holdings::{Distance, LayerRules, World};
use std::collections::BTreeMap;

/// GM-made layers need no pack entry, like `misc.*` kept values.
pub const MISC_LAYER: &str = "misc.";

/// One map layer of areas, as the kernel reads it.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Layer {
    pub pack: String,
    pub label: String,
    pub rules: LayerRules,
}

/// Every area layer of the stack, by overlay id.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Layers {
    pub layers: BTreeMap<String, Layer>,
}

impl Layers {
    /// Read `provides.other.overlays[]` entries that carry `areas`. A later
    /// pack's entry of the same id replaces an earlier one.
    pub fn from_packs(packs: &[(String, serde_json::Value)]) -> Result<Layers, Vec<String>> {
        let mut out = Layers::default();
        let mut problems = vec![];
        for (pack, v) in packs {
            for o in v.as_array().into_iter().flatten() {
                let Some(areas) = o.get("areas") else {
                    continue;
                };
                let id = o["id"].as_str().unwrap_or("").to_string();
                let at = format!("{pack}: overlay {id:?}");
                if id.is_empty() {
                    problems.push(format!("{at}: an overlay needs an id"));
                    continue;
                }
                let by = &areas["by"];
                if !by.is_object() {
                    problems.push(format!("{at}: areas.by says what is read at each place"));
                }
                let reach = &areas["fill"]["between_stars"]["reach"];
                let reach_m = if reach.is_null() {
                    None
                } else {
                    match serde_json::from_value::<Distance>(reach.clone())
                        .map_err(|e| e.to_string())
                        .and_then(|d| d.metres())
                    {
                        Ok(m) => Some(m),
                        Err(e) => {
                            problems.push(format!("{at}: fill.between_stars.reach: {e}"));
                            None
                        }
                    }
                };
                let weigh_by = match &areas["weigh_by"] {
                    serde_json::Value::Null => None,
                    serde_json::Value::String(s) if s == "count" => None,
                    serde_json::Value::String(s) => Some(s.clone()),
                    _ => {
                        problems.push(format!("{at}: weigh_by is a kept number's id or \"count\""));
                        None
                    }
                };
                out.layers.insert(
                    id.clone(),
                    Layer {
                        pack: pack.clone(),
                        label: o["label"].as_str().unwrap_or(&id).to_string(),
                        rules: LayerRules { reach_m, weigh_by },
                    },
                );
            }
        }
        if problems.is_empty() {
            Ok(out)
        } else {
            Err(problems)
        }
    }

    /// The rules for a layer: its pack entry's, or none for a GM's own
    /// `misc.` layer; an unknown layer is refused.
    pub fn rules(&self, layer: &str) -> Result<LayerRules, Response> {
        match self.layers.get(layer) {
            Some(l) => Ok(l.rules.clone()),
            None if layer.starts_with(MISC_LAYER) => Ok(LayerRules::default()),
            None => Err(err(
                "unknown_layer",
                format!(
                    "no map layer {layer:?} in the pack stack (a pack's provides.other.overlays with areas, or a GM's misc.* layer)"
                ),
            )),
        }
    }
}

/// The campaign world as the reading rules see it.
pub struct KernelWorld {
    /// Positions in metres, galactic frame, at the date asked.
    pub positions: BTreeMap<Address, [f64; 3]>,
    pub record: sse_chronicle::State,
}

impl World for KernelWorld {
    fn position(&self, place: &Address) -> Option<[f64; 3]> {
        // A place sits where its star system is.
        let mut a = Some(place.clone());
        while let Some(x) = a {
            if let Some(p) = self.positions.get(&x) {
                return Some(*p);
            }
            a = x.parent();
        }
        None
    }
    fn weight(&self, place: &Address, kind: &str) -> Option<f64> {
        match self.record.get(place, kind)? {
            Value::Float(x) => Some(*x),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
}

/// The drawn star system a place sits in, if any (`.../sys:f<frame>k<slot>`
/// or `...h<slot>`).
pub fn drawn_system(place: &Address) -> Option<Address> {
    let mut a = Some(place.clone());
    while let Some(x) = a {
        let s = x.last();
        if s.kind() == "sys" && s.key().starts_with('f') {
            return Some(x);
        }
        a = x.parent();
    }
    None
}
