//! `sse.reference.lookup`, the Look up block (matter pack design section 3A,
//! MP17a): the only door a flow has onto a fact about nature.
//!
//! Config: `{what?, property, conditions?}`.
//! - `what` names a sheet row (`h2o`, `constants/G`, `elements/fe`). Left
//!   out, the block takes it on a `what` input, so a flow can loop over
//!   substances.
//! - `property` names the value (`density`, `phases.latent_vaporisation`,
//!   `molar_mass`; a constant needs none).
//! - `conditions: true` adds `p_pa` and `t_k` inputs, so the answer is the
//!   phase there and that phase's own value.
//!
//! Outputs: `value` (its dimension from the sheet), `phase`, `known` (false
//! when the row lacks the property: the value is then 0, and the flow
//! should skip it) and `in_range` (false when the conditions lie outside
//! what the source covers). The note carries the "Why?": the row, the
//! property's path, the source with its DOI or URL, the evidence grade and
//! the packs that set the row.
//!
//! The block keeps no number of its own: it reads the sheet the kernel puts
//! in force for the call (`sse_flow::sheet`), and refuses to load without one.

use serde::Deserialize;
use serde_json::Value as J;
use sse_core::Dimension;
use sse_flow::{Atom, AtomDecl, AtomError, BlockClass, Call, CostClass, Out, Port, Shape, Val};

pub const ID: &str = "sse.reference.lookup";

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LookupCfg {
    #[serde(default)]
    pub what: Option<String>,
    #[serde(default)]
    pub property: String,
    #[serde(default)]
    pub conditions: bool,
}

fn port(name: &str, kind: &str, dim: Dimension, shape: Shape) -> Port {
    Port {
        name: name.into(),
        kind: kind.into(),
        dim,
        valid: None,
        shape: (shape != Shape::Number).then_some(shape),
        any_dim: false,
    }
}

pub struct Lookup {
    decl: AtomDecl,
}

impl Default for Lookup {
    fn default() -> Self {
        Self::new()
    }
}

impl Lookup {
    pub fn new() -> Self {
        let (inputs, outputs) = Self::ports_for(Dimension::NONE, false, false);
        Lookup {
            decl: AtomDecl {
                id: ID.into(),
                version: "1.0.0".into(),
                inputs,
                outputs,
                params: vec![],
                cost_class: CostClass::Trivial,
                references: vec![],
                explain: sse_flow::decl::ExplainText {
                    curious: "Looks a fact up on the master data sheet: the density of water, the value of G, the latent heat of melting.".into(),
                    science: "One read of the stacked master data sheet (packs bottom to top), at the stated pressure and temperature when given: the phase there and that phase's own value, with its source, evidence grade and range. Values are changed on the Reference screen, never on the block.".into(),
                    formula: String::new(),
                },
                class: Some(BlockClass::Engine),
                dims: None,
            },
        }
    }

    fn ports_for(dim: Dimension, wired_what: bool, conditions: bool) -> (Vec<Port>, Vec<Port>) {
        let mut ins = vec![];
        if wired_what {
            ins.push(port("what", "reference.row", Dimension::NONE, Shape::Ref));
        }
        if conditions {
            ins.push(port(
                "p_pa",
                "reference.pressure",
                Dimension::PRESSURE,
                Shape::Number,
            ));
            ins.push(port(
                "t_k",
                "reference.temperature",
                Dimension::TEMPERATURE,
                Shape::Number,
            ));
        }
        (
            ins,
            vec![
                port("value", "reference.value", dim, Shape::Number),
                port("phase", "reference.phase", Dimension::NONE, Shape::Choice),
                port("known", "reference.known", Dimension::NONE, Shape::Flag),
                port(
                    "in_range",
                    "reference.in_range",
                    Dimension::NONE,
                    Shape::Flag,
                ),
            ],
        )
    }

    pub fn cfg(config: Option<&J>) -> Result<LookupCfg, String> {
        let j = config.ok_or("Look up needs `property` (and usually `what`) in its config")?;
        let c: LookupCfg =
            serde_json::from_value(j.clone()).map_err(|e| format!("Look up: {e}"))?;
        if c.what.is_none() && c.property.is_empty() {
            return Err("Look up needs a property when `what` is wired in".into());
        }
        Ok(c)
    }

    fn checked(config: Option<&J>) -> Result<(LookupCfg, Dimension), String> {
        let c = Self::cfg(config)?;
        let sheet = sse_flow::sheet::now().ok_or(
            "Look up reads the master data sheet, and this stack has none (add the base-reference pack)",
        )?;
        if let Some(w) = &c.what {
            sheet.row_of(w).map_err(|e| format!("Look up: {e}"))?;
        }
        let dim = sheet
            .dimension(c.what.as_deref(), &c.property)
            .map_err(|e| format!("Look up: {e}"))?;
        Ok((c, dim))
    }
}

impl Atom for Lookup {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(format!(
            "{ID} is evaluated through its typed form"
        )))
    }
    fn ports(&self, config: Option<&J>) -> Option<(Vec<Port>, Vec<Port>)> {
        let (c, dim) = Self::checked(config).ok()?;
        Some(Self::ports_for(dim, c.what.is_none(), c.conditions))
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        Self::checked(config).map(|_| ())
    }
    fn eval_typed(&self, v: &[Val], _: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let c = Self::cfg(cx.config).map_err(AtomError)?;
        let sheet = sse_flow::sheet::now()
            .ok_or_else(|| AtomError("Look up has no master data sheet in force".into()))?;
        let mut it = v.iter();
        let what = match &c.what {
            Some(w) => w.clone(),
            None => it
                .next()
                .and_then(Val::text)
                .ok_or_else(|| AtomError("Look up: the `what` input is not a row".into()))?
                .to_string(),
        };
        let at = if c.conditions {
            let p = it.next().and_then(Val::num);
            let t = it.next().and_then(Val::num);
            Some(
                p.zip(t)
                    .ok_or_else(|| AtomError("Look up: p_pa and t_k must be numbers".into()))?,
            )
        } else {
            None
        };
        let r = sheet.look_up(&what, &c.property, at).map_err(AtomError)?;
        let note = serde_json::json!({
            "block": ID,
            "row": r.row,
            "property": r.property,
            "unit": r.unit,
            "known": r.known,
            "in_range": r.in_range,
            "phase": r.phase,
            "grade": r.grade,
            "source": r.source,
            "packs": r.packs,
            "edited": r.packs.len() > 1,
            "reason": r.reason,
        });
        Ok(Out {
            values: vec![
                Val::Number(if r.known { r.value } else { 0.0 }),
                Val::Choice(r.phase.unwrap_or_default()),
                Val::Flag(r.known),
                Val::Flag(r.in_range),
            ],
            note: Some(note),
            ..Default::default()
        })
    }
}

/// Every Look up node in a flow document: `(node id, what, property)`, for
/// the sheet's "used by" (design 3A: read off the wiring, never declared).
pub fn uses(flow: &J) -> Vec<(String, Option<String>, String)> {
    flow.get("nodes")
        .and_then(J::as_array)
        .into_iter()
        .flatten()
        .filter(|n| n.get("atom").and_then(J::as_str) == Some(ID))
        .filter_map(|n| {
            let c = Lookup::cfg(n.get("config")).ok()?;
            Some((n.get("id")?.as_str()?.to_string(), c.what, c.property))
        })
        .collect()
}
