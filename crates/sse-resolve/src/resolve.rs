//! `resolve(envelope, child) -> (facts for the child, the child's own envelope)` (H6.1).

use crate::dist::Sample;
use crate::envelope::{Budget, ChildTemplate, Envelope, EnvelopeError, FieldSpec, LifeSpec};
use crate::partition::{self, open_unit, stream, BudgetSplit, Work};
use serde::{Deserialize, Serialize};
use sse_chronicle::{
    kinds, Chronicle, ChronicleError, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::{Address, ContentHash, Draws, Seed};
use std::collections::BTreeMap;

/// Which child of an envelope.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildRef {
    /// A generated child by index (address key is the decimal index).
    Generated(u64),
    /// A pin by its position in `envelope.pins`.
    Pinned(usize),
}

/// The children an envelope holds: pins first (they claim first), then the
/// generated indices `0..generated`.
#[derive(Clone, Debug, PartialEq)]
pub struct Children {
    pub pins: Vec<Address>,
    pub generated: u64,
    pub splits: BTreeMap<String, BudgetSplit>,
}

impl Children {
    /// True when pins claim more than a budget holds (kept and marked).
    pub fn over_budget(&self) -> Vec<String> {
        self.splits
            .iter()
            .filter(|(_, s)| s.deficit > 0)
            .map(|(k, _)| k.clone())
            .collect()
    }
}

/// A resolved child.
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    pub address: Address,
    pub child: ChildRef,
    /// Facts fixed by this resolve: `budget.<id>` (minor units), drawn and
    /// inherited fields, `life.born_s` / `life.ends_s`, and bookkeeping
    /// (`resolve.tier`, `resolve.evidence`, `resolve.fill`).
    pub facts: BTreeMap<String, Value>,
    /// The child's own cloud, if the template goes deeper.
    pub envelope: Option<Envelope>,
    /// Deterministic work units spent.
    pub work: u64,
}

pub fn children(seed: &Seed, env: &Envelope) -> Result<Children, EnvelopeError> {
    env.validate()?;
    let mut w = Work::default();
    let generated = partition::generated_count(seed, env, &mut w);
    let mut splits = partition::budget_splits(env, generated);
    if let (crate::envelope::CountSpec::Fill { budget, .. }, Some(f)) =
        (&env.count, partition::fill_draws(seed, env, &mut w))
    {
        let s = splits.get_mut(budget).expect("validated");
        s.residue = s.remainder - f.iter().sum::<u64>();
    }
    Ok(Children {
        pins: env.pins.iter().map(|p| p.address.clone()).collect(),
        generated,
        splits,
    })
}

fn sample_value(s: Sample) -> Value {
    match s {
        Sample::Int(k) => Value::Int(k as i128),
        Sample::Num(x) => Value::Float(x),
        Sample::Cat(c) => Value::Text(c),
    }
}

/// Draw one field for the child at `addr`. Keyed on the child's own address,
/// so a child resolves without its siblings.
fn draw_field(
    seed: &Seed,
    addr: &Address,
    name: &str,
    spec: &FieldSpec,
    parent: &BTreeMap<String, Value>,
    work: &mut Work,
) -> Option<Value> {
    match spec {
        FieldSpec::Draw { dist, version } => {
            let d = Draws::new(
                *seed,
                addr.clone(),
                stream(&format!("resolve.field.{name}.v{version}")),
            );
            work.add(1);
            Some(sample_value(dist.quantile(open_unit(&d, 0))))
        }
        FieldSpec::Inherit { field } => parent.get(field).cloned(),
    }
}

fn num(s: Sample) -> f64 {
    match s {
        Sample::Num(x) => x,
        Sample::Int(k) => k as f64,
        Sample::Cat(_) => 0.0,
    }
}

fn draw_life(
    seed: &Seed,
    addr: &Address,
    life: &LifeSpec,
    facts: &mut BTreeMap<String, Value>,
    work: &mut Work,
) {
    let d = Draws::new(*seed, addr.clone(), stream("resolve.life.v1"));
    work.add(2);
    let born = m::round(num(life.born.quantile(open_unit(&d, 0))));
    let span = m::round(num(life.lifetime.quantile(open_unit(&d, 1))).max(0.0));
    let clamp = |x: f64| x.clamp(i64::MIN as f64, i64::MAX as f64) as i64;
    let b = clamp(born);
    facts.insert("life.born_s".into(), Value::Int(b as i128));
    facts.insert(
        "life.ends_s".into(),
        Value::Int(b.saturating_add(clamp(span)) as i128),
    );
}

fn child_envelope(
    t: &ChildTemplate,
    addr: &Address,
    parent: &Envelope,
    shares: &BTreeMap<String, u64>,
    facts: &BTreeMap<String, Value>,
) -> Envelope {
    let budgets = t
        .budgets
        .iter()
        .map(|id| {
            let b = &parent.budgets[id];
            (
                id.clone(),
                Budget {
                    amount: shares[id],
                    quantum: b.quantum,
                    dim: b.dim,
                    group: b.group.clone(),
                },
            )
        })
        .collect();
    Envelope {
        address: addr.clone(),
        tier: t.tier.clone(),
        epoch: parent.epoch,
        child_kind: t.child_kind.clone(),
        budgets,
        count: t.count.clone(),
        partition: t.partition.clone(),
        fields: t.fields.clone(),
        life: t.life.clone(),
        legacies: parent.legacies.clone(),
        pins: Vec::new(),
        pins_count_toward: true,
        uncertainty: parent.uncertainty.clone(),
        facts: facts.clone(),
        child: t.child.clone(),
    }
}

/// Resolve one child. Pure: uses only the envelope, the child's address,
/// keyed draws and pins (H6.1 rule 1).
pub fn resolve(seed: &Seed, env: &Envelope, child: &ChildRef) -> Result<Resolved, EnvelopeError> {
    let kids = children(seed, env)?;
    let mut work = Work::default();
    work.add(1);
    let p = match child {
        ChildRef::Generated(i) => {
            if *i >= kids.generated {
                return Err(EnvelopeError::IndexOutOfRange {
                    index: *i,
                    generated: kids.generated,
                });
            }
            let shares =
                partition::shares_for(seed, env, &kids.splits, kids.generated, *i, &mut work);
            let mut r = resolve_with_shares(seed, env, *i, shares)?;
            r.work += work.0;
            return Ok(r);
        }
        ChildRef::Pinned(p) => *p,
    };
    let pin = env.pins.get(p).ok_or(EnvelopeError::IndexOutOfRange {
        index: p as u64,
        generated: kids.generated,
    })?;
    let shares: BTreeMap<String, u64> = env
        .budgets
        .keys()
        .map(|id| (id.clone(), *pin.claims.get(id).unwrap_or(&0)))
        .collect();
    let (addr, mut facts, evidence) = (pin.address.clone(), pin.facts.clone(), pin.evidence);
    for (id, v) in &shares {
        facts.insert(format!("budget.{id}"), Value::Int(*v as i128));
    }
    // Pins keep every measured fact; anything unmeasured is filled around them and listed.
    let mut fill = Vec::new();
    for (name, spec) in &env.fields {
        if facts.contains_key(name) {
            continue;
        }
        if let Some(v) = draw_field(seed, &addr, name, spec, &env.facts, &mut work) {
            facts.insert(name.clone(), v);
            fill.push(Value::Text(name.clone()));
        }
    }
    if let Some(life) = &env.life {
        if !facts.contains_key("life.born_s") {
            draw_life(seed, &addr, life, &mut facts, &mut work);
            fill.push(Value::Text("life".into()));
        }
    }
    let tier = env
        .child
        .as_ref()
        .map_or(env.child_kind.clone(), |t| t.tier.clone());
    facts.insert("resolve.tier".into(), Value::Text(tier));
    facts.insert(
        "resolve.evidence".into(),
        sse_core::cbor::to_value(&evidence).expect("evidence encodes"),
    );
    if !fill.is_empty() {
        facts.insert("resolve.fill".into(), Value::Array(fill));
    }
    let over = kids.over_budget();
    if !over.is_empty() {
        facts.insert(
            "resolve.over_budget".into(),
            Value::Array(over.into_iter().map(Value::Text).collect()),
        );
    }
    let envelope = env
        .child
        .as_ref()
        .map(|t| child_envelope(t, &addr, env, &shares, &facts));
    Ok(Resolved {
        address: addr,
        child: child.clone(),
        facts,
        envelope,
        work: work.0,
    })
}

/// Resolve every child (pins first). One weight pass per budget group, so
/// this is the cheap way to resolve a whole cloud.
pub fn resolve_all(seed: &Seed, env: &Envelope) -> Result<Vec<Resolved>, EnvelopeError> {
    let kids = children(seed, env)?;
    let mut out = Vec::new();
    for p in 0..kids.pins.len() {
        out.push(resolve(seed, env, &ChildRef::Pinned(p))?);
    }
    let mut w = Work::default();
    let shares = partition::all_shares(seed, env, &kids.splits, kids.generated, &mut w);
    for (i, s) in shares.into_iter().enumerate() {
        // Same result as `resolve(.., Generated(i))`, without redoing the split.
        let mut r = resolve_with_shares(seed, env, i as u64, s)?;
        r.work += if i == 0 { w.0 } else { 0 };
        out.push(r);
    }
    Ok(out)
}

fn resolve_with_shares(
    seed: &Seed,
    env: &Envelope,
    i: u64,
    shares: BTreeMap<String, u64>,
) -> Result<Resolved, EnvelopeError> {
    let addr = env
        .address
        .child(&env.child_kind, i)
        .map_err(|e| EnvelopeError::BadName(e.to_string()))?;
    let mut work = Work::default();
    work.add(1);
    let mut facts = BTreeMap::new();
    for (id, v) in &shares {
        facts.insert(format!("budget.{id}"), Value::Int(*v as i128));
    }
    for (name, spec) in &env.fields {
        if let Some(v) = draw_field(seed, &addr, name, spec, &env.facts, &mut work) {
            facts.insert(name.clone(), v);
        }
    }
    if let Some(life) = &env.life {
        draw_life(seed, &addr, life, &mut facts, &mut work);
    }
    let tier = env
        .child
        .as_ref()
        .map_or(env.child_kind.clone(), |t| t.tier.clone());
    facts.insert("resolve.tier".into(), Value::Text(tier));
    facts.insert(
        "resolve.evidence".into(),
        sse_core::cbor::to_value(&Evidence::Resolved).expect("evidence encodes"),
    );
    let envelope = env
        .child
        .as_ref()
        .map(|t| child_envelope(t, &addr, env, &shares, &facts));
    Ok(Resolved {
        address: addr,
        child: ChildRef::Generated(i),
        facts,
        envelope,
        work: work.0,
    })
}

/// Where a body is in its life line at time `t` (H6.1 rule 6: any epoch is a
/// read of the line drawn once).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "kebab-case")]
pub enum Phase {
    /// No life line declared: the body is timeless at this tier.
    Timeless,
    Unborn {
        in_s: i64,
    },
    Alive {
        age_s: i64,
    },
    Ended {
        since_s: i64,
    },
}

pub fn phase_at(facts: &BTreeMap<String, Value>, t: UniverseTime) -> Phase {
    let get = |k: &str| match facts.get(k) {
        Some(Value::Int(v)) => Some(*v as i64),
        _ => None,
    };
    match (get("life.born_s"), get("life.ends_s")) {
        (Some(b), Some(e)) => {
            if t.s < b {
                Phase::Unborn { in_s: b - t.s }
            } else if t.s < e {
                Phase::Alive { age_s: t.s - b }
            } else {
                Phase::Ended { since_s: t.s - e }
            }
        }
        _ => Phase::Timeless,
    }
}

#[derive(Debug)]
pub enum RecordError {
    Envelope(EnvelopeError),
    Chronicle(ChronicleError),
}
impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::Envelope(e) => write!(f, "{e}"),
            RecordError::Chronicle(e) => write!(f, "{e}"),
        }
    }
}
impl From<EnvelopeError> for RecordError {
    fn from(e: EnvelopeError) -> Self {
        RecordError::Envelope(e)
    }
}
impl From<ChronicleError> for RecordError {
    fn from(e: ChronicleError) -> Self {
        RecordError::Chronicle(e)
    }
}
impl From<sse_chronicle::EventError> for RecordError {
    fn from(e: sse_chronicle::EventError) -> Self {
        RecordError::Chronicle(e.into())
    }
}

/// The recorded resolve of `addr`, if any (the resolved store, H5.5).
pub fn recorded<'c>(chronicle: &'c Chronicle, addr: &Address) -> Option<&'c Event> {
    chronicle
        .resolved()
        .find(|e| e.body.subject.first() == Some(addr))
}

/// Outcome of resolving through the chronicle.
#[derive(Clone, Debug, PartialEq)]
pub struct Recorded {
    pub event: ContentHash,
    /// True if this call appended the event; false if it was already there.
    pub new: bool,
    pub facts: BTreeMap<String, Value>,
    pub envelope: Option<Envelope>,
}

/// Resolve a child and record it as a `resolve` event (H6.1 rule 5). If the
/// child was resolved before, the recorded facts win, so an old universe does
/// not shift when a later pack version changes (H5.5). Resolve events are
/// timeless (occurred and activated at the origin): they fix the whole life
/// line, and a time is a read of it.
pub fn resolve_recorded(
    chronicle: &mut Chronicle,
    seed: &Seed,
    env: &Envelope,
    child: &ChildRef,
    publisher: &Publisher,
) -> Result<Recorded, RecordError> {
    let r = resolve(seed, env, child)?;
    if let Some(e) = recorded(chronicle, &r.address) {
        let facts = e
            .body
            .payload
            .set
            .get(&r.address)
            .cloned()
            .unwrap_or_default();
        // Rebuild the child's cloud from what was recorded, not from today's draws.
        let shares: BTreeMap<String, u64> = env
            .budgets
            .keys()
            .map(|id| {
                let v = match facts.get(&format!("budget.{id}")) {
                    Some(Value::Int(v)) => *v as u64,
                    _ => 0,
                };
                (id.clone(), v)
            })
            .collect();
        let envelope = env
            .child
            .as_ref()
            .map(|t| child_envelope(t, &r.address, env, &shares, &facts));
        return Ok(Recorded {
            event: e.id,
            new: false,
            facts,
            envelope,
        });
    }
    let causes = recorded(chronicle, &env.address)
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let mut set = BTreeMap::new();
    set.insert(r.address.clone(), r.facts.clone());
    let ev = Event::new(EventBody {
        kind: kinds::RESOLVE.into(),
        subject: vec![r.address.clone()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: chronicle.next_stamp(),
        causes,
        payload: Payload {
            set,
            ..Default::default()
        },
        // The resolved store; a pin's own evidence is kept in `resolve.evidence`.
        evidence: Evidence::Resolved,
        publisher: publisher.clone(),
    })?;
    let id = chronicle.append(ev)?;
    Ok(Recorded {
        event: id,
        new: true,
        facts: r.facts,
        envelope: r.envelope,
    })
}
