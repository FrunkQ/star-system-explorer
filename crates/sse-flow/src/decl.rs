//! Atom declarations and the atom trait (H6.2).

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
