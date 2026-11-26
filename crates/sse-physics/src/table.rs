//! `sse.table/1`: the one format the kernel bakes tables in for the
//! renderer (track ER, package ER11a; design
//! `handover/design/SSE4-edits-reach-design.md` section 11).
//!
//! The renderer keeps only geometry and lookups. A law whose input varies
//! across a picture is baked here into a table indexed by that input, from
//! the physics in force, and the renderer samples it. A table says:
//!
//! - its id, its axes (unit, even or logarithmic spacing, range, size) and
//!   its channels (unit, and where each comes from: a block or a compiled
//!   function), with the numbers as one flat array of 32-bit floats,
//!   channel fastest, then the first axis, then the second;
//! - the physics hash it was baked under, the blocks and sheet rows it
//!   read, and its `rests_on` edits (filled once ER8 marks results), so the
//!   picture can carry the mark;
//! - the size it was baked at, picked by the renderer's quality tier;
//! - how well it samples: the worst error of a multilinear read against
//!   the exact answer at probe points between the grid's nodes, beside the
//!   bound the table promises. Sampling is the one allowed approximation
//!   (fudge register, class "table sampling"); [`check_sampling`] is the
//!   helper every table's test uses.
//!
//! The renderer's copy of the sampler (`packages/ui-contract/src/table.ts`)
//! reads the same layout and is tested against this one.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// The format's name, carried by every table.
pub const FORMAT: &str = "sse.table/1";

/// How an axis's nodes are spaced between its ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spacing {
    /// Equal steps.
    Even,
    /// Equal steps in the logarithm (both ends above zero).
    Log,
}

/// One axis of a table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    /// What the axis is, as a short id (`"r"`, `"lambda_over_r"`).
    pub name: String,
    pub unit: String,
    pub spacing: Spacing,
    /// The first and last nodes (the ends are nodes).
    pub from: f64,
    pub to: f64,
    /// Nodes along the axis, at least two.
    pub n: usize,
    /// What a reader computes to index it, in words.
    pub about: String,
}

impl Axis {
    pub fn new(
        name: &str,
        unit: &str,
        spacing: Spacing,
        from: f64,
        to: f64,
        n: usize,
        about: &str,
    ) -> Self {
        Axis {
            name: name.into(),
            unit: unit.into(),
            spacing,
            from,
            to,
            n,
            about: about.into(),
        }
    }

    fn ends(&self) -> (f64, f64) {
        match self.spacing {
            Spacing::Even => (self.from, self.to),
            Spacing::Log => (m::ln(self.from), m::ln(self.to)),
        }
    }

    /// The value at node `i`.
    pub fn at(&self, i: usize) -> f64 {
        if i == 0 {
            return self.from;
        }
        if i + 1 == self.n {
            return self.to;
        }
        let (a, b) = self.ends();
        let s = a + (b - a) * i as f64 / (self.n - 1) as f64;
        match self.spacing {
            Spacing::Even => s,
            Spacing::Log => m::exp(s),
        }
    }

    /// The value at a fractional node index.
    pub fn at_index(&self, f: f64) -> f64 {
        let (a, b) = self.ends();
        let s = a + (b - a) * f / (self.n - 1) as f64;
        match self.spacing {
            Spacing::Even => s,
            Spacing::Log => m::exp(s),
        }
    }

    /// The fractional node index of `x`, held to the axis's ends.
    pub fn index_of(&self, x: f64) -> f64 {
        let (a, b) = self.ends();
        let s = match self.spacing {
            Spacing::Even => x,
            Spacing::Log => {
                if x > 0.0 {
                    m::ln(x)
                } else {
                    a
                }
            }
        };
        let f = (s - a) / (b - a) * (self.n - 1) as f64;
        if f.is_nan() {
            return 0.0;
        }
        f.clamp(0.0, (self.n - 1) as f64)
    }

    fn check(&self) -> Result<(), String> {
        let ok_ends = self.from.is_finite() && self.to.is_finite() && self.to > self.from;
        let ok_log = self.spacing == Spacing::Even || self.from > 0.0;
        if self.n < 2 || !ok_ends || !ok_log {
            return Err(format!(
                "axis {} needs two or more nodes and ends that rise{}",
                self.name,
                if self.spacing == Spacing::Log {
                    " from above zero"
                } else {
                    ""
                }
            ));
        }
        Ok(())
    }
}

/// One channel of a table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub unit: String,
    /// Where it comes from: a block id (`"sse.light.disc_shift"`) or a
    /// compiled function (`"sse_light::disc_temperature"`).
    pub from: String,
    pub about: String,
}

impl Channel {
    pub fn new(name: &str, unit: &str, from: &str, about: &str) -> Self {
        Channel {
            name: name.into(),
            unit: unit.into(),
            from: from.into(),
            about: about.into(),
        }
    }
}

/// The size a table is baked at. The renderer's quality tier picks it
/// ([`Size::of_tier`]); a phone gets the small size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    /// The renderer's drawing tiers (`ui-contract` `Tier`) and the device's
    /// measured tier (`basic`, `standard`, `capable`): `still`, `gpu-ultra`
    /// and `capable` take the large size, `gpu-high` and `standard` the
    /// medium, and everything else (`gpu-basic`, `canvas`, `text`, `basic`,
    /// a phone) the small. Unnamed: medium.
    pub fn of_tier(tier: Option<&str>) -> Self {
        match tier {
            Some("still" | "gpu-ultra" | "capable" | "large") => Size::Large,
            Some("gpu-high" | "standard" | "medium") | None => Size::Medium,
            Some(_) => Size::Small,
        }
    }

    /// Pick one of three values by size.
    pub fn pick<T: Copy>(self, small: T, medium: T, large: T) -> T {
        match self {
            Size::Small => small,
            Size::Medium => medium,
            Size::Large => large,
        }
    }
}

/// How well a table samples, measured when it is baked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sampling {
    /// Probe points read (midway between nodes, and a quarter of the way).
    pub probes: usize,
    /// Per channel, the worst error of a multilinear read: relative to the
    /// exact value, or absolute against `floor` where the exact value is
    /// smaller than it.
    pub worst: Vec<f64>,
    /// Per channel, the bound the table promises.
    pub bound: Vec<f64>,
    /// Per channel, the size below which the error is read as absolute.
    pub floor: Vec<f64>,
    /// Every channel within its bound.
    pub within: bool,
}

/// A table's exact function: one value per channel at a point.
pub type Exact = dyn Fn(&[f64]) -> Result<Vec<f64>, String>;

/// A block or sheet row a table read.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Read {
    /// `"block"`, `"sheet"` or `"compiled"`.
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// A baked table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// Always [`FORMAT`].
    pub format: String,
    pub id: String,
    /// What it was baked for, as asked (a disc, a body, a sea).
    pub subject: serde_json::Value,
    pub size: Size,
    /// The tier asked for, if one was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    pub axes: Vec<Axis>,
    pub channels: Vec<Channel>,
    /// Channel fastest, then the first axis, then the second.
    pub values: Vec<f32>,
    /// The hash of `values` with the axes and channels: the renderer keys
    /// its texture by it.
    pub hash: String,
    /// The physics hash it was baked under.
    pub physics: String,
    /// What it read.
    pub reads: Vec<Read>,
    /// The edits it rests on. Empty until ER8 marks results from the door.
    pub rests_on: Vec<String>,
    pub sampling: Sampling,
    /// What a reader must know, in words (how to index it, where it stops).
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
}

/// What a producer hands over; [`Table::seal`] stamps the rest.
pub struct Baked {
    pub axes: Vec<Axis>,
    pub channels: Vec<Channel>,
    /// The exact answer at a point (one coordinate per axis), one value per
    /// channel. The table's nodes are filled from it and its probes checked
    /// against it, so the two can never be different functions.
    pub exact: Box<Exact>,
    pub bound: Vec<f64>,
    pub floor: Vec<f64>,
    pub reads: Vec<Read>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
}

fn flat_index(axes: &[Axis], idx: &[usize]) -> usize {
    let mut k = 0;
    for d in (0..axes.len()).rev() {
        k = k * axes[d].n + idx[d];
    }
    k
}

impl Table {
    /// Fill the table from its exact function, probe it, and stamp it.
    pub fn seal(
        id: &str,
        subject: serde_json::Value,
        size: Size,
        tier: Option<String>,
        physics: String,
        b: Baked,
    ) -> Result<Table, String> {
        if b.axes.is_empty() || b.channels.is_empty() {
            return Err(format!("table {id} needs an axis and a channel"));
        }
        let nch = b.channels.len();
        if b.bound.len() != nch || b.floor.len() != nch {
            return Err(format!("table {id}: one bound and floor per channel"));
        }
        for a in &b.axes {
            a.check()?;
        }
        let cells: usize = b.axes.iter().map(|a| a.n).product();
        let mut values = vec![0f32; cells * nch];
        let mut idx = vec![0usize; b.axes.len()];
        for cell in 0..cells {
            let mut rest = cell;
            for (d, a) in b.axes.iter().enumerate() {
                idx[d] = rest % a.n;
                rest /= a.n;
            }
            let x: Vec<f64> = b.axes.iter().zip(&idx).map(|(a, &i)| a.at(i)).collect();
            let v = (b.exact)(&x)?;
            if v.len() != nch {
                return Err(format!("table {id}: {} values for {nch} channels", v.len()));
            }
            let k = flat_index(&b.axes, &idx) * nch;
            for c in 0..nch {
                if !v[c].is_finite() {
                    return Err(format!(
                        "table {id}: channel {} is not finite at {x:?}",
                        b.channels[c].name
                    ));
                }
                values[k + c] = v[c] as f32;
            }
        }
        let mut t = Table {
            format: FORMAT.into(),
            id: id.into(),
            subject,
            size,
            tier,
            axes: b.axes,
            channels: b.channels,
            values,
            hash: String::new(),
            physics,
            reads: b.reads,
            rests_on: vec![],
            sampling: Sampling {
                probes: 0,
                worst: vec![],
                bound: b.bound.clone(),
                floor: b.floor.clone(),
                within: false,
            },
            why: b.why,
            stand_ins: b.stand_ins,
        };
        t.sampling = check_sampling(&t, &*b.exact, &b.bound, &b.floor)?;
        t.hash = sse_core::ContentHash::of(&(&t.id, &t.axes, &t.channels, &t.values))
            .map_err(|e| format!("table {id} hashes: {e:?}"))?
            .hex();
        Ok(t)
    }

    /// The value of channel `c` at node `idx`.
    pub fn node(&self, idx: &[usize], c: usize) -> f64 {
        self.values[flat_index(&self.axes, idx) * self.channels.len() + c] as f64
    }

    /// Multilinear read of every channel at a point (one coordinate per
    /// axis), each coordinate held to its axis's ends. This is what the
    /// renderer's sampler does, on the CPU and the GPU.
    pub fn sample(&self, x: &[f64]) -> Vec<f64> {
        let nd = self.axes.len();
        let nch = self.channels.len();
        let mut lo = vec![0usize; nd];
        let mut fr = vec![0f64; nd];
        for d in 0..nd {
            let a = &self.axes[d];
            let f = a.index_of(x.get(d).copied().unwrap_or(a.from));
            let i = (m::floor(f) as usize).min(a.n - 2);
            lo[d] = i;
            fr[d] = f - i as f64;
        }
        let mut out = vec![0f64; nch];
        let mut idx = vec![0usize; nd];
        for corner in 0..(1usize << nd) {
            let mut w = 1.0;
            for d in 0..nd {
                let up = (corner >> d) & 1 == 1;
                idx[d] = lo[d] + up as usize;
                w *= if up { fr[d] } else { 1.0 - fr[d] };
            }
            if w == 0.0 {
                continue;
            }
            for (c, o) in out.iter_mut().enumerate() {
                *o += w * self.node(&idx, c);
            }
        }
        out
    }

    /// The channel's index by name.
    pub fn channel(&self, name: &str) -> Option<usize> {
        self.channels.iter().position(|c| c.name == name)
    }
}

/// The sampling test helper: read the table between its nodes (midway, and
/// a quarter of the way, along every axis) and compare with `exact`. Each
/// channel's error is relative to the exact value, or absolute against its
/// `floor` where the exact value is smaller than that. Every table's own
/// test asserts `within`; the bake op publishes the numbers.
pub fn check_sampling(
    t: &Table,
    exact: &Exact,
    bound: &[f64],
    floor: &[f64],
) -> Result<Sampling, String> {
    let nd = t.axes.len();
    let nch = t.channels.len();
    let mut worst = vec![0f64; nch];
    let mut probes = 0;
    // Probe every cell of a 1D table, and a spread of cells in more axes,
    // at fractions 0.25 and 0.5 along each axis.
    let stride: Vec<usize> = t
        .axes
        .iter()
        .map(|a| if nd == 1 { 1 } else { ((a.n - 1) / 24).max(1) })
        .collect();
    let counts: Vec<usize> = t
        .axes
        .iter()
        .zip(&stride)
        .map(|(a, s)| (a.n - 1).div_ceil(*s))
        .collect();
    let total: usize = counts.iter().product::<usize>() * (1 << nd);
    let mut x = vec![0f64; nd];
    for k in 0..total {
        let mut rest = k;
        let mut fracs = 0usize;
        for d in 0..nd {
            fracs |= (rest & 1) << d;
            rest >>= 1;
        }
        for d in 0..nd {
            let cell = (rest % counts[d]) * stride[d];
            rest /= counts[d];
            let a = &t.axes[d];
            let f = if (fracs >> d) & 1 == 1 { 0.5 } else { 0.25 };
            x[d] = a.at_index(cell as f64 + f);
        }
        let want = exact(&x)?;
        let got = t.sample(&x);
        for c in 0..nch {
            let err = (got[c] - want[c]).abs();
            let scale = want[c].abs();
            let e = if scale >= floor[c] {
                err / scale
            } else {
                err / floor[c]
            };
            if e > worst[c] {
                worst[c] = e;
            }
        }
        probes += 1;
    }
    let within = worst.iter().zip(bound).all(|(w, b)| w <= b);
    Ok(Sampling {
        probes,
        worst,
        bound: bound.to_vec(),
        floor: floor.to_vec(),
        within,
    })
}
