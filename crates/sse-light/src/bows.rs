//! Rainbows and halos from a refractive index (VP-L4, visual polish design
//! section 12). A bow's radius depends only on how strongly the falling
//! liquid bends light, so methane rain makes a wider bow than water; a
//! halo's on the crystal's index and the angle between the faces the light
//! crosses. Blocks `sse.light.rainbow` and `sse.light.halo`.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::Reference;

use crate::colour::run;

/// What something falling or floating in an air gives the sky.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bows {
    pub id: String,
    pub refractive_index: f64,
    /// "drop" (liquid rain: rainbows) or "crystal" (ice: halos).
    pub kind: String,
    /// Drops: the primary and secondary bow's radius round the point
    /// opposite the sun, degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_deg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_deg: Option<f64>,
    /// Crystals: halos round the sun through 60° and 90° faces, degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub halo_60_deg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub halo_90_deg: Option<f64>,
    pub references: Vec<Reference>,
}

fn deg(r: f64) -> f64 {
    r * 180.0 / m::PI
}

fn refs(id: &str) -> Vec<Reference> {
    crate::colour::block(id).decl().references.clone()
}

/// The bows or halos for one refractive index.
pub fn bows(id: &str, n: f64, kind: &str) -> Result<Bows, String> {
    let mut b = Bows {
        id: id.into(),
        refractive_index: n,
        kind: kind.into(),
        primary_deg: None,
        secondary_deg: None,
        halo_60_deg: None,
        halo_90_deg: None,
        references: vec![],
    };
    match kind {
        "drop" => {
            let o = run("sse.light.rainbow", &[n])?;
            b.primary_deg = Some(deg(o[0]));
            b.secondary_deg = Some(deg(o[1]));
            b.references = refs("sse.light.rainbow");
        }
        "crystal" => {
            // A hexagonal crystal's side faces meet at 60° (alternate sides)
            // and its end faces meet the sides at 90°.
            let x = n * m::sin(m::PI / 4.0);
            b.halo_60_deg = Some(deg(run("sse.light.halo", &[n, m::PI / 3.0])?[0]));
            if x < 1.0 {
                b.halo_90_deg = Some(deg(run("sse.light.halo", &[n, m::PI / 2.0])?[0]));
            }
            b.references = refs("sse.light.halo");
        }
        other => return Err(format!("{id}: kind {other}; kinds: drop, crystal")),
    }
    Ok(b)
}
