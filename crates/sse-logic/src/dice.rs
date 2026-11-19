//! Chance stays deterministic (record design 3.4 and 8A.3).
//!
//! A `roll` is a keyed draw:
//! `draw(table_secret, subject address, "<flow id>.<node id>.v1", counter)`,
//! where the counter is the tick or turn index, never a running generator.
//! The same campaign, orders and dice give the same rolls in every replay,
//! in whatever order the blocks are evaluated.
//!
//! Dice never use the universe seed: ordinary seats hold it, so a roll keyed
//! on it could be worked out in advance. Rolls are keyed on the **table
//! secret**, held only on the GM's machine. Seats receive a commitment (a
//! hash of the secret); when the GM reveals the secret, anyone can check
//! the commitment and replay the dice.

use serde::Deserialize;
use serde_json::Value as J;
use sse_core::draw::{Draws, Stream};
use sse_core::hash::Hasher;
use sse_core::{Address, ContentHash, Dimension, Seed};
use sse_flow::{
    Atom, AtomDecl, AtomError, BlockClass, Call, CostClass, Out, Param, Port, Shape, Val,
};

/// Most dice in one roll; each die takes its own draw index.
pub const MAX_DICE: u64 = 100;
const PER_COUNTER: u64 = 128;

/// A table secret from the GM machine's own entropy (the UI passes bytes
/// from the platform's secure random source). Never sent to a seat.
pub fn table_secret(entropy: &[u8]) -> Seed {
    let mut h = Hasher::new("sse.table-secret.v1");
    h.update(entropy);
    Seed(h.finish())
}

/// The commitment every seat receives when a campaign first uses dice.
pub fn commitment(secret: &Seed) -> ContentHash {
    let mut h = Hasher::new("sse.table-secret.commit.v1");
    h.update(&secret.0 .0);
    h.finish()
}

/// After a reveal: does this secret match the commitment seats were sent?
pub fn verify(secret: &Seed, commitment_sent: &ContentHash) -> bool {
    commitment(secret) == *commitment_sent
}

/// The stream a roll node draws on: `<flow id>.<node id>.v1`, each part
/// lower-cased with anything outside `a-z 0-9 _` written as `_`.
pub fn stream_for(flow_id: &str, node_id: &str) -> Stream {
    let clean = |s: &str| -> String {
        let t: String = s
            .chars()
            .map(|c| {
                let c = c.to_ascii_lowercase();
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if t.is_empty() {
            "_".into()
        } else {
            t
        }
    };
    let parts: Vec<String> = flow_id
        .split('.')
        .chain(std::iter::once(node_id))
        .map(clean)
        .collect();
    Stream::new(&format!("{}.v1", parts.join("."))).expect("cleaned stream names are valid")
}

/// One die: a value in 1..=sides, for die `k` of the roll at `counter`.
pub fn die(
    secret: &Seed,
    subject: &Address,
    stream: &Stream,
    counter: u64,
    k: u64,
    sides: u64,
) -> u64 {
    let d = Draws::new(*secret, subject.clone(), stream.clone());
    1 + d.below(counter.wrapping_mul(PER_COUNTER).wrapping_add(k), sides)
}

fn subject_of(cx: &Call) -> Result<Address, AtomError> {
    cx.env
        .subject
        .clone()
        .ok_or_else(|| AtomError("a roll needs a subject (the thing it is rolled for)".into()))
}
fn secret_of(cx: &Call) -> Result<Seed, AtomError> {
    cx.env.table_secret.ok_or_else(|| {
        AtomError(
            "dice are rolled on the GM's machine only (no table secret here); ask the GM".into(),
        )
    })
}

fn decl(
    id: &str,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    params: Vec<Param>,
    c: &str,
    s: &str,
    f: &str,
) -> AtomDecl {
    AtomDecl {
        id: id.into(),
        version: "1.0.0".into(),
        inputs,
        outputs,
        params,
        cost_class: CostClass::Trivial,
        references: vec![],
        explain: sse_flow::decl::ExplainText {
            curious: c.into(),
            science: s.into(),
            formula: f.into(),
        },
        class: Some(BlockClass::Logic),
        dims: None,
    }
}
fn port(name: &str, shape: Shape) -> Port {
    Port {
        name: name.into(),
        kind: shape.to_string(),
        dim: Dimension::NONE,
        valid: None,
        shape: (shape != Shape::Number).then_some(shape),
        any_dim: false,
    }
}

/// `chance.roll`: N dice of S sides.
pub struct Roll(AtomDecl);
/// `chance.draw`: one entry from a weighted table in the node's config.
pub struct TableDraw(AtomDecl);

pub fn all() -> Vec<Box<dyn Atom>> {
    vec![
        Box::new(Roll(decl(
            "chance.roll",
            vec![],
            vec![
                port("total", Shape::Number),
                port("dice", Shape::List(Box::new(Shape::Number))),
            ],
            vec![
                Param {
                    name: "dice".into(),
                    dim: Dimension::NONE,
                    default: 1.0,
                    range: Some((1.0, MAX_DICE as f64)),
                    description: "How many dice.".into(),
                },
                Param {
                    name: "sides".into(),
                    dim: Dimension::NONE,
                    default: 6.0,
                    range: Some((2.0, 1000.0)),
                    description: "Sides on each die.".into(),
                },
            ],
            "Rolls dice for the GM.",
            "A keyed draw on the table secret, the subject, this node and the turn: the same turn gives the same dice in every replay, and no player device can work them out.",
            "sum of N dice, each 1..S",
        ))),
        Box::new(TableDraw(decl(
            "chance.draw",
            vec![],
            vec![port("value", Shape::Choice)],
            vec![],
            "Draws one entry from a table, more likely the heavier its weight.",
            "A keyed draw on the table secret, as for dice; weights are whole numbers.",
            "P(entry) = weight / total",
        ))),
    ]
}

fn whole(x: f64, what: &str) -> Result<u64, AtomError> {
    if x.fract() != 0.0 || x < 1.0 {
        return Err(AtomError(format!("{what} must be a whole number from 1")));
    }
    Ok(x as u64)
}

impl Atom for Roll {
    fn decl(&self) -> &AtomDecl {
        &self.0
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(
            "chance.roll is evaluated through its typed form".into(),
        ))
    }
    fn eval_typed(&self, _: &[Val], params: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let (n, sides) = (whole(params[0], "dice")?, whole(params[1], "sides")?);
        let (secret, subject) = (secret_of(cx)?, subject_of(cx)?);
        let stream = stream_for(cx.flow_id, cx.node_id);
        let dice: Vec<u64> = (0..n)
            .map(|k| die(&secret, &subject, &stream, cx.env.counter, k, sides))
            .collect();
        Ok(Out::values(vec![
            Val::Number(dice.iter().sum::<u64>() as f64),
            Val::List(dice.into_iter().map(|d| Val::Number(d as f64)).collect()),
        ]))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableCfg {
    entries: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    value: String,
    weight: u64,
}
fn table(config: Option<&J>) -> Result<TableCfg, String> {
    let j = config.ok_or("chance.draw needs `entries` in its config")?;
    let t: TableCfg = serde_json::from_value(j.clone()).map_err(|e| format!("chance.draw: {e}"))?;
    if t.entries.is_empty() || t.entries.iter().any(|e| e.weight == 0) {
        return Err("chance.draw: entries must be non-empty with weights of 1 or more".into());
    }
    if t.entries
        .iter()
        .map(|e| e.weight)
        .try_fold(0u64, u64::checked_add)
        .is_none()
    {
        return Err("chance.draw: weights are too large".into());
    }
    Ok(t)
}

impl Atom for TableDraw {
    fn decl(&self) -> &AtomDecl {
        &self.0
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(
            "chance.draw is evaluated through its typed form".into(),
        ))
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        table(config).map(|_| ())
    }
    fn eval_typed(&self, _: &[Val], _: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let t = table(cx.config).map_err(AtomError)?;
        let (secret, subject) = (secret_of(cx)?, subject_of(cx)?);
        let stream = stream_for(cx.flow_id, cx.node_id);
        let total: u64 = t.entries.iter().map(|e| e.weight).sum();
        let mut x = die(&secret, &subject, &stream, cx.env.counter, 0, total) - 1;
        for e in &t.entries {
            if x < e.weight {
                return Ok(Out::values(vec![Val::Choice(e.value.clone())]));
            }
            x -= e.weight;
        }
        unreachable!("x is below the total weight")
    }
}
