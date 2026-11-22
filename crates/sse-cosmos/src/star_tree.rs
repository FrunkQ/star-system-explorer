//! Y1: the count tree (night sky design section 4.2).
//!
//! Stars fall into classes by luminosity (`StarParams::class_edges_lsun`).
//! For each class:
//! - a **root block** (`2^tree_depth` cells on a side, 640 pc by default)
//!   gets a Poisson count from its expected number;
//! - each block splits its count among its eight children by a keyed
//!   multinomial, each child's share in proportion to its expected number,
//!   down to the 10 pc cell;
//! - at the leaf each star is drawn as G5 draws it, with its mass and birth
//!   drawn on condition of its class (keyed rejection, so the conditioning
//!   is exact).
//!
//! Splitting a Poisson count multinomially in proportion to the children's
//! expectations gives independent Poisson counts in the children (the
//! splitting property), so each cell's statistics are G5's. A sky draw then
//! walks only the branches that hold a class visible at its distance, so its
//! cost grows with the stars found and not with the volume. One star has one
//! place: the bright star a sky walk finds 1 kpc away is the same star the
//! cell draw gives when someone flies there, because both use the same keys.
//!
//! Expected numbers: a cell's is G5's own `expected(cell)`. A larger node's
//! is the envelope at its centre times its volume, and a root block's is the
//! sum of its eight children's. A cell's mean is then G5's up to the
//! curvature of the density across each node, a second-order difference
//! (fudge F34).
//!
//! The class shares of a root block come from a quadrature over the IMF and
//! the ring's star formation history at the block's centre radius. Fudge
//! F34 records that they are held over the block, along with the normal
//! approximation of large binomials, the same one `haloes::poisson` uses
//! above 50.

use crate::stars::{birth_from, imf_quantile, StarSystem, Stars};
use sse_core::math as m;
use sse_core::{Address, Draws, Stream};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// A tree node: level (0 is the root block) and its index at that level.
type Node = (u32, [i64; 3]);

/// A node's split of one class's count among its children.
type Split = Vec<(Node, u64)>;

/// A walk through the tree in one frame, remembering expected numbers and
/// class shares so a batch of cells shares the work.
pub struct TreeWalk<'s, 'a> {
    pub stars: &'s Stars<'a>,
    pub frame: i64,
    expect: RefCell<BTreeMap<Node, f64>>,
    shares: RefCell<BTreeMap<[i64; 2], ClassMix>>,
    /// Splits already made, by node and class (a split is keyed, so this
    /// only saves the work).
    splits: RefCell<BTreeMap<(Node, usize), Split>>,
}

/// A root block's mix of classes: each class's share of its systems, and
/// the corner of mass and age where that class lives, so a leaf draw
/// conditioned on a rare bright class searches only there.
#[derive(Clone, Debug)]
pub struct ClassMix {
    pub share: Vec<f64>,
    /// Lowest IMF unit (mass quantile) any member can have, with a margin.
    pub u_floor: Vec<f64>,
    /// Greatest age any member can have, Gyr, with a margin.
    pub age_ceiling_gyr: Vec<f64>,
}

/// Share of a birth history's stars born by `t` (Gyr).
fn born_by(cdf: &[(f64, f64)], acc: f64, t: f64) -> f64 {
    if acc <= 0.0 {
        return 0.0;
    }
    let v = match cdf.windows(2).find(|w| w[1].0 >= t) {
        None => acc,
        Some(w) if t <= w[0].0 => w[0].1,
        Some(w) => {
            let f = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-300);
            w[0].1 + f * (w[1].1 - w[0].1)
        }
    };
    (v / acc).clamp(0.0, 1.0)
}

impl TreeWalk<'_, '_> {
    /// Envelope reads made so far (the walk's cost).
    pub fn reads(&self) -> usize {
        self.expect.borrow().len()
    }
}

/// A keyed binomial draw.
fn binomial(d: &Draws, i: u64, n: u64, p: f64) -> u64 {
    if n == 0 || p <= 0.0 {
        return 0;
    }
    if p >= 1.0 {
        return n;
    }
    if p > 0.5 {
        return n - binomial(d, i, n, 1.0 - p);
    }
    let nf = n as f64;
    let var = nf * p * (1.0 - p);
    if var > 25.0 {
        let x = nf * p + m::sqrt(var) * d.normal(i);
        return (m::round(x).max(0.0) as u64).min(n);
    }
    // Exact inversion from zero (the mean is small here).
    let u = d.unit(2 * i + 1);
    let r = p / (1.0 - p);
    let mut pk = m::exp(nf * m::log1p(-p));
    let mut cdf = pk;
    let mut k = 0u64;
    while u > cdf && k < n {
        pk *= (nf - k as f64) / (k as f64 + 1.0) * r;
        k += 1;
        cdf += pk;
    }
    k
}

impl<'s, 'a> TreeWalk<'s, 'a> {
    pub fn new(stars: &'s Stars<'a>, frame: i64) -> Self {
        TreeWalk {
            stars,
            frame,
            expect: RefCell::new(BTreeMap::new()),
            shares: RefCell::new(BTreeMap::new()),
            splits: RefCell::new(BTreeMap::new()),
        }
    }

    fn depth(&self) -> u32 {
        self.stars.params.tree_depth
    }

    /// Cells on a side of a node at `level`.
    fn span(&self, level: u32) -> i64 {
        1i64 << (self.depth() - level)
    }

    /// The node at `level` that holds `cell`.
    pub fn node_of(&self, cell: [i64; 3], level: u32) -> Node {
        let s = self.span(level);
        (
            level,
            [
                cell[0].div_euclid(s),
                cell[1].div_euclid(s),
                cell[2].div_euclid(s),
            ],
        )
    }

    fn children(&self, n: Node) -> Vec<Node> {
        let (l, [x, y, z]) = n;
        (0..8)
            .map(|b| {
                (
                    l + 1,
                    [
                        2 * x + (b & 1),
                        2 * y + ((b >> 1) & 1),
                        2 * z + ((b >> 2) & 1),
                    ],
                )
            })
            .collect()
    }

    /// Expected systems in a node: G5's own for a cell, and above that the
    /// envelope averaged over the node's height times its volume (fudge F34).
    pub fn expected(&self, n: Node) -> f64 {
        if let Some(v) = self.expect.borrow().get(&n) {
            return *v;
        }
        let st = self.stars;
        let c = st.params.cell_pc;
        let span = self.span(n.0);
        let v = if span == 1 {
            st.expected(n.1, self.frame)
        } else {
            let size = span as f64 * c;
            let t0 = self.frame as f64 * st.params.frame_myr;
            let g = [
                (n.1[0] as f64 + 0.5) * size,
                (n.1[1] as f64 + 0.5) * size,
                (n.1[2] as f64 + 0.5) * size,
            ];
            // Four-point Gauss-Legendre in height: the disc's vertical
            // profile is steep and curved, so a centre sample alone reads a
            // coarse node a few per cent low and the counts drift from the
            // cells' own means. Across the plane the envelope varies on
            // kiloparsec scales, so the centre serves there.
            const GL: [(f64, f64); 4] = [
                (-0.861_136_311_594_052_6, 0.347_854_845_137_453_9),
                (-0.339_981_043_584_856_3, 0.652_145_154_862_546_1),
                (0.339_981_043_584_856_3, 0.652_145_154_862_546_1),
                (0.861_136_311_594_052_6, 0.347_854_845_137_453_9),
            ];
            let mean: f64 = GL
                .iter()
                .map(|(x, w)| 0.5 * w * st.density_at([g[0], g[1], g[2] + 0.5 * size * x], t0))
                .sum();
            mean * size * size * size
        };
        self.expect.borrow_mut().insert(n, v);
        v
    }

    /// A root block's expected systems: the sum of its children's, so the
    /// first split is consistent with it.
    fn root_expected(&self, root: [i64; 3]) -> f64 {
        self.children((0, root))
            .iter()
            .map(|c| self.expected(*c))
            .sum()
    }

    /// Share of each class among a root block's systems: a quadrature over
    /// the IMF (fine in the bright tail) and the birth history.
    pub fn class_shares(&self, root: [i64; 3]) -> Vec<f64> {
        self.class_mix(root).share
    }

    /// The class shares with each class's corner of mass and age.
    pub fn class_mix(&self, root: [i64; 3]) -> ClassMix {
        // The mix depends on the block's radius only, so blocks stacked in
        // height share one.
        let column = [root[0], root[1]];
        if let Some(v) = self.shares.borrow().get(&column) {
            return v.clone();
        }
        let st = self.stars;
        let p = &st.params;
        let c = p.cell_pc;
        let span = self.span(0) as f64;
        let cx = (root[0] as f64 + 0.5) * span * c;
        let cy = (root[1] as f64 + 0.5) * span * c;
        let rc = m::hypot(cx, cy) / 1000.0;
        let t_now = self.frame as f64 * p.frame_myr / 1000.0;
        let (cdf, acc) = st.birth_cdf_at(rc, t_now);
        // Ages on a log grid, so the short lives of bright stars are
        // resolved, each bin weighted by the births in it.
        let born_by = |t: f64| born_by(&cdf, acc, t);
        let oldest = (t_now - st.env.disc.t_start_gyr).max(1e-3);
        let na = 48;
        let edges: Vec<f64> = (0..=na)
            .map(|k| {
                if k == 0 {
                    0.0
                } else {
                    1.0e-4 * m::pow(oldest / 1.0e-4, (k - 1) as f64 / (na - 1) as f64)
                }
            })
            .collect();
        // (mid age, births, the next bin's outer edge as the age margin)
        let bins: Vec<(f64, f64, f64)> = edges
            .windows(2)
            .enumerate()
            .map(|(k, e)| {
                let mid = if e[0] > 0.0 {
                    m::sqrt(e[0] * e[1])
                } else {
                    0.5 * e[1]
                };
                let margin = edges[(k + 2).min(na)];
                (mid, born_by(t_now - e[0]) - born_by(t_now - e[1]), margin)
            })
            .filter(|b| b.1 > 0.0)
            .collect();
        let classes = p.class_edges_lsun.len() + 1;
        let mut w = vec![0.0; classes];
        let mut u_floor = vec![1.0f64; classes];
        let mut ceiling = vec![0.0f64; classes];
        // Three grids in the IMF's unit: the bulk, the brightest per cent and
        // the brightest per mille.
        for (lo, hi, nm) in [(0.0, 0.99, 256), (0.99, 0.999, 128), (0.999, 1.0, 128)] {
            let du = (hi - lo) / nm as f64;
            for i in 0..nm {
                let u = lo + (i as f64 + 0.5) * du;
                let mass = imf_quantile(p, u);
                for (age, share, margin) in &bins {
                    let l = p.state(mass, *age).luminosity_lsun;
                    let c = p.class_of(l);
                    w[c] += du * share;
                    // Two grid steps below, one age bin beyond.
                    u_floor[c] = u_floor[c].min(u - 2.5 * du);
                    ceiling[c] = ceiling[c].max(*margin);
                }
            }
        }
        let total: f64 = w.iter().sum();
        if total > 0.0 {
            for v in w.iter_mut() {
                *v /= total;
            }
        }
        let mix = ClassMix {
            share: w,
            u_floor: u_floor.into_iter().map(|u| u.clamp(0.0, 1.0)).collect(),
            age_ceiling_gyr: ceiling
                .into_iter()
                .map(|a| if a > 0.0 { a } else { oldest })
                .collect(),
        };
        self.shares.borrow_mut().insert(column, mix.clone());
        mix
    }

    fn node_address(&self, n: Node) -> Result<Address, String> {
        self.stars
            .galaxy
            .child_key(
                "block",
                &format!("{}_{}_{}_{}", n.0, n.1[0], n.1[1], n.1[2]),
            )
            .map_err(|e| format!("{e:?}"))
    }

    fn counts_stream(&self, n: Node) -> Result<Draws, String> {
        Ok(Draws::new(
            self.stars.seed,
            self.node_address(n)?,
            Stream::new(&format!(
                "cosmos.stars.tree.v{}",
                self.stars.params.key_version
            ))
            .map_err(|e| e.to_string())?,
        ))
    }

    /// Count of `class` in the root block.
    fn root_count(&self, root: [i64; 3], class: usize) -> Result<u64, String> {
        let lambda = self.root_expected(root) * self.class_shares(root)[class];
        let d = self.counts_stream((0, root))?;
        let i = (self.frame as u64)
            .wrapping_mul(1 << 12)
            .wrapping_add(4000 + class as u64);
        Ok(crate::haloes::poisson(&d, i, lambda))
    }

    /// Split a node's count of `class` among its children.
    fn split(&self, n: Node, class: usize, count: u64) -> Result<Vec<(Node, u64)>, String> {
        if let Some(v) = self.splits.borrow().get(&(n, class)) {
            if v.iter().map(|c| c.1).sum::<u64>() == count {
                return Ok(v.clone());
            }
        }
        let v = self.split_fresh(n, class, count)?;
        self.splits.borrow_mut().insert((n, class), v.clone());
        Ok(v)
    }

    fn split_fresh(&self, n: Node, class: usize, count: u64) -> Result<Vec<(Node, u64)>, String> {
        let ch = self.children(n);
        if count == 0 {
            return Ok(ch.into_iter().map(|c| (c, 0)).collect());
        }
        let e: Vec<f64> = ch.iter().map(|c| self.expected(*c)).collect();
        let total: f64 = e.iter().sum();
        let d = self.counts_stream(n)?;
        let mut left = count;
        let mut rest = total;
        let mut out = vec![];
        for (k, c) in ch.into_iter().enumerate() {
            let take = if k == 7 || rest <= 0.0 {
                left
            } else {
                let i = (self.frame as u64)
                    .wrapping_mul(1 << 12)
                    .wrapping_add((class as u64) * 16 + k as u64);
                binomial(&d, i, left, e[k] / rest)
            };
            left -= take;
            rest -= e[k];
            out.push((c, take));
        }
        Ok(out)
    }

    /// Count of `class` in one cell, walked from its root block.
    pub fn cell_count(&self, cell: [i64; 3], class: usize) -> Result<u64, String> {
        let root = self.node_of(cell, 0).1;
        let mut count = self.root_count(root, class)?;
        for level in 0..self.depth() {
            let here = self.node_of(cell, level);
            let want = self.node_of(cell, level + 1);
            count = self
                .split(here, class, count)?
                .into_iter()
                .find(|(c, _)| *c == want)
                .map_or(0, |(_, k)| k);
            if count == 0 {
                return Ok(0);
            }
        }
        Ok(count)
    }

    /// Every cell holding stars of `class` inside a sphere (grid coordinates,
    /// pc), with their counts: only branches that hold some are walked.
    pub fn cells_within(
        &self,
        class: usize,
        centre: [f64; 3],
        radius_pc: f64,
    ) -> Result<Vec<([i64; 3], u64)>, String> {
        let c = self.stars.params.cell_pc;
        let root_pc = self.span(0) as f64 * c;
        let lo = |v: f64| m::floor((v - radius_pc) / root_pc) as i64;
        let hi = |v: f64| m::floor((v + radius_pc) / root_pc) as i64;
        let mut stack = vec![];
        for x in lo(centre[0])..=hi(centre[0]) {
            for y in lo(centre[1])..=hi(centre[1]) {
                for z in lo(centre[2])..=hi(centre[2]) {
                    if self.meets((0, [x, y, z]), centre, radius_pc) {
                        let n = self.root_count([x, y, z], class)?;
                        if n > 0 {
                            stack.push(((0, [x, y, z]), n));
                        }
                    }
                }
            }
        }
        let mut out = vec![];
        while let Some((node, n)) = stack.pop() {
            if node.0 == self.depth() {
                out.push((node.1, n));
                continue;
            }
            for (ch, k) in self.split(node, class, n)? {
                if k > 0 && self.meets(ch, centre, radius_pc) {
                    stack.push((ch, k));
                }
            }
        }
        out.sort();
        Ok(out)
    }

    /// Whether a node's box meets a sphere.
    fn meets(&self, n: Node, centre: [f64; 3], r: f64) -> bool {
        let size = self.span(n.0) as f64 * self.stars.params.cell_pc;
        let mut d2 = 0.0;
        for (ix, v) in n.1.iter().zip(centre) {
            let lo = *ix as f64 * size;
            let hi = lo + size;
            let gap = if v < lo {
                lo - v
            } else if v > hi {
                v - hi
            } else {
                0.0
            };
            d2 += gap * gap;
        }
        d2 <= r * r
    }

    /// The systems of one class in one cell (all of them, in slot order,
    /// when `class` is `None`).
    pub fn draw(&self, cell: [i64; 3], class: Option<usize>) -> Result<Vec<StarSystem>, String> {
        let st = self.stars;
        let p = &st.params;
        let frame = self.frame;
        let addr = st.cell_address(cell)?;
        let t0 = frame as f64 * p.frame_myr;
        let t_now = t0 / 1000.0;
        let (cdf, acc) = st.birth_cdf(cell, t_now);
        let classes = p.class_edges_lsun.len() + 1;
        let mix = self.class_mix(self.node_of(cell, 0).1);
        let mut out = vec![];
        let mut slot = 0u64;
        for c in 0..classes {
            let n = self.cell_count(cell, c)?;
            if class.is_some_and(|k| k != c) {
                slot += n;
                continue;
            }
            let d = Draws::new(
                st.seed,
                addr.clone(),
                Stream::new(&format!("cosmos.stars.c{c}.v{}", p.key_version))
                    .map_err(|e| e.to_string())?,
            );
            // Search only the corner of mass and age where the class lives.
            let fu = if mix.u_floor[c] < 1.0 {
                mix.u_floor[c]
            } else {
                0.0
            };
            let fb = born_by(&cdf, acc, t_now - mix.age_ceiling_gyr[c]);
            for j in 0..n {
                let i = |f: u64| {
                    (frame as u64)
                        .wrapping_mul(1 << 44)
                        .wrapping_add((j << 28).wrapping_add(f))
                };
                // Mass and birth on condition of the class: keyed rejection.
                let mut best: Option<(f64, f64, f64)> = None;
                let mut found = None;
                for a in 0..p.class_attempts {
                    let ia = |f: u64| i((1 + a) * 64 + f);
                    let birth = birth_from(&cdf, acc, fb + (1.0 - fb) * d.unit(ia(3)), t_now);
                    let mass = imf_quantile(p, fu + (1.0 - fu) * d.unit(ia(4)));
                    let l = p.state(mass, t_now - birth).luminosity_lsun;
                    if p.class_of(l) == c {
                        found = Some((mass, birth));
                        break;
                    }
                    let miss = p.class_miss(l, c);
                    if best.is_none_or(|b| miss < b.2) {
                        best = Some((mass, birth, miss));
                    }
                }
                let (mass, birth) = found.unwrap_or_else(|| {
                    let b = best.expect("at least one attempt");
                    (b.0, b.1)
                });
                out.push(st.complete(&d, &i, &addr, cell, frame, slot, mass, birth)?);
                slot += 1;
            }
        }
        Ok(out)
    }
}

impl<'a> Stars<'a> {
    /// G5's draw for a cell, through the count tree (`key_version` 2).
    pub(crate) fn draw_tree(
        &self,
        cell: [i64; 3],
        frame: i64,
        class: Option<usize>,
    ) -> Result<Vec<StarSystem>, String> {
        TreeWalk::new(self, frame).draw(cell, class)
    }
}

impl crate::stars::StarParams {
    /// The luminosity class of a star (index into the class edges).
    pub fn class_of(&self, l_lsun: f64) -> usize {
        self.class_edges_lsun
            .iter()
            .filter(|e| l_lsun >= **e)
            .count()
    }

    /// How far a luminosity is from a class, in decades (0 inside it).
    fn class_miss(&self, l_lsun: f64, class: usize) -> f64 {
        let e = &self.class_edges_lsun;
        let ll = m::log10(l_lsun.max(1e-30));
        let lo = if class == 0 {
            f64::NEG_INFINITY
        } else {
            m::log10(e[class - 1])
        };
        let hi = if class >= e.len() {
            f64::INFINITY
        } else {
            m::log10(e[class])
        };
        if ll < lo {
            lo - ll
        } else if ll >= hi {
            ll - hi
        } else {
            0.0
        }
    }
}
