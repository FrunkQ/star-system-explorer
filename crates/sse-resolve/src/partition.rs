//! Partitioning a parent's budgets among its children (H6.1 rules 2 and 3).
//!
//! The partition is drawn up front from keyed draws on the *parent's*
//! address, so resolving child 7 never needs children 0 to 6 beyond this
//! cheap split, and the order children are looked at changes nothing.
//! Amounts are integers: pins claim first, then the remainder is split with
//! exact closure (sum of children + pins + residue == budget, always).

use crate::dist::Sample;
use crate::envelope::{CountSpec, Envelope, PartitionSpec, Weights};
use sse_core::math as m;
use sse_core::{Draws, Seed, Stream};
use std::collections::BTreeMap;

pub(crate) fn stream(s: &str) -> Stream {
    Stream::new(s).expect("engine stream names are valid")
}

/// Open-interval unit draw in (0, 1), so quantile functions never see 0 or 1.
pub(crate) fn open_unit(d: &Draws, i: u64) -> f64 {
    ((d.u64(i) >> 11) as f64 + 0.5) * (1.0 / (1u64 << 53) as f64)
}

/// Work counter in deterministic units: one unit per keyed draw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work(pub u64);
impl Work {
    pub(crate) fn add(&mut self, n: u64) {
        self.0 = self.0.saturating_add(n);
    }
}

/// How the budgets of one envelope divide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetSplit {
    pub total: u64,
    /// Sum of pin claims (may exceed the total: see `deficit`).
    pub pinned: u64,
    /// What pins claimed beyond the total. Non-zero means the pins are
    /// physically impossible together; they are honoured and marked.
    pub deficit: u64,
    /// What is left for generated children.
    pub remainder: u64,
    /// Left unclaimed because there are no generated children.
    pub residue: u64,
}

/// Number of generated (not pinned) children.
pub fn generated_count(seed: &Seed, env: &Envelope, work: &mut Work) -> u64 {
    let d = Draws::new(*seed, env.address.clone(), stream("resolve.count.v1"));
    work.add(1);
    let n = match &env.count {
        CountSpec::Fixed { n } => *n,
        CountSpec::Poisson { mean } => {
            match (crate::dist::Distribution::Poisson { mean: *mean }).quantile(open_unit(&d, 0)) {
                Sample::Int(k) => k,
                _ => unreachable!(),
            }
        }
        CountSpec::Uniform { lo, hi } => lo + d.below(0, hi - lo + 1),
    };
    if env.pins_count_toward {
        n.saturating_sub(env.pins.len() as u64)
    } else {
        n
    }
}

/// Budget arithmetic for every budget of the envelope.
pub fn budget_splits(env: &Envelope, generated: u64) -> BTreeMap<String, BudgetSplit> {
    env.budgets
        .iter()
        .map(|(id, b)| {
            let pinned: u128 = env
                .pins
                .iter()
                .map(|p| *p.claims.get(id).unwrap_or(&0) as u128)
                .sum();
            let pinned = pinned.min(u64::MAX as u128) as u64;
            let deficit = pinned.saturating_sub(b.amount);
            let remainder = b.amount.saturating_sub(pinned);
            let residue = if generated == 0 { remainder } else { 0 };
            (
                id.clone(),
                BudgetSplit {
                    total: b.amount,
                    pinned,
                    deficit,
                    remainder,
                    residue,
                },
            )
        })
        .collect()
}

/// Integer weights for a flat split of `n` children in one budget group.
fn flat_weights(
    seed: &Seed,
    env: &Envelope,
    group: &str,
    weights: &Weights,
    n: u64,
    work: &mut Work,
) -> Vec<u64> {
    let raw: Vec<f64> = match weights {
        Weights::Equal => vec![1.0; n as usize],
        Weights::Exponential | Weights::Drawn { .. } => {
            let d = Draws::new(
                *seed,
                env.address.clone(),
                stream(&format!("resolve.split.{group}.v1")),
            );
            work.add(n);
            (0..n)
                .map(|j| {
                    let u = open_unit(&d, j);
                    match weights {
                        Weights::Exponential => -m::ln(u),
                        Weights::Drawn { dist } => match dist.quantile(u) {
                            Sample::Num(x) => x.max(0.0),
                            Sample::Int(k) => k as f64,
                            Sample::Cat(_) => 0.0,
                        },
                        Weights::Equal => unreachable!(),
                    }
                })
                .collect()
        }
    };
    let max = raw.iter().cloned().fold(0.0f64, f64::max);
    if max <= 0.0 {
        return vec![1; n as usize];
    }
    const SCALE: f64 = (1u64 << 40) as f64;
    raw.iter()
        .map(|w| {
            if *w <= 0.0 {
                0
            } else {
                (m::round(w / max * SCALE) as u64).max(1)
            }
        })
        .collect()
}

/// Exact largest-remainder apportionment of `amount` by integer weights.
fn apportion(amount: u64, w: &[u64]) -> Vec<u64> {
    let total: u128 = w.iter().map(|x| *x as u128).sum();
    if total == 0 {
        return vec![0; w.len()];
    }
    let a = amount as u128;
    let mut base: Vec<u64> = Vec::with_capacity(w.len());
    let mut rems: Vec<(u128, usize)> = Vec::with_capacity(w.len());
    for (i, wi) in w.iter().enumerate() {
        let q = a * *wi as u128;
        base.push((q / total) as u64);
        rems.push((q % total, i));
    }
    let given: u128 = base.iter().map(|x| *x as u128).sum();
    let left = (a - given) as usize;
    // Largest remainder first; ties go to the lower index.
    rems.sort_by(|x, y| y.0.cmp(&x.0).then(x.1.cmp(&y.1)));
    for (_, i) in rems.iter().take(left) {
        base[*i] += 1;
    }
    base
}

/// One child's share of one budget by tree halving. Costs log2(n) draws.
#[allow(clippy::too_many_arguments)] // all plain inputs of one pure function
fn tree_share(
    seed: &Seed,
    env: &Envelope,
    group: &str,
    jitter: f64,
    amount: u64,
    n: u64,
    index: u64,
    work: &mut Work,
) -> u64 {
    let d = Draws::new(
        *seed,
        env.address.clone(),
        stream(&format!("resolve.tree.{group}.v1")),
    );
    let (mut a, mut b, mut amt, mut node) = (0u64, n, amount, 1u64);
    while b - a > 1 {
        let mid = a + (b - a) / 2;
        let fair = (mid - a) as f64 / (b - a) as f64;
        let u = if jitter > 0.0 {
            work.add(1);
            d.unit(node)
        } else {
            0.5
        };
        let f = (fair * (1.0 + jitter * (2.0 * u - 1.0))).clamp(0.0, 1.0);
        let num = ((f * (1u64 << 32) as f64) as u64).min(1u64 << 32);
        let left = ((amt as u128 * num as u128) >> 32) as u64;
        if index < mid {
            amt = left;
            b = mid;
            node *= 2;
        } else {
            amt -= left;
            a = mid;
            node = node * 2 + 1;
        }
    }
    amt
}

/// Every budget's amount for generated child `index` (of `n`).
pub fn shares_for(
    seed: &Seed,
    env: &Envelope,
    splits: &BTreeMap<String, BudgetSplit>,
    n: u64,
    index: u64,
    work: &mut Work,
) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    let mut flat_cache: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for (id, b) in &env.budgets {
        let rem = splits[id].remainder;
        let share = match &env.partition {
            PartitionSpec::Flat { weights } => {
                let w = flat_cache
                    .entry(b.group.clone())
                    .or_insert_with(|| flat_weights(seed, env, &b.group, weights, n, work));
                apportion(rem, w)[index as usize]
            }
            PartitionSpec::Tree { jitter } => {
                tree_share(seed, env, &b.group, *jitter, rem, n, index, work)
            }
        };
        out.insert(id.clone(), share);
    }
    out
}

/// Every generated child's shares at once (one weight pass per group).
pub fn all_shares(
    seed: &Seed,
    env: &Envelope,
    splits: &BTreeMap<String, BudgetSplit>,
    n: u64,
    work: &mut Work,
) -> Vec<BTreeMap<String, u64>> {
    let mut per: Vec<BTreeMap<String, u64>> = vec![BTreeMap::new(); n as usize];
    for (id, b) in &env.budgets {
        let rem = splits[id].remainder;
        let col: Vec<u64> = match &env.partition {
            PartitionSpec::Flat { weights } => {
                apportion(rem, &flat_weights(seed, env, &b.group, weights, n, work))
            }
            PartitionSpec::Tree { jitter } => (0..n)
                .map(|i| tree_share(seed, env, &b.group, *jitter, rem, n, i, work))
                .collect(),
        };
        for (i, v) in col.into_iter().enumerate() {
            per[i].insert(id.clone(), v);
        }
    }
    per
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn apportion_closes_exactly(amount in 0u64..u64::MAX, w in proptest::collection::vec(0u64..(1u64 << 40), 1..200)) {
            let s = apportion(amount, &w);
            let total: u128 = s.iter().map(|x| *x as u128).sum();
            if w.iter().any(|x| *x > 0) {
                prop_assert_eq!(total, amount as u128);
            }
        }
    }
}
