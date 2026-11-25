//! The bend table (exotic objects design 3.2, block `sse.gravity.light_path`).
//!
//! Layout, all lengths in gravitational radii r_g = GM/c²:
//!
//! - [`ROWS`] rows, one per impact parameter b. The first [`CAPTURED_ROWS`]
//!   are captured rays, b from 0 up to just below the critical value
//!   b_c = √27; the rest escape, b from just above b_c out to [`B_MAX`].
//!   Both halves crowd towards b_c (spacing grows with the square of the
//!   distance from it), where rays wind round the photon orbit and change
//!   fastest. [`b_of_row`] and [`row_of_b`] convert.
//! - [`COLS`] columns, the angle φ swept round the mass since the ray left
//!   the viewer far away, evenly from 0 to [`PHI_MAX`] = 3π: enough for the
//!   direct image, the arch over the top and the thin ring (three crossings
//!   of a disc's plane).
//! - Each cell is u = r_g / r at that angle: 0 far away, 1/2 at the surface
//!   of no return. Once a ray has escaped again its cells are 0; once it is
//!   swallowed they are 1/2.
//! - Each row also carries whether the ray is captured, the angle at which it
//!   escapes (its whole sweep) or is swallowed, its turn (sweep − π, the
//!   deflection of an escaping ray) and its closest approach.
//!
//! To trace a pixel the renderer works out the pixel's b and the plane its
//! ray lies in (design 3.2), then reads u at the angles where that plane
//! crosses the disc: r = 1/u, inside the disc's edges or not. An escaping
//! ray's turn gives the direction to look up in the sky behind.
//!
//! The viewer is taken as far away (b and φ measured from infinity). A camera
//! within about fifty r_g reads the same curves from part-way along; that,
//! and spin, are later packages (XO4, XO7).

use sse_core::hash::{ContentHash, Hasher};
use sse_core::math as m;

/// Rows (impact parameters).
pub const ROWS: usize = 256;
/// Rows below the critical impact parameter (captured rays).
pub const CAPTURED_ROWS: usize = 128;
/// Columns (angles swept).
pub const COLS: usize = 256;
/// Integration steps between two columns.
pub const SUB: usize = 32;
/// Largest impact parameter in the table, r_g. Beyond it the weak bender
/// (`sse.gravity.weak_bend`) is within 6% and its next term takes it to 0.1%.
pub const B_MAX: f64 = 64.0;
/// Largest angle swept in the table: three half turns.
pub const PHI_MAX: f64 = 3.0 * m::PI;
/// u at the surface of no return (r = 2 r_g).
pub const U_HORIZON: f64 = 0.5;
/// Content-hash domain (bump the version if the layout changes).
pub const DOMAIN: &str = "sse.gravity.bend_table/1";
/// Hardest stop for a ray that winds round the photon orbit: one hundred half
/// turns. No row comes near it (the rows nearest b_c sweep about three and a half).
const PHI_STOP: f64 = 100.0 * m::PI;

/// The critical impact parameter √27: rays inside it are swallowed. It is
/// also the shadow's edge seen from far away.
pub fn b_crit() -> f64 {
    3.0 * m::sqrt(3.0)
}

/// Impact parameter of a row, in r_g.
pub fn b_of_row(i: usize) -> f64 {
    let bc = b_crit();
    if i < CAPTURED_ROWS {
        let t = i as f64 / CAPTURED_ROWS as f64;
        let q = 1.0 - t;
        bc * (1.0 - q * q)
    } else {
        let s = (i - CAPTURED_ROWS + 1) as f64 / (ROWS - CAPTURED_ROWS) as f64;
        bc + (B_MAX - bc) * s * s
    }
}

/// Where an impact parameter falls among the rows, as a fractional row
/// index, clamped to its own side of b_c (a ray is never interpolated
/// between a captured row and an escaping one). `None` beyond [`B_MAX`].
pub fn row_of_b(b: f64) -> Option<f64> {
    let bc = b_crit();
    if !(0.0..=B_MAX).contains(&b) {
        return None;
    }
    if b < bc {
        let t = 1.0 - m::sqrt(1.0 - b / bc);
        Some((t * CAPTURED_ROWS as f64).min((CAPTURED_ROWS - 1) as f64))
    } else {
        let s = m::sqrt((b - bc) / (B_MAX - bc));
        let at = s * (ROWS - CAPTURED_ROWS) as f64 - 1.0 + CAPTURED_ROWS as f64;
        Some(at.max(CAPTURED_ROWS as f64))
    }
}

/// Angle swept at a column.
pub fn phi_of_col(k: usize) -> f64 {
    PHI_MAX * k as f64 / (COLS - 1) as f64
}

/// One impact parameter's ray.
#[derive(Clone, Debug, PartialEq)]
pub struct BendRow {
    /// Impact parameter, r_g.
    pub b: f64,
    /// Whether the ray is swallowed.
    pub captured: bool,
    /// Angle swept when the ray escapes again, or reaches the surface of no
    /// return when captured.
    pub end_phi: f64,
    /// For an escaping ray, how far it has been turned: end_phi − π. Zero
    /// for a captured ray.
    pub turn: f64,
    /// Closest approach as u = r_g / r: the turning point of an escaping ray,
    /// 1/2 for a captured one.
    pub closest_u: f64,
    /// u = r_g / r at each column's angle.
    pub u: Vec<f64>,
}

/// The whole table.
#[derive(Clone, Debug, PartialEq)]
pub struct BendTable {
    pub rows: Vec<BendRow>,
}

/// u'' = 3u² − u, as a step of the state (u, w = du/dφ).
fn rk4(u: f64, w: f64, h: f64) -> (f64, f64) {
    let f = |u: f64| 3.0 * u * u - u;
    let (k1u, k1w) = (w, f(u));
    let (u2, w2) = (u + 0.5 * h * k1u, w + 0.5 * h * k1w);
    let (k2u, k2w) = (w2, f(u2));
    let (u3, w3) = (u + 0.5 * h * k2u, w + 0.5 * h * k2w);
    let (k3u, k3w) = (w3, f(u3));
    let (u4, w4) = (u + h * k3u, w + h * k3w);
    let (k4u, k4w) = (w4, f(u4));
    (
        u + h / 6.0 * (k1u + 2.0 * k2u + 2.0 * k3u + k4u),
        w + h / 6.0 * (k1w + 2.0 * k2w + 2.0 * k3w + k4w),
    )
}

/// The turning point of an escaping ray: the smallest positive root of
/// 2u³ − u² + 1/b² = 0, which lies between 0 and 1/3 when b > √27. Found by
/// halving the interval (plain arithmetic, the same bits everywhere).
pub fn turning_point(b: f64) -> f64 {
    let c = 1.0 / (b * b);
    let f = |u: f64| 2.0 * u * u * u - u * u + c;
    let (mut lo, mut hi) = (0.0_f64, 1.0 / 3.0);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Trace one ray of impact parameter `b` (r_g) in from far away.
pub fn trace(b: f64) -> BendRow {
    let captured = b < b_crit();
    let mut u_col = vec![0.0; COLS];
    if b <= 0.0 {
        // Straight in: swallowed at once, sweeping no angle.
        for v in u_col.iter_mut().skip(1) {
            *v = U_HORIZON;
        }
        return BendRow {
            b,
            captured: true,
            end_phi: 0.0,
            turn: 0.0,
            closest_u: U_HORIZON,
            u: u_col,
        };
    }
    let h = PHI_MAX / ((COLS - 1) * SUB) as f64;
    let (mut u, mut w) = (0.0_f64, 1.0 / b);
    let mut end_phi = None;
    let mut step = 0usize;
    loop {
        let phi = h * step as f64;
        let (un, wn) = rk4(u, w, h);
        if un >= U_HORIZON {
            end_phi = Some(phi + h * (U_HORIZON - u) / (un - u));
        } else if un <= 0.0 && w < 0.0 {
            end_phi = Some(phi + h * u / (u - un));
        }
        if end_phi.is_some() {
            break;
        }
        u = un;
        w = wn;
        step += 1;
        if step.is_multiple_of(SUB) && step / SUB < COLS {
            u_col[step / SUB] = u;
        }
        if phi > PHI_STOP {
            break;
        }
    }
    let end = end_phi.unwrap_or(PHI_STOP);
    // Columns past the end: escaped (0) or swallowed (1/2).
    let first_after = (step / SUB) + 1;
    for v in u_col.iter_mut().skip(first_after) {
        *v = if captured { U_HORIZON } else { 0.0 };
    }
    BendRow {
        b,
        captured,
        end_phi: end,
        turn: if captured { 0.0 } else { end - m::PI },
        closest_u: if captured {
            U_HORIZON
        } else {
            turning_point(b)
        },
        u: u_col,
    }
}

impl BendTable {
    /// Work the table out (about two million small steps; a few tens of
    /// milliseconds natively). Done once and cached by its users.
    pub fn compute() -> BendTable {
        BendTable {
            rows: (0..ROWS).map(|i| trace(b_of_row(i))).collect(),
        }
    }

    /// The table's content hash: its layout and every number's bits.
    pub fn hash(&self) -> ContentHash {
        let mut h = Hasher::new(DOMAIN);
        for n in [ROWS, CAPTURED_ROWS, COLS, SUB] {
            h.update(&(n as u64).to_le_bytes());
        }
        h.update(&B_MAX.to_bits().to_le_bytes());
        h.update(&PHI_MAX.to_bits().to_le_bytes());
        for r in &self.rows {
            h.update(&r.b.to_bits().to_le_bytes());
            h.update(&[u8::from(r.captured)]);
            for x in [r.end_phi, r.turn, r.closest_u] {
                h.update(&x.to_bits().to_le_bytes());
            }
            for x in &r.u {
                h.update(&x.to_bits().to_le_bytes());
            }
        }
        h.finish()
    }

    /// u = r_g / r on a ray of impact parameter `b` at angle `phi` swept,
    /// read off the table (linear between rows on the same side of b_c and
    /// between columns). `None` outside the table.
    pub fn u_at(&self, b: f64, phi: f64) -> Option<f64> {
        if !(0.0..=PHI_MAX).contains(&phi) {
            return None;
        }
        let at = row_of_b(b)?;
        let i0 = m::floor(at) as usize;
        let i1 = if i0 + 1 < ROWS && (i0 + 1 < CAPTURED_ROWS) == (i0 < CAPTURED_ROWS) {
            i0 + 1
        } else {
            i0
        };
        let fr = at - i0 as f64;
        let ck = phi / PHI_MAX * (COLS - 1) as f64;
        let k0 = (m::floor(ck) as usize).min(COLS - 2);
        let fc = ck - k0 as f64;
        let cell = |i: usize| {
            let r = &self.rows[i].u;
            r[k0] + (r[k0 + 1] - r[k0]) * fc
        };
        Some(cell(i0) + (cell(i1) - cell(i0)) * fr)
    }

    /// The table as one flat list for a float texture: per row, the four
    /// row facts (b, captured as 0 or 1, end angle, turn) then the cells.
    /// Row length `COLS + 4`.
    pub fn flat(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(ROWS * (COLS + 4));
        for r in &self.rows {
            out.extend([r.b, f64::from(u8::from(r.captured)), r.end_phi, r.turn]);
            out.extend(&r.u);
        }
        out
    }
}
