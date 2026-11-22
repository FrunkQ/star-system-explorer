//! `sse.model.number`, the Model number block (equation block design 4): a
//! number that belongs to the model, not to nature (a fitted slope, an
//! exponent from a paper, how much of a sphere shares the heat), in its own
//! small block on the flow, so every letter of an Equation has a visible
//! line from a source.
//!
//! Config: `{name, value, unit, range?, grade, source}`.
//! - `value` in `unit`; `range` (same unit) is where the model allows it,
//!   and a value outside is refused at load.
//! - `grade`: `derived`, `measured`, `fitted` or `assumed`. Fitted and
//!   assumed numbers are the pack's fudges (ND6 lists them).
//! - `source`: `{text, doi | url, where?}`.
//!
//! The block is the editable control: the GM or Orac changes the value here,
//! through a change request, and every Equation it feeds follows. Constants
//! of nature are not model numbers: they come from the data sheet through a
//! Look up block.

use serde::Deserialize;
use serde_json::{json, Value as J};
use sse_core::quantity::parse_unit;
use sse_core::Dimension;
use sse_flow::{Atom, AtomDecl, AtomError, BlockClass, Call, CostClass, Out, Port, Val};

pub const ID: &str = "sse.model.number";
pub const GRADES: [&str; 4] = ["derived", "measured", "fitted", "assumed"];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCfg {
    pub name: String,
    pub value: f64,
    pub unit: String,
    #[serde(default)]
    pub range: Option<(f64, f64)>,
    pub grade: String,
    pub source: crate::equation::SourceSpec,
    /// What quantity kind the wire carries (default `model.number`).
    #[serde(default)]
    pub kind: Option<String>,
}

impl ModelCfg {
    pub fn parse(config: Option<&J>) -> Result<(ModelCfg, f64, Dimension), String> {
        let j = config.ok_or("a Model number needs name, value, unit, grade and source")?;
        let c: ModelCfg =
            serde_json::from_value(j.clone()).map_err(|e| format!("Model number: {e}"))?;
        let me = format!("Model number {:?}", c.name);
        if c.name.split_whitespace().count() < 2 {
            return Err(format!("{me}: its name says what the number is, in words"));
        }
        if !GRADES.contains(&c.grade.as_str()) {
            return Err(format!("{me}: grade is one of {}", GRADES.join(", ")));
        }
        if c.source.doi.as_deref().is_none_or(str::is_empty)
            && c.source.url.as_deref().is_none_or(str::is_empty)
        {
            return Err(format!("{me}: its source needs a DOI or an address"));
        }
        if !c.value.is_finite() {
            return Err(format!("{me}: the value is a finite number"));
        }
        if let Some((lo, hi)) = c.range {
            if lo > hi || c.value < lo || c.value > hi {
                return Err(format!(
                    "{me}: {} is outside its allowed range {lo} to {hi}",
                    c.value
                ));
            }
        }
        let (s, d) = parse_unit(&c.unit).map_err(|e| format!("{me}: {e}"))?;
        let si = c.value * s;
        Ok((c, si, d))
    }
}

pub struct ModelNumber {
    decl: AtomDecl,
}

impl Default for ModelNumber {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelNumber {
    pub fn new() -> Self {
        ModelNumber {
            decl: AtomDecl {
                id: ID.into(),
                version: "1.0.0".into(),
                inputs: vec![],
                outputs: vec![],
                params: vec![],
                cost_class: CostClass::Trivial,
                references: vec![],
                explain: sse_flow::decl::ExplainText {
                    curious: "A number the model needs (a fitted slope, a share of the sky), with where it comes from.".into(),
                    science: "A number that belongs to the model, not to nature: value, unit, allowed range, grade (derived, measured, fitted, assumed) and its source. Fitted and assumed numbers are listed among the pack's fudges.".into(),
                    formula: String::new(),
                },
                class: Some(BlockClass::Engine),
                dims: None,
            },
        }
    }
}

impl Atom for ModelNumber {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(format!(
            "{ID} is evaluated through its typed form"
        )))
    }
    fn ports(&self, config: Option<&J>) -> Option<(Vec<Port>, Vec<Port>)> {
        let (c, _, d) = ModelCfg::parse(config).ok()?;
        let scale = parse_unit(&c.unit).ok()?.0;
        Some((
            vec![],
            vec![Port {
                name: "value".into(),
                kind: c.kind.unwrap_or_else(|| "model.number".into()),
                dim: d,
                valid: c.range.map(|(lo, hi)| (lo * scale, hi * scale)),
                shape: None,
                any_dim: false,
            }],
        ))
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        ModelCfg::parse(config).map(|_| ())
    }
    fn eval_typed(&self, _: &[Val], _: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let (c, si, _) = ModelCfg::parse(cx.config).map_err(AtomError)?;
        Ok(Out {
            values: vec![Val::Number(si)],
            note: Some(json!({
                "block": ID,
                "name": c.name,
                "value": c.value,
                "unit": c.unit,
                "range": c.range,
                "grade": c.grade,
                "fudge": c.grade == "fitted" || c.grade == "assumed",
                "source": {"text": c.source.text, "doi": c.source.doi, "url": c.source.url, "where": c.source.at},
            })),
            ..Default::default()
        })
    }
}
