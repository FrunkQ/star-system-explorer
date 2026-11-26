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
//! - `shift`: seen over emitted frequency, from the block in force
//!   `sse.light.disc_shift` (Luminet 1979) at (r, λ). The block is
//!   evaluated with b = |λ|, i = π/2 and α = ±π/2, which gives the same
//!   b sin i sin α; the bake checks at probe points that the block in force
//!   depends on its inputs only through r and λ, and says so when it does not.
//! - `t4`: the fourth power of the ring's own temperature, K⁴
//!   (`sse_light::disc_temperature`, Shakura and Sunyaev). T⁴ rather than T
//!   because T rises from zero at the inner edge as (r − r_in)^¼, which no
//!   linear read follows, while T⁴ rises linearly there.
//!
//! The renderer reads both at (r, λ/r) and sees the point glowing at
//! g · (t4)^¼.

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

/// What the table is baked for: one thin disc round a compact mass, as
/// `read_light_paths` describes it (`feed.disc` and the thin flow's rate).
#[derive(Clone, Debug, Deserialize)]
pub struct DiscSubject {
    /// The mass, kg; or give `gravitational_radius_m` instead.
    #[serde(default)]
    pub mass_kg: Option<f64>,
    #[serde(default)]
    pub gravitational_radius_m: Option<f64>,
    /// The rate the thin disc carries, kg/s.
    pub rate_kg_s: f64,
    pub inner_m: f64,
    pub outer_m: f64,
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
    (size.pick(384, 1024, 2048), size.pick(17, 33, 65))
}

/// What the table promises, per channel (shift, t4), by size: the worst
/// error of a linear read between nodes, relative to the exact value (for
/// t4, relative to a tenth of the hottest where the disc is dimmer than
/// that: T⁴ rises from zero at the inner edge). Measured on the three
/// fixture discs, 26 Nov 2026: shift 0.34 %, 0.09 %, 0.02 %; t4 4.8 %,
/// 0.9 %, 0.22 % (a quarter of that in T).
pub fn bounds(size: Size) -> [f64; 2] {
    size.pick([5e-3, 6e-2], [1.5e-3, 1.2e-2], [4e-4, 3e-3])
}

/// Bake the disc table from the blocks in force.
pub fn bake(
    subject: &serde_json::Value,
    size: Size,
    blocks: &[FormulaAtom],
) -> Result<Baked, String> {
    let s: DiscSubject = serde_json::from_value(subject.clone())
        .map_err(|e| format!("a disc subject is {{mass_kg or gravitational_radius_m, rate_kg_s, inner_m, outer_m}}: {e}"))?;
    let shift = Arc::new(block(blocks, SHIFT_BLOCK)?);
    let rg_block = block(blocks, RG_BLOCK)?;
    let rg = match (s.gravitational_radius_m, s.mass_kg) {
        (Some(rg), _) => rg,
        (None, Some(mass)) => run(&rg_block, &[mass])?,
        (None, None) => return Err("a disc subject needs mass_kg or gravitational_radius_m".into()),
    };
    if !(rg > 0.0 && s.rate_kg_s >= 0.0 && s.inner_m > 0.0 && s.outer_m > s.inner_m) {
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
    let (r_in, r_out) = (s.inner_m / rg, s.outer_m / rg);
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
            "shift",
            "1",
            SHIFT_BLOCK,
            "seen frequency over emitted frequency (Luminet 1979); 0 where no light arrives",
        ),
        Channel::new(
            "t4",
            "K4",
            "sse_light::disc_temperature",
            "the fourth power of the ring's own temperature (Shakura and Sunyaev); the point is seen glowing at shift × t4^¼",
        ),
    ];
    let (rate, r_in_m) = (s.rate_kg_s, s.inner_m);
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
        Ok(vec![g, (t * t) * (t * t)])
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
        let via = exact(&[r, lambda / r])?[0];
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
            "Read it at r and b sin i sin α / r; the point glows at shift × t4^¼.".into(),
        ],
        stand_ins,
    })
}
