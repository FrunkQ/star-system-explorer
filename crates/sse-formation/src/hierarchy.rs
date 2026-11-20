//! B1c part 2: multiples as nested pairs. Alex, 19 Nov: big multiples are
//! built as close binaries (one object) forming parts of wider binaries,
//! which is how real stable multiples are arranged (Evans 1968; Tokovinin
//! 2014). A system is a tree: a leaf is one body, a node is a pair of
//! subtrees orbiting their common barycentre. Depth is unlimited, so a
//! triple, a 2+2 quadruple and a Castor-like sextuple are all one shape.
//!
//! Every pair whose member is itself a pair is tested with the Mardling and
//! Aarseth (2001) criterion, the inner pair against the other side as its
//! perturber. Unstable configurations are tagged with a `why`, never
//! refused: authored systems stand as written.
//!
//! The generator's chain (`stars::Star` with each companion around
//! everything inside it) is the special case where every outer pair has one
//! leaf; `from_chain` turns it into a tree so both share one check.

use crate::stars::{critical_ratio, period_days, Star};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// One orbit of a pair: the two sides about their barycentre.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PairOrbit {
    /// Separation of the two sides' barycentres, AU.
    pub semi_major_axis_au: f64,
    pub eccentricity: f64,
    /// Mutual inclination to the orbits inside it, degrees (0 = coplanar).
    #[serde(default)]
    pub inclination_deg: f64,
}

/// A multiple system as nested pairs.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Node {
    /// One body: `id` names it (a star's index, or any address the caller
    /// keeps), `mass_msun` its mass.
    Body { id: String, mass_msun: f64 },
    /// Two subtrees about their common barycentre.
    Pair {
        a: Box<Node>,
        b: Box<Node>,
        orbit: PairOrbit,
    },
}

impl Node {
    pub fn body(id: impl Into<String>, mass_msun: f64) -> Self {
        Node::Body {
            id: id.into(),
            mass_msun,
        }
    }

    pub fn pair(a: Node, b: Node, a_au: f64, e: f64) -> Self {
        Node::Pair {
            a: Box::new(a),
            b: Box::new(b),
            orbit: PairOrbit {
                semi_major_axis_au: a_au,
                eccentricity: e,
                inclination_deg: 0.0,
            },
        }
    }

    /// Total mass below this node, solar masses.
    pub fn mass(&self) -> f64 {
        match self {
            Node::Body { mass_msun, .. } => *mass_msun,
            Node::Pair { a, b, .. } => a.mass() + b.mass(),
        }
    }

    /// Number of bodies below this node.
    pub fn count(&self) -> usize {
        match self {
            Node::Body { .. } => 1,
            Node::Pair { a, b, .. } => a.count() + b.count(),
        }
    }

    /// Levels of nesting (a body is 0, a binary 1, a triple or 2+2 is 2).
    pub fn depth(&self) -> usize {
        match self {
            Node::Body { .. } => 0,
            Node::Pair { a, b, .. } => 1 + a.depth().max(b.depth()),
        }
    }

    /// A short name for this subtree: body ids joined, pairs bracketed,
    /// e.g. "(Aa+Ab)+(Ba+Bb)".
    pub fn label(&self) -> String {
        match self {
            Node::Body { id, .. } => id.clone(),
            Node::Pair { a, b, .. } => {
                let w = |n: &Node| match n {
                    Node::Body { .. } => n.label(),
                    _ => format!("({})", n.label()),
                };
                format!("{}+{}", w(a), w(b))
            }
        }
    }
}

/// One pair's facts and its stability against each inner pair.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PairCheck {
    /// The pair, by `Node::label`.
    pub pair: String,
    /// Path from the root: "" for the root, then "a"/"b" per step down.
    pub path: String,
    pub mass_msun: f64,
    pub period_days: f64,
    /// Smallest outer-to-inner ratio over its inner pairs, with the
    /// Mardling and Aarseth critical ratio for that inner pair; None when
    /// both sides are single bodies.
    pub ratio: Option<f64>,
    pub critical: Option<f64>,
    pub stable: bool,
    pub why: String,
}

/// Check every pair in the tree, root first, depth first (a before b).
pub fn check(root: &Node) -> Vec<PairCheck> {
    let mut out = vec![];
    walk(root, String::new(), &mut out);
    out
}

fn walk(n: &Node, path: String, out: &mut Vec<PairCheck>) {
    let Node::Pair { a, b, orbit } = n else {
        return;
    };
    let total = n.mass();
    let p = period_days(orbit.semi_major_axis_au, total.max(1e-12));
    let mut worst: Option<(f64, f64, String)> = None;
    let mut stable = true;
    for (inner, other) in [(a.as_ref(), b.as_ref()), (b.as_ref(), a.as_ref())] {
        if let Node::Pair { orbit: io, .. } = inner {
            if io.semi_major_axis_au <= 0.0 {
                continue;
            }
            let ratio = orbit.semi_major_axis_au / io.semi_major_axis_au;
            let q = other.mass() / inner.mass().max(1e-12);
            let incl = (orbit.inclination_deg - io.inclination_deg)
                .abs()
                .to_radians();
            let crit = critical_ratio(q, orbit.eccentricity, incl.min(m::PI));
            let ok = ratio > crit;
            stable &= ok;
            let margin = ratio / crit;
            if worst.as_ref().is_none_or(|w| margin < w.0 / w.1) {
                worst = Some((ratio, crit, inner.label()));
            }
        }
    }
    let why = match &worst {
        None => format!(
            "a simple pair: {:.3} au, period {:.1} days",
            orbit.semi_major_axis_au, p
        ),
        Some((r, c, inner)) => format!(
            "{} the pair {inner}: {r:.1} times its orbit; it needs more than {c:.1} (Mardling and Aarseth 2001){}",
            if stable { "stable around" } else { "unstable around" },
            if stable {
                ""
            } else {
                "; the system would break up within a few outer orbits"
            }
        ),
    };
    out.push(PairCheck {
        pair: n.label(),
        path: path.clone(),
        mass_msun: total,
        period_days: p,
        ratio: worst.as_ref().map(|w| w.0),
        critical: worst.as_ref().map(|w| w.1),
        stable,
        why,
    });
    walk(a, format!("{path}a"), out);
    walk(b, format!("{path}b"), out);
}

/// The generator's chain as a tree: the primary and the first companion
/// form the innermost pair; each later companion pairs with everything
/// inside it. Body ids are the stars' indices in the chain.
pub fn from_chain(stars: &[Star]) -> Option<Node> {
    let mut it = stars.iter().enumerate();
    let (_, first) = it.next()?;
    let mut node = Node::body("0", first.mass);
    for (i, s) in it {
        let o = s.orbit.as_ref()?;
        node = Node::pair(
            node,
            Node::body(i.to_string(), s.mass),
            o.semi_major_axis_au,
            o.eccentricity,
        );
    }
    Some(node)
}

#[cfg(test)]
#[path = "hierarchy_tests.rs"]
mod tests;
