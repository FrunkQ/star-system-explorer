//! Atom declarations and the atom trait (H6.2).

use crate::value::{Shape, Val};
use serde::{Deserialize, Serialize};
use sse_core::Dimension;
use std::collections::BTreeMap;
use std::fmt;

/// How expensive an operation is, in deterministic work units (build plan
/// section 2). Devices whose capability tier is below an operation's class do
/// less or ask the GM machine for recorded results.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CostClass {
    /// Closed-form arithmetic: under ~1e3 work units.
    Trivial,
    /// Small iterative work: under ~1e6.
    Light,
    /// Under ~1e9: seconds on a phone.
    Medium,
    /// Above: GM machine or capable devices only.
    Heavy,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Port {
    pub name: String,
    /// Default quantity kind this port carries.
    pub kind: String,
    pub dim: Dimension,
    /// Range (SI) where the physics of the atom holds, if bounded.
    #[serde(default)]
    pub valid: Option<(f64, f64)>,
    /// What the port carries (J2); absent means a number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<Shape>,
    /// A logic number port that takes any dimension (`dims` on the atom says
    /// how outputs follow); `dim` is then ignored.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub any_dim: bool,
}

impl Port {
    pub fn shape(&self) -> Shape {
        self.shape.clone().unwrap_or(Shape::Number)
    }
}

/// How a logic atom's any-dimension number outputs follow its inputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DimRule {
    /// All any-dimension number inputs share one dimension; outputs have it.
    Same,
    /// Output = first input x second input.
    Product,
    /// Output = first input / second input.
    Quotient,
}

/// What kind of block an atom is (J2, record design 3.2 and 3.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockClass {
    /// Logic: no citation rule; drawn differently from physics.
    Logic,
    /// A dedicated engine behind a block face, with its own editor.
    Engine,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub dim: Dimension,
    pub default: f64,
    /// Allowed range (SI); outside it the flow is rejected at validation.
    #[serde(default)]
    pub range: Option<(f64, f64)>,
    pub description: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct ExplainText {
    /// Curious lens: no background assumed.
    pub curious: String,
    /// Science lens: mechanism and assumptions.
    pub science: String,
    /// Formula in display form.
    pub formula: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AtomDecl {
    pub id: String,
    pub version: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    #[serde(default)]
    pub params: Vec<Param>,
    pub cost_class: CostClass,
    /// Citations. Real physics atoms must have at least one (P1 rule).
    #[serde(default)]
    pub references: Vec<Reference>,
    pub explain: ExplainText,
    /// Absent: a physics atom on numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<BlockClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dims: Option<DimRule>,
}

impl AtomDecl {
    pub fn input(&self, n: &str) -> Option<(usize, &Port)> {
        self.inputs.iter().enumerate().find(|(_, p)| p.name == n)
    }
    pub fn output(&self, n: &str) -> Option<(usize, &Port)> {
        self.outputs.iter().enumerate().find(|(_, p)| p.name == n)
    }
    pub fn param(&self, n: &str) -> Option<(usize, &Param)> {
        self.params.iter().enumerate().find(|(_, p)| p.name == n)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomError(pub String);
impl fmt::Display for AtomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A compiled, tested operator. Inputs, params and outputs are SI values in
/// the order of the declaration.
pub trait Atom: Send + Sync {
    fn decl(&self) -> &AtomDecl;
    fn eval(&self, inputs: &[f64], params: &[f64]) -> Result<Vec<f64>, AtomError>;
    /// Typed evaluation (logic and engine blocks). The default reads numbers
    /// and calls `eval`, so physics atoms need nothing new.
    fn eval_typed(&self, inputs: &[Val], params: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let _ = cx;
        let xs: Vec<f64> = inputs
            .iter()
            .map(|v| v.num().ok_or_else(|| AtomError("expected a number".into())))
            .collect::<Result<_, _>>()?;
        Ok(Out::values(
            self.eval(&xs, params)?
                .into_iter()
                .map(Val::Number)
                .collect(),
        ))
    }
    /// A node's own ports when its config sets them (the shape a read gives,
    /// how many inputs an `and` takes). `None`: the declaration's ports.
    fn ports(&self, config: Option<&serde_json::Value>) -> Option<(Vec<Port>, Vec<Port>)> {
        let _ = config;
        None
    }
    /// Check a node's `config` at validation (logic and engine blocks).
    fn check_config(&self, config: Option<&serde_json::Value>) -> Result<(), String> {
        match config {
            None => Ok(()),
            Some(_) => Err(format!("{} takes no config", self.decl().id)),
        }
    }
}

/// What a typed call returns.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Out {
    pub values: Vec<Val>,
    /// Inputs the result depended on (in order), for evidence; `None`: all.
    /// `and` stops at its first false input, `or` at its first true one,
    /// `select` uses the condition and the chosen branch.
    pub used: Option<Vec<usize>>,
    /// World facts this block read: (field or `tag:<name>`, value).
    pub reads: Vec<(String, sse_core::cbor::Value)>,
    /// An engine block's own explanation (a class label's axes and region).
    pub note: Option<serde_json::Value>,
}
impl Out {
    pub fn values(values: Vec<Val>) -> Self {
        Out {
            values,
            ..Default::default()
        }
    }
}

/// What logic blocks may read of the world: one subject's facts (and its
/// parent's, as `parent.<field>`), tags, other addresses' facts, children.
/// The record is never offered to physics (record design 2.2.1).
pub trait World {
    fn fact(
        &self,
        address: Option<&sse_core::Address>,
        field: &str,
    ) -> Option<sse_core::cbor::Value>;
    fn has_tag(&self, address: Option<&sse_core::Address>, tag: &str) -> bool;
    fn children(&self, _address: &sse_core::Address) -> Vec<sse_core::Address> {
        Vec::new()
    }
    /// Every fact of one address (the subject when `None`), for engine
    /// blocks that read a whole subject (the classifier).
    fn facts(
        &self,
        _address: Option<&sse_core::Address>,
    ) -> BTreeMap<String, sse_core::cbor::Value> {
        BTreeMap::new()
    }
    /// The facts of that address's parent (read as `parent.<field>`).
    fn parent_facts(
        &self,
        _address: Option<&sse_core::Address>,
    ) -> BTreeMap<String, sse_core::cbor::Value> {
        BTreeMap::new()
    }
    /// Every tag on that address.
    fn tags(&self, _address: Option<&sse_core::Address>) -> std::collections::BTreeSet<String> {
        std::collections::BTreeSet::new()
    }
    /// A kept value from the record (GM machine only; never physics).
    fn kept(
        &self,
        _address: Option<&sse_core::Address>,
        _kind: &str,
    ) -> Option<sse_core::cbor::Value> {
        None
    }
}

/// The environment of one typed evaluation.
pub struct Env<'a> {
    pub world: Option<&'a dyn World>,
    /// The subject facts are read from by default.
    pub subject: Option<sse_core::Address>,
    /// Universe time of the reading, seconds.
    pub now_s: i64,
    /// Hysteresis: the widened form for staying in a state once entered.
    pub staying: bool,
    /// The universe seed, for engine blocks that resolve the universe
    /// (placement). Never used for dice.
    pub seed: Option<sse_core::Seed>,
    /// The table secret for dice (GM machine only); none: rolls refuse.
    pub table_secret: Option<sse_core::Seed>,
    /// The tick or turn index, the dice counter.
    pub counter: u64,
}
impl Env<'_> {
    pub fn empty() -> Env<'static> {
        Env {
            world: None,
            subject: None,
            now_s: 0,
            staying: false,
            seed: None,
            table_secret: None,
            counter: 0,
        }
    }
}

/// One node's call: its environment, config and identity.
pub struct Call<'a, 'e> {
    pub env: &'a Env<'e>,
    pub config: Option<&'a serde_json::Value>,
    pub flow_id: &'a str,
    pub node_id: &'a str,
}

/// The atoms known to a kernel.
#[derive(Default)]
pub struct AtomRegistry {
    atoms: BTreeMap<String, Box<dyn Atom>>,
}

impl AtomRegistry {
    pub fn insert(&mut self, a: Box<dyn Atom>) {
        self.atoms.insert(a.decl().id.clone(), a);
    }
    pub fn get(&self, id: &str) -> Option<&dyn Atom> {
        self.atoms.get(id).map(|b| b.as_ref())
    }
    pub fn decls(&self) -> impl Iterator<Item = &AtomDecl> {
        self.atoms.values().map(|a| a.decl())
    }
}

/// A citation: readable text plus, where one exists, a DOI and a link, so an
/// explain panel can send a reader to the paper. Serialised as
/// `{ text, doi?, url? }`; a bare string is accepted on read.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Reference {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Reference {
    /// Split a citation written as `"<text>. doi:<doi>"` or `"<text>. <https url>"`.
    /// A DOI also gets its resolver link. Text with neither stays as it is.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if let Some(i) = s.rfind(" doi:") {
            let doi = s[i + 5..].trim().to_string();
            return Reference {
                text: s[..i].trim().to_string(),
                url: Some(format!("https://doi.org/{doi}")),
                doi: Some(doi),
            };
        }
        if let Some(i) = s.rfind(" https://") {
            return Reference {
                text: s[..i].trim().to_string(),
                doi: None,
                url: Some(s[i + 1..].trim().to_string()),
            };
        }
        Reference {
            text: s.to_string(),
            doi: None,
            url: None,
        }
    }

    /// True when a reader can follow the citation to its source.
    pub fn linked(&self) -> bool {
        self.doi.is_some() || self.url.is_some()
    }
}

impl From<&str> for Reference {
    fn from(s: &str) -> Self {
        Reference::parse(s)
    }
}

impl<'de> Deserialize<'de> for Reference {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Full {
            text: String,
            #[serde(default)]
            doi: Option<String>,
            #[serde(default)]
            url: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            Text(String),
            Full(Full),
        }
        Ok(match Either::deserialize(d)? {
            Either::Text(s) => Reference::parse(&s),
            Either::Full(f) => Reference {
                text: f.text,
                doi: f.doi,
                url: f.url,
            },
        })
    }
}
