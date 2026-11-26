//! XO2: the feed, matter falling onto anything through a disc (exotic
//! objects design 2.2). One descriptor on any body that is gaining matter;
//! it never asks what kind of body it is.
//!
//! - **Where the matter comes from**: a stated rate, a companion's overflow
//!   (with the companion's pull cutting the disc off,
//!   `sse.feed.disc_outer_edge`), or gas swept up from the surroundings
//!   (`sse.feed.capture_from_surroundings`).
//! - **Where the disc ends inside**: the largest of the surface, the last
//!   stable orbit and the magnetic field's reach (`sse.feed.inner_edge`).
//! - **How much comes out as light** (`sse.feed.efficiency`) and the
//!   Eddington rate it is measured against (`sse.feed.eddington`).
//! - **Which flow the gas takes**: thin, hot and thick strengths from the
//!   Eddington share (`sse.feed.flow_strengths`), blended, not classes.
//!
//! The maths is in those formula blocks (`packs/base-physics/formulas/feed.json`)
//! and the gravity blocks; this module only wires them and assembles the
//! light: the thin share as the Shakura and Sunyaev disc (`Part::Disc`, or
//! traced from a stated tilt by [`crate::disc_image`]), the hot share as a
//! glow of stated slope (`Part::PowerLaw`, a logged stand-in).

use crate::*;
use serde::{Deserialize, Serialize};
use sse_flow::Atom;

/// The formula blocks the feed runs, in order.
pub const BLOCKS: &[&str] = &[
    "sse.gravity.gravitational_radius",
    "sse.gravity.horizon",
    "sse.gravity.last_stable_orbit",
    "sse.feed.capture_from_surroundings",
    "sse.feed.disc_outer_edge",
    "sse.feed.inner_edge",
    "sse.feed.efficiency",
    "sse.feed.eddington",
    "sse.feed.flow_strengths",
];

/// The hot flow's glow: from hard X-rays (0.01 nm) to radio at 1 cm, equal
/// power in every decade (F_ν ∝ ν⁻¹). Stand-in, design section 9.
pub const HOT_FROM_M: f64 = 1e-11;
pub const HOT_TO_M: f64 = 1e-2;
pub const HOT_ALPHA: f64 = 1.0;
pub const HOT_STAND_IN: &str = "the hot thin flow is drawn as a glow of stated slope (equal power per decade from 0.01 nm X-rays to 1 cm radio), not a computed gas (fudge register: exotic objects, hot flow)";
pub const THICK_STAND_IN: &str = "the thick flow's light is drawn as the thin disc's until a slim-disc block lands (fudge register: exotic objects, thick flow)";
pub const NEWTONIAN_DISC: &str = "the thin disc's brightness follows the Newtonian law of Shakura and Sunyaev, not the relativistic one of Novikov and Thorne (fudge register: exotic objects)";

/// A block from the physics in force, run on numbers. Its settings are
/// still the shipped ones (each parameter's stated default): no stack can
/// change them until ER2's `physics::run` lands, which replaces this.
/// Outside a kernel call nothing is in force, and the run says so.
pub(crate) fn run_block(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    let physics = sse_physics::now()
        .ok_or_else(|| format!("{id}: no physics in force (run inside a kernel call)"))?;
    let a = physics
        .blocks()
        .iter()
        .find(|a| a.decl().id == id)
        .ok_or_else(|| format!("formula block {id} is missing"))?;
    let params: Vec<f64> = a.decl().params.iter().map(shipped_setting).collect();
    a.eval(inputs, &params)
        .map_err(|e| format!("{id}: {}", e.0))
}

/// A block parameter as shipped (ER2 gives the stack's).
fn shipped_setting(p: &sse_flow::Param) -> f64 {
    p.default
}

/// Where the matter comes from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum FeedSource {
    /// A stated rate, with a stated outer edge (the reason in `why`).
    Given {
        rate_kg_s: f64,
        outer_m: f64,
        why: String,
    },
    /// A companion overflowing onto the body: its rate, the separation and
    /// its mass (the disc's outer edge follows from those).
    Companion {
        rate_kg_s: f64,
        separation_m: f64,
        donor_mass_kg: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        donor: Option<String>,
    },
    /// Gas swept up from the surroundings.
    Surroundings {
        gas_density_kg_m3: f64,
        sound_speed_m_s: f64,
        #[serde(default)]
        relative_speed_m_s: f64,
    },
}

/// A body that is (or may be) gaining matter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeedAsk {
    pub mass_kg: f64,
    /// Its surface; 0 for a black hole (the horizon stands in).
    #[serde(default)]
    pub radius_m: f64,
    /// Black hole spin a, −1 to 1; 0 until XO7 draws it.
    #[serde(default)]
    pub spin: f64,
    #[serde(default)]
    pub magnetic_moment_a_m2: f64,
    pub source: FeedSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowKind {
    Thin,
    Hot,
    Thick,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    pub kind: FlowKind,
    pub strength: f64,
}

/// What set the disc's inner edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InnerSetBy {
    Surface,
    LastStableOrbit,
    MagneticField,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscEdges {
    pub inner_m: f64,
    pub inner_set_by: InnerSetBy,
    pub outer_m: f64,
    pub magnetospheric_m: f64,
    /// The disc's plane is the body's equator (a tilted feed is later, XO16).
    pub plane: String,
}

/// The feed descriptor (design 3.5's `feed`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Feed {
    pub rate_kg_s: f64,
    pub eddington_rate_kg_s: f64,
    pub eddington_luminosity_w: f64,
    pub eddington_share: f64,
    pub efficiency: f64,
    /// Which block supplied the matter.
    pub source: String,
    pub flows: Vec<Flow>,
    /// How well the hot flow shines against a thin disc fed as fast.
    pub hot_lit_share: f64,
    pub disc: DiscEdges,
    /// Light given off by the feed, W: thin and thick share at the
    /// disc's own law, hot share at its reduced efficiency.
    pub luminosity_w: f64,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub blocks: Vec<String>,
}

/// Sizes every compact body has (gravity blocks), in metres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sizes {
    pub gravitational_radius_m: f64,
    pub horizon_m: f64,
    pub last_stable_orbit_m: f64,
}

pub fn sizes(mass_kg: f64, spin: f64) -> Result<Sizes, String> {
    let rg = run_block("sse.gravity.gravitational_radius", &[mass_kg])?[0];
    let horizon = run_block("sse.gravity.horizon", &[rg, spin.abs()])?[0];
    let isco = run_block("sse.gravity.last_stable_orbit", &[rg, spin])?[0];
    Ok(Sizes {
        gravitational_radius_m: rg,
        horizon_m: horizon,
        last_stable_orbit_m: isco,
    })
}

/// Work out the feed for a body.
pub fn describe(ask: &FeedAsk) -> Result<Feed, String> {
    if ask.mass_kg <= 0.0 {
        return Err("a feed needs a mass above zero".into());
    }
    let s = sizes(ask.mass_kg, ask.spin)?;
    let rg = s.gravitational_radius_m;
    let surface = if ask.radius_m > 0.0 {
        ask.radius_m
    } else {
        s.horizon_m
    };
    let mut why = vec![];
    let mut stand_ins = vec![];
    let mut blocks: Vec<String> = vec![
        "sse.gravity.gravitational_radius".into(),
        "sse.gravity.horizon".into(),
        "sse.gravity.last_stable_orbit".into(),
    ];
    // A body with a surface outside its last stable orbit has no use for
    // the orbit (it is inside the body); the surface wins in the block.
    let (rate, outer, source) = match &ask.source {
        FeedSource::Given {
            rate_kg_s,
            outer_m,
            why: w,
        } => {
            why.push(w.clone());
            (*rate_kg_s, *outer_m, "given".to_string())
        }
        FeedSource::Companion {
            rate_kg_s,
            separation_m,
            donor_mass_kg,
            donor,
        } => {
            let q = donor_mass_kg / ask.mass_kg;
            let rout = run_block("sse.feed.disc_outer_edge", &[*separation_m, q])?[0];
            blocks.push("sse.feed.disc_outer_edge".into());
            why.push(format!(
                "Fed by {} at {rate_kg_s:.3e} kg/s across {separation_m:.3e} m; the companion's pull cuts the disc off at {rout:.3e} m.",
                donor.as_deref().unwrap_or("a companion")
            ));
            (*rate_kg_s, rout, "sse.feed.disc_outer_edge".to_string())
        }
        FeedSource::Surroundings {
            gas_density_kg_m3,
            sound_speed_m_s,
            relative_speed_m_s,
        } => {
            let out = run_block(
                "sse.feed.capture_from_surroundings",
                &[
                    ask.mass_kg,
                    *gas_density_kg_m3,
                    *sound_speed_m_s,
                    *relative_speed_m_s,
                ],
            )?;
            blocks.push("sse.feed.capture_from_surroundings".into());
            why.push(format!(
                "Sweeps up gas of {gas_density_kg_m3:.2e} kg/m³ from within {:.3e} m at {:.3e} kg/s (Bondi capture: an upper limit, most of it is blown out again before it reaches the middle).",
                out[1], out[0]
            ));
            (
                out[0],
                out[1],
                "sse.feed.capture_from_surroundings".to_string(),
            )
        }
    };
    let edge = run_block(
        "sse.feed.inner_edge",
        &[
            ask.mass_kg,
            surface,
            s.last_stable_orbit_m,
            rate,
            ask.magnetic_moment_a_m2,
        ],
    )?;
    blocks.push("sse.feed.inner_edge".into());
    let (rm, rin) = (edge[0], edge[1]);
    let inner_set_by = if rin == rm && rm > 0.0 {
        InnerSetBy::MagneticField
    } else if surface >= s.last_stable_orbit_m {
        InnerSetBy::Surface
    } else {
        InnerSetBy::LastStableOrbit
    };
    let eta = run_block("sse.feed.efficiency", &[rg, rin])?[0];
    blocks.push("sse.feed.efficiency".into());
    let edd = run_block("sse.feed.eddington", &[ask.mass_kg, eta])?;
    blocks.push("sse.feed.eddington".into());
    let (l_edd, mdot_edd) = (edd[0], edd[1]);
    let share = if mdot_edd > 0.0 { rate / mdot_edd } else { 0.0 };
    let f = run_block("sse.feed.flow_strengths", &[share])?;
    blocks.push("sse.feed.flow_strengths".into());
    let (thin, hot, thick, lit) = (f[0], f[1], f[2], f[3]);
    let outer = outer.max(rin);
    let disc_power = (thin + thick) * rate * eta * C * C;
    let hot_power = hot * rate * eta * C * C * lit;
    why.push(format!(
        "The disc ends inside at {rin:.3e} m ({}), {:.1} gravitational radii; {:.2}% of the infalling mass comes out as light.",
        match inner_set_by {
            InnerSetBy::Surface => "the surface",
            InnerSetBy::LastStableOrbit => "the last stable orbit",
            InnerSetBy::MagneticField => "where the magnetic field takes over",
        },
        rin / rg,
        eta * 100.0
    ));
    why.push(format!(
        "Fed at {share:.2e} of the Eddington rate ({mdot_edd:.3e} kg/s): thin disc {thin:.2}, hot flow {hot:.2}, thick disc {thick:.2}."
    ));
    if hot > 0.01 {
        stand_ins.push(HOT_STAND_IN.into());
    }
    if thick > 0.01 {
        stand_ins.push(THICK_STAND_IN.into());
    }
    if thin + thick > 0.01 {
        stand_ins.push(NEWTONIAN_DISC.into());
    }
    if ask.spin != 0.0 {
        stand_ins.push("the hole's spin moves the inner edge and the efficiency here, but its light paths are drawn with the no-spin table until XO7".into());
    }
    Ok(Feed {
        rate_kg_s: rate,
        eddington_rate_kg_s: mdot_edd,
        eddington_luminosity_w: l_edd,
        eddington_share: share,
        efficiency: eta,
        source,
        flows: vec![
            Flow {
                kind: FlowKind::Thin,
                strength: thin,
            },
            Flow {
                kind: FlowKind::Hot,
                strength: hot,
            },
            Flow {
                kind: FlowKind::Thick,
                strength: thick,
            },
        ],
        hot_lit_share: lit,
        disc: DiscEdges {
            inner_m: rin,
            inner_set_by,
            outer_m: outer,
            magnetospheric_m: rm,
            plane: "equator".into(),
        },
        luminosity_w: disc_power + hot_power,
        why,
        stand_ins,
        blocks,
    })
}

impl Feed {
    /// The rate the thin disc carries (thin and thick shares).
    pub fn disc_rate_kg_s(&self) -> f64 {
        let s = |k| {
            self.flows
                .iter()
                .find(|f| f.kind == k)
                .map_or(0.0, |f| f.strength)
        };
        (s(FlowKind::Thin) + s(FlowKind::Thick)) * self.rate_kg_s
    }

    /// The hot flow's glow, W.
    pub fn hot_power_w(&self) -> f64 {
        let hot = self
            .flows
            .iter()
            .find(|f| f.kind == FlowKind::Hot)
            .map_or(0.0, |f| f.strength);
        hot * self.rate_kg_s * self.efficiency * C * C * self.hot_lit_share
    }
}
