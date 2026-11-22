//! Constants with a history (DM6, galaxy histories design section 3.2,
//! "the bone"). Any constants row may carry a `law` beside its value:
//!
//! - `{"kind": "constant"}`, the default for every row;
//! - `{"kind": "drift", "per_yr": r, "anchor_gyr": t0}`: a steady drift,
//!   value(t) = v (1 + r (t - t0)), with t in years after the origin and v
//!   the value at `anchor_gyr`;
//! - `{"kind": "step", "at_gyr": t1, "before": b}`: the value is `b` before
//!   `at_gyr` and `v` from then on.
//!
//! A law is never real: the measured drift of G is zero to about a part in
//! 10^13 a year, so every non-constant law carries `mark` "authored" or
//! "fictional" and its own `src`. A law goes on a measured row, never on a
//! worked-out one: the worked-out rows follow their inputs at each date.

use serde_json::{Map, Value as J};

/// Years in a gigayear.
const GYR: f64 = 1.0e9;

/// One row's time law.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Law {
    Constant,
    Drift { per_yr: f64, anchor_gyr: f64 },
    Step { at_gyr: f64, before: f64 },
}

impl Law {
    /// The row's law, or why it cannot be read.
    pub fn of(row: &Map<String, J>) -> Result<Law, String> {
        let Some(l) = row.get("law") else {
            return Ok(Law::Constant);
        };
        let f = |k: &str| l.get(k).and_then(J::as_f64);
        let law = match l.get("kind").and_then(J::as_str) {
            Some("constant") => return Ok(Law::Constant),
            Some("drift") => Law::Drift {
                per_yr: f("per_yr").ok_or("a drift needs per_yr (a fraction per year)")?,
                anchor_gyr: f("anchor_gyr").ok_or(
                    "a drift needs anchor_gyr (the date its value holds, Gyr after the origin)",
                )?,
            },
            Some("step") => Law::Step {
                at_gyr: f("at_gyr").ok_or("a step needs at_gyr (Gyr after the origin)")?,
                before: f("before").ok_or("a step needs before (the value before the date)")?,
            },
            Some(k) => return Err(format!("law kind {k} is not constant, drift or step")),
            None => return Err("a law needs a kind (constant, drift or step)".into()),
        };
        match l.get("mark").and_then(J::as_str) {
            Some("authored" | "fictional") => {}
            _ => {
                return Err(
                    "a time law is never real (G's measured drift is zero to about a part in 10^13 a year): mark it authored or fictional"
                        .into(),
                )
            }
        }
        if l.get("src").and_then(J::as_str).is_none() {
            return Err("a time law needs its own src".into());
        }
        Ok(law)
    }

    /// The value at `t_gyr` of a row whose value is `v`.
    pub fn at(&self, v: f64, t_gyr: f64) -> f64 {
        match *self {
            Law::Constant => v,
            Law::Drift { per_yr, anchor_gyr } => v * (1.0 + per_yr * (t_gyr - anchor_gyr) * GYR),
            Law::Step { at_gyr, before } => {
                if t_gyr < at_gyr {
                    before
                } else {
                    v
                }
            }
        }
    }
}
