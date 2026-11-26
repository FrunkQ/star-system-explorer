//! ER11a: the black hole disc as a baked table (`sse.table/1`, edits-reach
//! design section 11). The picture (XO4) samples it instead of repeating the
//! disc's laws on the GPU, so an edit to a law changes the picture.
//!
//! Two axes, so the table needs no rebake when the viewer drags the tilt:
//!
//! - `r`: where on the disc the ray landed, in gravitational radii, spaced
//!   logarithmically from the disc's inner edge to its outer edge;
//! - `lambda_over_r`: the ray's angular momentum about the disc's axis,
//!   λ = b sin i sin α (in r_g; conserved along the bent ray), over r.
//!   A photon leaving radius r for a far viewer has b ≤ r / √(1 − 2r_g/r)
//!   (its effective potential), so |λ/r| never exceeds that at the inner
//!   edge; the axis spans exactly that.
//!
//! Two channels:
//!
//! - `redshift`: emitted over seen frequency, 1 + z = 1/g, from the block
//!   in force `sse.light.disc_shift` (Luminet 1979) at (r, λ). The inverse
//!   is tabled because it is what varies gently: for Luminet's law it is
//!   linear in λ, so a linear read across λ is exact there, while g itself
//!   curves (a table of g read 30 times less accurately at the same size). The block is
//!   evaluated with b = |λ|, i = π/2 and α = ±π/2, which gives the same
//!   b sin i sin α; the bake checks at probe points that the block in force
//!   depends on its inputs only through r and λ, and says so when it does not.
//! - `t4`: the fourth power of the ring's own temperature, K⁴
//!   (`sse_light::disc_temperature`, Shakura and Sunyaev). T⁴ rather than T
//!   because T rises from zero at the inner edge as (r − r_in)^¼, which no
//!   linear read follows, while T⁴ rises linearly there.
//!
//! The renderer reads both at (r, λ/r) and sees the point glowing at
//! (t4)^¼ / redshift.

use crate::m;
use serde::Deserialize;
use sse_formula::FormulaAtom;
use sse_physics::table::{Axis, Baked, Channel, Read, Size, Spacing};
use std::sync::Arc;

/// The table's id.
pub const ID: &str = "sse.light.disc";

/// The block whose law the shift channel bakes.
pub const SHIFT_BLOCK: &str = "sse.light.disc_shift";
const RG_BLOCK: &str = "sse.gravity.gravitational_radius";

/// What the table is baked for: one thin disc round a compact mass. Either
/// the numbers, or `feed` as `read_light_paths` gives it (the disc's edges,
/// and the rate the thin and thick flows carry, `Feed::disc_rate_kg_s`).
#[derive(Clone, Debug, Deserialize)]
pub struct DiscSubject {
    /// The mass, kg; or give `gravitational_radius_m` instead.
    #[serde(default)]
    pub mass_kg: Option<f64>,
    #[serde(default)]
    pub gravitational_radius_m: Option<f64>,
    /// The rate the disc carries, kg/s.
    #[serde(default)]
    pub rate_kg_s: Option<f64>,
    #[serde(default)]
    pub inner_m: Option<f64>,
    #[serde(default)]
    pub outer_m: Option<f64>,
    #[serde(default)]
    pub feed: Option<crate::feed::Feed>,
}

impl DiscSubject {
    fn disc(&self) -> Result<(f64, f64, f64), String> {
        let f = self.feed.as_ref();
        let rate = self.rate_kg_s.or(f.map(|f| f.disc_rate_kg_s()));
        let inner = self.inner_m.or(f.map(|f| f.disc.inner_m));
        let outer = self.outer_m.or(f.map(|f| f.disc.outer_m));
        match (rate, inner, outer) {
            (Some(r), Some(i), Some(o)) => Ok((r, i, o)),
            _ => Err("a disc subject needs rate_kg_s, inner_m and outer_m, or a feed".into()),
        }
    }
}

fn block(blocks: &[FormulaAtom], id: &str) -> Result<FormulaAtom, String> {
    let a = blocks
        .iter()
        .find(|a| sse_flow::Atom::decl(*a).id == id)
        .ok_or_else(|| format!("the block {id} is not in force"))?;
    if !sse_flow::Atom::decl(a).params.is_empty() {
        // Settings in force come with ER2's block door; this block has none.
        return Err(format!(
            "{id} has settings, which a table cannot read until ER2"
        ));
    }
    Ok(a.clone())
}

fn run(a: &FormulaAtom, inputs: &[f64]) -> Result<f64, String> {
    sse_flow::Atom::eval(a, inputs, &[])
        .map(|v| v[0])
        .map_err(|e| format!("{}: {}", sse_flow::Atom::decl(a).id, e.0))
}

fn version(a: &FormulaAtom) -> String {
    sse_flow::Atom::decl(a).version.to_string()
}

/// Nodes along each axis, by size.
pub fn nodes(size: Size) -> (usize, usize) {
    (size.pick(384, 1024, 2048), size.pick(9, 17, 33))
}

/// What the table promises, per channel (redshift, t4), by size: the worst
/// error of a linear read between nodes, relative to the exact value (for
/// t4, relative to a tenth of the hottest where the disc is dimmer than
/// that: T⁴ rises from zero at the inner edge). Measured on the three
/// fixture discs, 26 Nov 2026: redshift 0.016 %, 0.0023 %, 0.0006 %; t4
/// 4.8 %, 0.9 %, 0.22 % (a quarter of that in T, and those worst points sit
/// in the dim inner rim; at each disc's brightest point T is within 0.007 %
/// on the medium table).
pub fn bounds(size: Size) -> [f64; 2] {
    size.pick([2e-4, 6e-2], [3e-5, 1.2e-2], [8e-6, 3e-3])
}

/// Bake the disc table from the blocks in force.
pub fn bake(
    subject: &serde_json::Value,
    size: Size,
    blocks: &[FormulaAtom],
) -> Result<Baked, String> {
    let s: DiscSubject = serde_json::from_value(subject.clone())
        .map_err(|e| format!("a disc subject is {{mass_kg or gravitational_radius_m, and rate_kg_s, inner_m, outer_m or a feed}}: {e}"))?;
    let (rate_kg_s, inner_m, outer_m) = s.disc()?;
    let shift = Arc::new(block(blocks, SHIFT_BLOCK)?);
    let rg_block = block(blocks, RG_BLOCK)?;
    let rg = match (s.gravitational_radius_m, s.mass_kg) {
        (Some(rg), _) => rg,
        (None, Some(mass)) => run(&rg_block, &[mass])?,
        (None, None) => return Err("a disc subject needs mass_kg or gravitational_radius_m".into()),
    };
    if !(rg > 0.0 && rate_kg_s >= 0.0 && inner_m > 0.0 && outer_m > inner_m) {
        return Err(
            "a disc needs a mass, a rate of zero or more, and an outer edge beyond its inner edge"
                .into(),
        );
    }
    // The mass the temperature law reads, from r_g (the block in force).
    let mass_kg = match s.mass_kg {
        Some(mk) => mk,
        None => {
            let per_kg = run(&rg_block, &[1.0])?;
            rg / per_kg
        }
    };
    let (r_in, r_out) = (inner_m / rg, outer_m / rg);
    let q_max = 1.0 / m::sqrt(1.0 - 2.0 / r_in);
    if !q_max.is_finite() {
        return Err(format!(
            "the disc's inner edge, {r_in:.3} gravitational radii, is inside the horizon's double: no light leaves it for a far viewer"
        ));
    }
    let (nr, nq) = nodes(size);
    let axes = vec![
        Axis::new(
            "r",
            "r_g",
            Spacing::Log,
            r_in,
            r_out,
            nr,
            "where the ray landed on the disc, in gravitational radii",
        ),
        Axis::new(
            "lambda_over_r",
            "1",
            Spacing::Even,
            -q_max,
            q_max,
            nq,
            "the ray's angular momentum about the disc's axis, b sin i sin α (b in gravitational radii, i the viewer's tilt from the axis, α the picture angle from the axis as seen towards the left), over r",
        ),
    ];
    let channels = vec![
        Channel::new(
            "redshift",
            "1",
            SHIFT_BLOCK,
            "emitted frequency over seen frequency, 1 + z, the inverse of the block's shift g (Luminet 1979); 0 where no light arrives",
        ),
        Channel::new(
            "t4",
            "K4",
            "sse_light::disc_temperature",
            "the fourth power of the ring's own temperature (Shakura and Sunyaev); the point is seen glowing at t4^¼ / redshift",
        ),
    ];
    let (rate, r_in_m) = (rate_kg_s, inner_m);
    let sh = shift.clone();
    let exact = move |x: &[f64]| -> Result<Vec<f64>, String> {
        let (r, q) = (x[0], x[1]);
        let lambda = q * r;
        let side = if lambda < 0.0 {
            -m::PI / 2.0
        } else {
            m::PI / 2.0
        };
        let g = run(&sh, &[1.0, r, lambda.abs(), m::PI / 2.0, side])?;
        let t = crate::disc_temperature(mass_kg, rate, r_in_m, r * rg);
        Ok(vec![if g > 0.0 { 1.0 / g } else { 0.0 }, (t * t) * (t * t)])
    };
    // The indexing holds only for a block that reads b, i and α through
    // b sin i sin α: check that at a few probe points with other angles.
    let mut indexing = 0f64;
    for (k, (tilt, alpha)) in [(0.3, 1.1), (1.2, -0.4), (0.9, 2.6), (1.45, -2.2)]
        .into_iter()
        .enumerate()
    {
        let r = r_in * (1.5 + k as f64);
        let b = r * 0.7;
        let direct = run(&shift, &[1.0, r, b, tilt, alpha])?;
        let lambda = b * m::sin(tilt) * m::sin(alpha);
        let z = exact(&[r, lambda / r])?[0];
        let via = if z > 0.0 { 1.0 / z } else { 0.0 };
        let e = (direct - via).abs() / direct.abs().max(1e-12);
        indexing = indexing.max(e);
    }
    // Where the disc is dimmer than a tenth of its hottest T⁴, the error is
    // measured against that tenth (the glow is zero at the inner edge).
    let mut hottest = 0f64;
    for i in 0..nr {
        hottest = hottest.max(exact(&[axes[0].at(i), 0.0])?[1]);
    }
    let mut stand_ins = vec![];
    if indexing > 1e-9 {
        stand_ins.push(format!(
            "the block in force reads b, i and α other than through b sin i sin α (off by up to {indexing:.2e} at probe points), so this table, indexed by that product, does not hold for it"
        ));
    }
    Ok(Baked {
        axes,
        channels,
        exact: Box::new(exact),
        bound: bounds(size).to_vec(),
        // Shifts below a thousandth (no light arrives) are read as absolute.
        floor: vec![1e-3, hottest * 0.1],
        reads: vec![
            Read {
                kind: "block".into(),
                id: SHIFT_BLOCK.into(),
                version: Some(version(&shift)),
            },
            Read {
                kind: "block".into(),
                id: RG_BLOCK.into(),
                version: Some(version(&rg_block)),
            },
            Read {
                kind: "compiled".into(),
                id: "sse_light::disc_temperature".into(),
                version: None,
            },
        ],
        why: vec![
            format!(
                "The disc from {r_in:.3} to {r_out:.1} gravitational radii (r_g = {rg:.4e} m), fed at {rate:.4e} kg/s."
            ),
            "Read it at r and b sin i sin α / r; the point glows at t4^¼ / redshift.".into(),
        ],
        stand_ins,
    })
}
