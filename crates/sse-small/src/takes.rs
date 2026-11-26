//! What a designed body would take (RW16a, "implausible is not a limit"):
//! the small-body designer never refuses a pin, it says what holding it
//! together would need, against what real material holds. Steer, never stop.
//!
//! Two closed-form stresses, both order-of-magnitude (fudge SB-takes):
//! - spin: a uniform sphere turning faster than gravity alone holds needs a
//!   tensile stress across its middle of about rho R^2 (w^2 - 4 pi G rho / 3) / 2,
//!   the excess outward pull integrated from the centre to the surface;
//! - shape: a body big enough to pull itself round presses its centre at
//!   P = (2 pi / 3) G rho^2 R^2 (a uniform sphere); keeping it lumpy needs a
//!   material stronger than that.
//!
//! The material figures they are set against are the data sheet's strengths
//! of the formation rock and ice (fudge SB-strength-sheet: recalled values).

use crate::{Member, Settings};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const G: f64 = sse_core::sheet::G;

/// One thing a body's design asks of its material.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Take {
    /// "spin" or "shape".
    pub about: String,
    /// In words a GM reads.
    pub says: String,
    /// The stress the design needs, Pa.
    pub needs_pa: f64,
    /// What the body's own material holds, Pa (its structure's figure).
    pub holds_pa: f64,
}

fn pa_words(p: f64) -> String {
    if p >= 1e9 {
        format!("{:.1} GPa", p / 1e9)
    } else if p >= 1e6 {
        format!("{:.1} MPa", p / 1e6)
    } else if p >= 1e3 {
        format!("{:.1} kPa", p / 1e3)
    } else {
        format!("{:.0} Pa", p)
    }
}

/// What a resolved or designed body takes to hold together as it is: empty
/// when gravity and its own material are enough.
pub fn takes(b: &Member, s: &Settings) -> Vec<Take> {
    let st = &s.strength;
    let rho = b.bulk_density_kg_m3.max(100.0);
    let r = 0.5 * b.diameter_km * 1000.0;
    let icy = b.makeup.ice > 0.5;
    let mut out = vec![];
    // Spin.
    let w = 2.0 * m::PI / (b.spin_hours.max(1e-6) * 3600.0);
    let wc2 = 4.0 / 3.0 * m::PI * G * rho;
    let spin_needs = 0.5 * rho * r * r * (w * w - wc2);
    let (holds, what) = match b.structure.kind.as_str() {
        "rubble-pile" | "ring-aggregate" => (st.regolith_cohesion_pa, "loose rubble clings with"),
        "relaxed" => (0.0, "a body held by gravity alone has"),
        _ if icy => (st.ice_tensile_pa, "solid ice holds"),
        _ => (st.rock_tensile_pa, "solid rock holds"),
    };
    if spin_needs > 0.0 {
        let says = if spin_needs <= holds {
            format!(
                "It turns faster than gravity alone could hold it (once in {:.2} hours), but its material is enough: it needs {} and {} about {}.",
                b.spin_hours,
                pa_words(spin_needs),
                what,
                pa_words(holds)
            )
        } else {
            format!(
                "Turning once in {:.2} hours it would fly apart: holding it together needs {} across its middle, and {} only about {}. Something must bind it: a tether, a shell, or material far stronger than nature makes.",
                b.spin_hours,
                pa_words(spin_needs),
                what,
                pa_words(holds)
            )
        };
        out.push(Take {
            about: "spin".into(),
            says,
            needs_pa: spin_needs,
            holds_pa: holds,
        });
    }
    // Shape: lumpy above the round size.
    let round = crate::round_km(&b.makeup, s);
    if b.structure.kind != "relaxed" && b.diameter_km > round {
        let p = 2.0 / 3.0 * m::PI * G * rho * rho * r * r;
        let holds = if icy {
            st.ice_compressive_pa
        } else {
            st.rock_compressive_pa
        };
        out.push(Take {
            about: "shape".into(),
            says: format!(
                "At {:.0} km it is past the size where a body of this makeup pulls itself round (about {:.0} km). Its own weight presses its centre at {}; {} gives way at about {}, so over time it would slump round unless something stronger holds its shape.",
                b.diameter_km,
                round,
                pa_words(p),
                if icy { "ice" } else { "rock" },
                pa_words(holds)
            ),
            needs_pa: p,
            holds_pa: holds,
        });
    }
    if b.structure.kind == "relaxed" && b.diameter_km < 0.5 * round {
        out.push(Take {
            about: "shape".into(),
            says: format!(
                "At {:.1} km it is far too small for its own gravity to pull it round (that starts near {:.0} km for this makeup): a round body this size was melted and cooled in free fall, or shaped by hand.",
                b.diameter_km, round
            ),
            needs_pa: 0.0,
            holds_pa: 0.0,
        });
    }
    out
}
