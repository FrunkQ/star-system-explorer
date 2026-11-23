//! What a pack says about its own trust (ND6, new-physics demo 2.5): the
//! sources it cites, the fudges it makes and what each stands in for, and
//! the published numbers it claims to reproduce. The kernel runs every
//! reproduction check when the stack loads, so the report is worked out on
//! the reader's machine, not claimed by the author, and a pack whose checks
//! fail is refused with the numbers side by side.
//!
//! All three are left out of a pack's JSON when empty, so a pack without
//! them hashes as it always did.

use serde::{Deserialize, Serialize};
use sse_flow::ParamValue;
use std::collections::{BTreeMap, BTreeSet};

/// A source the pack cites: a paper, a table, a data set.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Short id the pack's fudges and checks refer to (`hollands2026`).
    pub id: String,
    /// The citation in words.
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The source's own licence, when it is not the pack's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
}

/// A simplification the pack makes, said plainly.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fudge {
    pub id: String,
    /// What the pack does ("a reduced model of the sodium line wings").
    pub what: String,
    /// What it stands in for ("the paper's full model atmospheres").
    pub stands_in_for: String,
    /// What would replace it, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace_with: Option<String>,
    /// Flows or boxes of this pack it lives in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flows: Vec<String>,
    /// A source id, when the fudge itself comes from one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// A number a source published that the pack's flow should give back.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Published {
    pub value: f64,
    /// The unit the source gives it in; SI when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Relative: 0.05 is within 5 %. The pack's own stated closeness.
    pub tolerance: f64,
}

/// One reproduction check: a case from a source, run through a flow.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    /// What is being reproduced ("Sun-like model, 5800 K").
    pub name: String,
    /// A source id from the pack's `sources`.
    pub source: String,
    /// Where in the source ("Table 2", "Fig. 4").
    #[serde(rename = "where", default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// The flow to run, from any pack in the stack.
    pub flow: String,
    pub inputs: BTreeMap<String, ParamValue>,
    /// Published values by quantity kind.
    pub expect: BTreeMap<String, Published>,
}

/// The shape of the three lists, checked with the stack (they need no
/// atoms). Returns problems in words.
pub fn check_shape(
    sources: &[Source],
    fudges: &[Fudge],
    checks: &[Check],
    own: &BTreeSet<String>,
) -> Vec<String> {
    let mut out = vec![];
    let mut ids = BTreeSet::new();
    for s in sources {
        if !ids.insert(s.id.as_str()) {
            out.push(format!("source {} appears twice", s.id));
        }
        if s.text.trim().is_empty() {
            out.push(format!("source {} says what it is in words", s.id));
        }
        let linked = |x: &Option<String>| x.as_deref().is_some_and(|t| !t.trim().is_empty());
        if !linked(&s.doi) && !linked(&s.url) {
            out.push(format!("source {} has a DOI or an address", s.id));
        }
    }
    let mut fids = BTreeSet::new();
    for f in fudges {
        if !fids.insert(f.id.as_str()) {
            out.push(format!("fudge {} appears twice", f.id));
        }
        if f.what.trim().is_empty() || f.stands_in_for.trim().is_empty() {
            out.push(format!(
                "fudge {} says what it does and what it stands in for",
                f.id
            ));
        }
        if let Some(s) = &f.source {
            if !ids.contains(s.as_str()) {
                out.push(format!(
                    "fudge {} cites {s}, which is not among the pack's sources",
                    f.id
                ));
            }
        }
        for fl in &f.flows {
            if !own.contains(fl) {
                out.push(format!(
                    "fudge {} lives in {fl}, which is not a flow or box of this pack",
                    f.id
                ));
            }
        }
    }
    let mut names = BTreeSet::new();
    for c in checks {
        if !names.insert(c.name.as_str()) {
            out.push(format!("reproduction check {:?} appears twice", c.name));
        }
        if !ids.contains(c.source.as_str()) {
            out.push(format!(
                "reproduction check {:?} cites {}, which is not among the pack's sources",
                c.name, c.source
            ));
        }
        if c.expect.is_empty() {
            out.push(format!("reproduction check {:?} expects nothing", c.name));
        }
        for (k, e) in &c.expect {
            if !(e.tolerance.is_finite() && e.tolerance >= 0.0) || !e.value.is_finite() {
                out.push(format!(
                    "reproduction check {:?}: {k} needs a finite value and a tolerance of zero or more",
                    c.name
                ));
            }
        }
    }
    out
}
