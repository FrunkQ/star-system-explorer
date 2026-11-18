//! Envelopes (H6.1): what a parent knows about children it has not looked at.

use crate::dist::Distribution;
use serde::{Deserialize, Serialize};
use sse_chronicle::{Evidence, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, Dimension};
use std::collections::BTreeMap;
use std::fmt;

/// A conserved total, in integer minor units so closure is exact.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Budget {
    /// Amount in minor units.
    pub amount: u64,
    /// SI value of one minor unit (e.g. 1e20 kg).
    pub quantum: f64,
    pub dim: Dimension,
    /// Budgets in the same group are split by the same share vector (metals
    /// follow gas mass); a different group gets independent shares.
    #[serde(default = "default_group")]
    pub group: String,
}
fn default_group() -> String {
    "mass".into()
}

/// How many generated children there are.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CountSpec {
    Fixed {
        n: u64,
    },
    Poisson {
        mean: f64,
    },
    Uniform {
        lo: u64,
        hi: u64,
    },
    /// Draw children from `dist` until the next one would not fit in
    /// `budget` (in its minor units: `unit` per unit of `dist`). Each child's
    /// share of that budget is its own draw, so the drawn distribution comes
    /// through exactly (an initial mass function: no star is rescaled because
    /// a rare big one is missing). What is left is residue, under one draw.
    /// The partition is not used; other budgets follow these shares.
    Fill {
        budget: String,
        dist: Distribution,
        unit: f64,
    },
}

/// The weights a budget is split by.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Weights {
    /// Every child the same (remainders by index).
    Equal,
    /// Flat Dirichlet(1): weights `-ln u`.
    Exponential,
    /// Weights drawn from a distribution (a mass function, say).
    Drawn { dist: Distribution },
    /// Weights given by index, for children whose shares a model computes
    /// (birth cohorts by star formation history). Needs a fixed count of the
    /// same length.
    Given { weights: Vec<f64> },
}

/// Flat splits compute every child's weight (exact Hamilton apportionment);
/// tree splits halve the budget recursively, so one child costs log2(n)
/// draws and very large counts stay cheap.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "kebab-case")]
pub enum PartitionSpec {
    Flat {
        weights: Weights,
    },
    /// Each split takes its fair fraction times `1 + jitter * (2u - 1)`.
    Tree {
        jitter: f64,
    },
}
impl Default for PartitionSpec {
    fn default() -> Self {
        PartitionSpec::Flat {
            weights: Weights::Exponential,
        }
    }
}

/// How one fact of a generated child is set.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum FieldSpec {
    /// Drawn from a distribution with the child's own keyed draw.
    Draw {
        dist: Distribution,
        /// Algorithm version, part of the stream name (`resolve.field.<name>.v<version>`).
        #[serde(default = "one")]
        version: u32,
    },
    /// Copied from the parent's own facts.
    Inherit { field: String },
}
fn one() -> u32 {
    1
}

/// A child's whole life line, drawn once (H6.1 rule 6). Times are seconds
/// from the universe origin; any epoch is a read of this line.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeSpec {
    pub born: Distribution,
    pub lifetime: Distribution,
}

/// An authored or catalogued child that must appear. Pins claim first.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pin {
    /// Must be a direct child of the envelope's address.
    pub address: Address,
    #[serde(default)]
    pub claims: BTreeMap<String, u64>,
    #[serde(default)]
    pub facts: BTreeMap<String, Value>,
    pub evidence: Evidence,
}

/// The template for a generated child's own envelope. Its budgets are the
/// child's claimed shares, so conservation holds all the way down.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChildTemplate {
    pub tier: String,
    pub child_kind: String,
    pub count: CountSpec,
    #[serde(default)]
    pub partition: PartitionSpec,
    /// Which of the child's claimed budgets it passes down, by id.
    #[serde(default)]
    pub budgets: Vec<String>,
    #[serde(default)]
    pub fields: BTreeMap<String, FieldSpec>,
    #[serde(default)]
    pub life: Option<LifeSpec>,
    #[serde(default)]
    pub child: Option<Box<ChildTemplate>>,
}

/// H6.1 `Envelope`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Envelope {
    pub address: Address,
    /// Tier name: a suite convention, not a kernel enum (H5.1).
    pub tier: String,
    pub epoch: UniverseTime,
    /// Address segment kind of generated children (`sys`, `body`, ...).
    pub child_kind: String,
    #[serde(default)]
    pub budgets: BTreeMap<String, Budget>,
    pub count: CountSpec,
    #[serde(default)]
    pub partition: PartitionSpec,
    #[serde(default)]
    pub fields: BTreeMap<String, FieldSpec>,
    #[serde(default)]
    pub life: Option<LifeSpec>,
    /// What died here before; carried for suites and explanations.
    #[serde(default)]
    pub legacies: BTreeMap<String, Value>,
    #[serde(default)]
    pub pins: Vec<Pin>,
    /// Pins count toward the drawn count (catalogue fill: only what the
    /// catalogue misses is generated). Default true.
    #[serde(default = "yes")]
    pub pins_count_toward: bool,
    /// Declared spread, for honest labelling.
    #[serde(default)]
    pub uncertainty: BTreeMap<String, f64>,
    /// The parent's own facts, readable by `Inherit` fields.
    #[serde(default)]
    pub facts: BTreeMap<String, Value>,
    #[serde(default)]
    pub child: Option<Box<ChildTemplate>>,
}
fn yes() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnvelopeError {
    BadName(String),
    BadDistribution { field: String, reason: String },
    BadPin { pin: String, reason: String },
    UnknownBudget { pin: String, budget: String },
    BadCount(String),
    BadPartition(String),
    BadTemplate(String),
    IndexOutOfRange { index: u64, generated: u64 },
}
impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvelopeError::BadName(n) => write!(f, "{n:?} must be lowercase letters, digits and _"),
            EnvelopeError::BadDistribution { field, reason } => {
                write!(f, "field {field}: {reason}")
            }
            EnvelopeError::BadPin { pin, reason } => write!(f, "pin {pin}: {reason}"),
            EnvelopeError::UnknownBudget { pin, budget } => {
                write!(f, "pin {pin} claims from unknown budget {budget}")
            }
            EnvelopeError::BadCount(r) => write!(f, "count: {r}"),
            EnvelopeError::BadPartition(r) => write!(f, "partition: {r}"),
            EnvelopeError::BadTemplate(r) => write!(f, "child template: {r}"),
            EnvelopeError::IndexOutOfRange { index, generated } => {
                write!(
                    f,
                    "child {index} does not exist: this cloud has {generated} generated children"
                )
            }
        }
    }
}
impl std::error::Error for EnvelopeError {}

pub(crate) fn check_name(n: &str) -> Result<(), EnvelopeError> {
    let ok = !n.is_empty()
        && n.as_bytes()[0].is_ascii_lowercase()
        && n.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_');
    if ok {
        Ok(())
    } else {
        Err(EnvelopeError::BadName(n.into()))
    }
}

fn check_fields(fields: &BTreeMap<String, FieldSpec>) -> Result<(), EnvelopeError> {
    for (name, f) in fields {
        check_name(name)?;
        if let FieldSpec::Draw { dist, .. } = f {
            dist.validate()
                .map_err(|e| EnvelopeError::BadDistribution {
                    field: name.clone(),
                    reason: e.0,
                })?;
        }
    }
    Ok(())
}

fn check_count(c: &CountSpec) -> Result<(), EnvelopeError> {
    match c {
        CountSpec::Poisson { mean } if !mean.is_finite() || *mean < 0.0 => {
            Err(EnvelopeError::BadCount("poisson mean must be >= 0".into()))
        }
        CountSpec::Uniform { lo, hi } if lo > hi => Err(EnvelopeError::BadCount("lo > hi".into())),
        CountSpec::Fill { dist, unit, .. } => {
            dist.validate().map_err(|e| EnvelopeError::BadCount(e.0))?;
            let positive = match dist {
                Distribution::PowerLaw { .. }
                | Distribution::BrokenPowerLaw { .. }
                | Distribution::LogNormal { .. } => true,
                Distribution::Uniform { lo, .. } => *lo > 0.0,
                Distribution::Fixed { value } => *value > 0.0,
                _ => false,
            };
            if !positive || !(unit.is_finite() && *unit > 0.0) {
                return Err(EnvelopeError::BadCount(
                    "fill needs a positive numeric distribution and unit > 0".into(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn check_partition(p: &PartitionSpec) -> Result<(), EnvelopeError> {
    match p {
        PartitionSpec::Tree { jitter } if !(0.0..=1.0).contains(jitter) => Err(
            EnvelopeError::BadPartition("tree jitter must be in [0, 1]".into()),
        ),
        PartitionSpec::Flat {
            weights: Weights::Drawn { dist },
        } => {
            dist.validate()
                .map_err(|e| EnvelopeError::BadPartition(e.0))?;
            if matches!(dist, Distribution::Choice { .. }) {
                return Err(EnvelopeError::BadPartition(
                    "weights must be numeric".into(),
                ));
            }
            Ok(())
        }
        PartitionSpec::Flat {
            weights: Weights::Given { weights },
        } if weights.iter().any(|w| !w.is_finite() || *w < 0.0) => Err(
            EnvelopeError::BadPartition("given weights must be finite and >= 0".into()),
        ),
        _ => Ok(()),
    }
}

fn check_template(t: &ChildTemplate) -> Result<(), EnvelopeError> {
    check_name(&t.child_kind)?;
    check_count(&t.count)?;
    check_partition(&t.partition)?;
    check_fields(&t.fields)?;
    if let Some(l) = &t.life {
        l.born
            .validate()
            .map_err(|e| EnvelopeError::BadTemplate(e.0))?;
        l.lifetime
            .validate()
            .map_err(|e| EnvelopeError::BadTemplate(e.0))?;
    }
    if let Some(c) = &t.child {
        for b in &c.budgets {
            if !t.budgets.contains(b) {
                return Err(EnvelopeError::BadTemplate(format!(
                    "grandchild passes down budget {b} that its parent does not hold"
                )));
            }
        }
        check_template(c)?;
    }
    Ok(())
}

impl Envelope {
    pub fn validate(&self) -> Result<(), EnvelopeError> {
        check_name(&self.child_kind)?;
        for (id, b) in &self.budgets {
            check_name(id)?;
            check_name(&b.group)?;
            if !(b.quantum.is_finite() && b.quantum > 0.0) {
                return Err(EnvelopeError::BadName(format!(
                    "budget {id} quantum must be > 0"
                )));
            }
        }
        check_count(&self.count)?;
        if let CountSpec::Fill { budget, .. } = &self.count {
            if !self.budgets.contains_key(budget) {
                return Err(EnvelopeError::BadCount(format!(
                    "fill names unknown budget {budget}"
                )));
            }
        }
        check_partition(&self.partition)?;
        if let PartitionSpec::Flat {
            weights: Weights::Given { weights },
        } = &self.partition
        {
            if self.count
                != (CountSpec::Fixed {
                    n: weights.len() as u64,
                })
            {
                return Err(EnvelopeError::BadPartition(
                    "given weights need a fixed count of the same length".into(),
                ));
            }
        }
        check_fields(&self.fields)?;
        if let Some(l) = &self.life {
            for (n, d) in [("life.born", &l.born), ("life.lifetime", &l.lifetime)] {
                d.validate().map_err(|e| EnvelopeError::BadDistribution {
                    field: n.into(),
                    reason: e.0,
                })?;
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for p in &self.pins {
            let name = p.address.to_string();
            if p.address.parent().as_ref() != Some(&self.address) {
                return Err(EnvelopeError::BadPin {
                    pin: name,
                    reason: format!("must be a direct child of {}", self.address),
                });
            }
            let key = p.address.last().key();
            if key.bytes().all(|c| c.is_ascii_digit()) {
                return Err(EnvelopeError::BadPin {
                    pin: name,
                    reason: "a pin may not take a generated index; use the authored (@n) or a catalogue key".into(),
                });
            }
            if !seen.insert(p.address.clone()) {
                return Err(EnvelopeError::BadPin {
                    pin: name,
                    reason: "pinned twice".into(),
                });
            }
            for b in p.claims.keys() {
                if !self.budgets.contains_key(b) {
                    return Err(EnvelopeError::UnknownBudget {
                        pin: name,
                        budget: b.clone(),
                    });
                }
            }
        }
        if let Some(t) = &self.child {
            for b in &t.budgets {
                if !self.budgets.contains_key(b) {
                    return Err(EnvelopeError::BadTemplate(format!(
                        "child passes down budget {b} that this envelope does not hold"
                    )));
                }
            }
            check_template(t)?;
        }
        Ok(())
    }
}
