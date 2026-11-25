//! Rolled profiles (WBH part 2): a pack's dice procedure for what the
//! physics cannot decide (a world's government, law, tech level, port and
//! bases), as data. A program names a code sheet and a list of steps; each
//! step reads the sheet's current symbols (physics, authored, or rolled by an
//! earlier step), rolls d6s drawn from the universe's seed (the same on every
//! device), adds its modifiers, and gives a symbol. Nothing here writes: the
//! kernel turns the results into authored facts, marked as rolled, that the
//! GM can overrule. A segment that already has an authored code keeps it.

use crate::{ehex, ehex_value, Reading, Why, UNKNOWN};
use serde::{Deserialize, Serialize};
use sse_core::address::Address;
use sse_core::draw::{draw, Seed, Stream};
use std::collections::BTreeMap;

/// A condition on the current symbols: `segment`'s symbol is one of `in`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cond {
    pub segment: String,
    #[serde(rename = "in")]
    pub one_of: Vec<String>,
}

/// Raise the total to at least `min` when every condition holds.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Floor {
    pub when: Vec<Cond>,
    pub min: i64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Step {
    /// The segment it fills (or a name of its own for a yes-or-no roll).
    pub id: String,
    #[serde(default)]
    pub label: String,
    /// The fact it writes. Left out: the sheet segment's `authored` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writes: Option<String>,
    /// How many six-sided dice.
    pub dice: u32,
    #[serde(default)]
    pub plus: i64,
    /// Segments whose values are added (Cepheus: government adds population).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add: Vec<String>,
    /// Modifiers by another segment's symbol: segment, then symbol, then DM.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dm: BTreeMap<String, BTreeMap<String, i64>>,
    /// The result is 0 without a roll when any of these holds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zero_when: Vec<Cond>,
    /// The step is skipped unless every one of these holds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only_when: Vec<Cond>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub floors: Vec<Floor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    /// Totals to symbols: the last row whose number the total reaches (the
    /// first row below that). Left out: the total as an eHex digit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub table: Vec<(i64, String)>,
    /// A yes-or-no roll: true when the total reaches it (a base).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<i64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Program {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub about: String,
    /// The code sheet whose symbols the steps read.
    pub sheet: String,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub source: String,
    /// A fact whose presence means the world came with its whole profile (an
    /// imported list's `provenance.code`): then every step keeps what the
    /// world has, bases included, unless rolled again on purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
}

/// One step's result.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rolled {
    pub id: String,
    pub label: String,
    /// The fact to write; none when the step was skipped or kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writes: Option<String>,
    /// The symbol, or "yes" / "no" for a yes-or-no roll.
    pub symbol: String,
    pub dice: Vec<u32>,
    pub total: i64,
    /// "rolled", "zero" (no roll: a condition gives 0), "kept" (an authored
    /// code stays), "skipped" (its conditions do not hold).
    pub how: String,
    /// In words: "2D6 rolled 3 + 5, -7, + population 8 = 9".
    pub why: String,
}

fn symbol_of<'a>(now: &'a BTreeMap<String, String>, seg: &str) -> &'a str {
    now.get(seg).map(String::as_str).unwrap_or(UNKNOWN)
}

fn holds(now: &BTreeMap<String, String>, c: &Cond) -> bool {
    c.one_of.iter().any(|s| s == symbol_of(now, &c.segment))
}

fn value_of(now: &BTreeMap<String, String>, seg: &str) -> Result<i64, String> {
    let s = symbol_of(now, seg);
    s.chars()
        .next()
        .filter(|_| s.chars().count() == 1)
        .and_then(ehex_value)
        .ok_or_else(|| format!("needs a number for {seg}, which reads {s:?}"))
}

/// The stream a program draws from (its own, so adding a program elsewhere
/// never moves another's dice).
fn stream_of(program: &str) -> Stream {
    let id: String = program
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '.'
            }
        })
        .collect();
    let parts: Vec<&str> = id.split('.').filter(|p| !p.is_empty()).collect();
    Stream::new(&format!("roll.{}.v1", parts.join(".")))
        .unwrap_or_else(|_| Stream::new("roll.program.v1").expect("a fixed stream"))
}

/// Run `program` over one world's `reading` (through `program.sheet`).
/// `authored` maps each segment id to the fact holding its authored code.
/// `decided` is the world's `program.decided_by` fact, in words, when it has
/// one. A code already decided (set by hand, or the whole profile came with
/// the world) is kept; with `reroll`, every code is rolled again.
pub fn roll(
    program: &Program,
    reading: &Reading,
    authored: &BTreeMap<String, String>,
    decided: Option<&str>,
    seed: &Seed,
    address: &Address,
    reroll: bool,
) -> Result<Vec<Rolled>, String> {
    let mut now: BTreeMap<String, String> = reading
        .segments
        .iter()
        .map(|s| (s.id.clone(), s.symbol.clone()))
        .collect();
    let stream = stream_of(&program.id);
    let mut out = Vec::new();
    for (i, st) in program.steps.iter().enumerate() {
        let label = if st.label.is_empty() {
            st.id.clone()
        } else {
            st.label.clone()
        };
        let writes = st.writes.clone().or_else(|| authored.get(&st.id).cloned());
        let seg = reading.segment(&st.id);
        let is_authored = seg.is_some_and(|s| matches!(s.why, Why::Authored { .. }));
        let mut row = Rolled {
            id: st.id.clone(),
            label,
            writes: None,
            symbol: symbol_of(&now, &st.id).to_string(),
            dice: vec![],
            total: 0,
            how: String::new(),
            why: String::new(),
        };
        if !reroll && (is_authored || decided.is_some()) {
            row.how = "kept".into();
            row.why = match decided.filter(|_| !is_authored) {
                Some(d) => format!("kept: the world came with its profile ({d})"),
                None => format!("kept: set by hand as {}", row.symbol),
            };
            out.push(row);
            continue;
        }
        if !st.only_when.iter().all(|c| holds(&now, c)) {
            row.how = "skipped".into();
            row.symbol = if st.target.is_some() {
                "no".into()
            } else {
                row.symbol
            };
            row.why = "skipped: its conditions do not hold".into();
            out.push(row);
            continue;
        }
        let Some(field) = writes else {
            return Err(format!(
                "step {}: no fact to write (give `writes`, or roll a segment with an authored field)",
                st.id
            ));
        };
        row.writes = Some(field);
        let mut words: Vec<String> = Vec::new();
        let total = if st.zero_when.iter().any(|c| holds(&now, c)) {
            row.how = "zero".into();
            let c = st
                .zero_when
                .iter()
                .find(|c| holds(&now, c))
                .expect("one holds");
            words.push(format!(
                "0 without a roll: {} is {}",
                c.segment,
                symbol_of(&now, &c.segment)
            ));
            0
        } else {
            row.how = "rolled".into();
            for d in 0..st.dice {
                let n = draw(seed, address, &stream, (i as u64) * 16 + u64::from(d));
                row.dice.push((n % 6) as u32 + 1);
            }
            let dice: i64 = row.dice.iter().map(|d| i64::from(*d)).sum();
            words.push(format!(
                "{}D6 rolled {}",
                st.dice,
                row.dice
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(" + ")
            ));
            let mut t = dice + st.plus;
            if st.plus != 0 {
                words.push(format!("{:+}", st.plus));
            }
            for a in &st.add {
                let v = value_of(&now, a).map_err(|e| format!("step {}: {e}", st.id))?;
                t += v;
                words.push(format!("+ {a} {v}"));
            }
            for (on, by) in &st.dm {
                let s = symbol_of(&now, on);
                if let Some(dm) = by.get(s) {
                    t += dm;
                    words.push(format!("{dm:+} for {on} {s}"));
                }
            }
            t
        };
        let mut t = total;
        for f in st.floors.iter().filter(|_| row.how == "rolled") {
            if f.when.iter().all(|c| holds(&now, c)) && t < f.min {
                words.push(format!("raised to {} (a floor)", f.min));
                t = f.min;
            }
        }
        if let Some(m) = st.min {
            t = t.max(m);
        }
        if let Some(m) = st.max {
            t = t.min(m);
        }
        row.total = t;
        row.symbol = if let Some(target) = st.target {
            if t >= target { "yes" } else { "no" }.into()
        } else if !st.table.is_empty() {
            st.table
                .iter()
                .rev()
                .find(|(n, _)| t >= *n)
                .or(st.table.first())
                .map(|(_, s)| s.clone())
                .expect("a row")
        } else {
            ehex(t.max(0))
                .map(String::from)
                .unwrap_or_else(|| UNKNOWN.into())
        };
        row.why = format!(
            "{} = {}{}",
            words.join(", "),
            t,
            match st.target {
                Some(target) => format!(" (needs {target}+)"),
                None if st.table.is_empty() => String::new(),
                None => format!(" gives {}", row.symbol),
            }
        );
        now.insert(st.id.clone(), row.symbol.clone());
        out.push(row);
    }
    Ok(out)
}
