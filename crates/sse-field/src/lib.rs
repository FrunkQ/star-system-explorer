//! Track MF6 part 1 (magnetic fields design 6.1 and 6.6): what stops the
//! wind at a body and how its conducting layers answer a changing field.
//!
//! Every body gets three possible obstacles, each computed: its own field
//! (a stand-off from pressure balance), its air pushed against by the wind
//! (an induced boundary), and its bare surface. The outermost wins. A moon
//! inside its host's bubble feels the host's field and plasma instead of the
//! wind, and a moon near the bubble's edge can cross in and out as the wind
//! gusts. Bodies are described by conditions, never by kind.
//!
//! The field itself is handed in (MF2 will publish it from each world's
//! interior); so is the air. Field history (`read_field_history`) waits on MF3.

use serde::{Deserialize, Serialize};
use sse_atoms::field as fb;
use sse_atoms::light::{magnetopause_radius, MU0};
use sse_core::math as m;
use sse_flow::Reference;

/// The stellar wind at a body (or at its host).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wind {
    pub density_kg_m3: f64,
    pub speed_m_s: f64,
    /// North-south field in the wind, T (southward negative); 0 if unknown.
    #[serde(default)]
    pub bz_t: f64,
}

impl Wind {
    pub fn ram(&self) -> f64 {
        self.density_kg_m3 * self.speed_m_s * self.speed_m_s
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Air {
    pub surface_pressure_pa: f64,
    /// Scale height of the upper air, m.
    pub scale_height_m: f64,
}

/// A conducting layer: a salty ocean, a magma ocean, an iron core.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub conductivity_s_m: f64,
    pub thickness_m: f64,
    pub outer_radius_m: f64,
}

/// The host a moon orbits, for moons that may sit inside its bubble.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Host {
    pub address: String,
    pub radius_m: f64,
    pub moment_a_m2: f64,
    /// The moon's orbital distance, m.
    pub orbit_m: f64,
    /// Pressure of the host's trapped plasma flowing past the moon, Pa.
    #[serde(default)]
    pub plasma_pa: f64,
    /// Swing of the field the moon sees: amplitude, T, and period, s (a
    /// tilted host field turning past the moon). Drives induction.
    #[serde(default)]
    pub varying_field_t: f64,
    #[serde(default)]
    pub varying_period_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub address: String,
    pub radius_m: f64,
    /// Dipole moment, A m² (0 for none).
    #[serde(default)]
    pub moment_a_m2: f64,
    #[serde(default)]
    pub air: Option<Air>,
    /// The star's wind here (for a moon: at its host).
    pub wind: Wind,
    #[serde(default)]
    pub host: Option<Host>,
    #[serde(default)]
    pub layers: Vec<Layer>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Air pressure at the induced boundary over the wind's ram pressure.
    pub ionopause_k: f64,
    /// The wind's gusts: ram pressure from `gust[0]` to `gust[1]` times its
    /// mean (the Sun's wind varies by about this much).
    pub gust: [f64; 2],
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            ionopause_k: 1.0,
            gust: [0.25, 4.0],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Obstacle {
    /// "field", "air" or "surface".
    pub kind: String,
    pub standoff_m: f64,
    pub standoff_radii: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Induced {
    pub layer: String,
    pub amplitude: f64,
    pub skin_depth_m: f64,
    /// Peak induced dipole moment, A m².
    pub moment_a_m2: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostBubble {
    pub host: String,
    /// "inside", "outside" or "crosses" (in and out as the wind gusts).
    pub place: String,
    pub host_standoff_m: [f64; 3],
    pub host_field_t: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldRead {
    pub address: String,
    /// What pushes on the body: "wind" or "host field".
    pub pressed_by: String,
    pub outside_pressure_pa: f64,
    pub candidates: Vec<Obstacle>,
    pub obstacle: Obstacle,
    /// Shue flaring exponent (only against a wind).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flaring: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_bubble: Option<HostBubble>,
    pub induced: Vec<Induced>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn refs_of(ids: &[&str]) -> Vec<Reference> {
    let mut out: Vec<Reference> = vec![];
    for a in sse_atoms::field::atoms()
        .iter()
        .chain(sse_atoms::light::atoms().iter())
    {
        let d = a.decl();
        if ids.contains(&d.id.as_str()) {
            for r in &d.references {
                if !out.contains(r) {
                    out.push(r.clone());
                }
            }
        }
    }
    out
}

fn radii(x: f64, r: f64) -> String {
    format!("{:.1} of its radii", x / r)
}

/// What stops the wind at a body, the bubble's shape and its layers' answer.
pub fn read(b: &Body, set: &Settings) -> Result<FieldRead, String> {
    if b.radius_m.is_nan()
        || b.radius_m <= 0.0
        || b.wind.density_kg_m3 < 0.0
        || b.wind.speed_m_s < 0.0
    {
        return Err(format!(
            "{}: radius must be positive and the wind not negative",
            b.address
        ));
    }
    let mut why = vec![];
    let mut stand_ins = vec![];
    let mut used = vec!["sse.light.magnetopause_radius"];
    // Where is it: in the wind, or inside a host's bubble?
    let mut host_bubble = None;
    let mut inside = false;
    let mut outside_b = 0.0;
    let mut plasma = 0.0;
    if let Some(h) = &b.host {
        let at = |gust: f64| {
            let w = Wind {
                density_kg_m3: b.wind.density_kg_m3 * gust,
                ..b.wind
            };
            magnetopause_radius(h.moment_a_m2, w.density_kg_m3, w.speed_m_s).max(h.radius_m)
        };
        let s = [at(set.gust[1]), at(1.0), at(set.gust[0])];
        let place = if h.orbit_m < s[0] {
            "inside"
        } else if h.orbit_m > s[2] {
            "outside"
        } else {
            "crosses"
        };
        let bf = MU0 * h.moment_a_m2 / (4.0 * m::PI * h.orbit_m * h.orbit_m * h.orbit_m);
        why.push(match place {
            "inside" => format!("It orbits inside {}'s magnetic bubble (edge {:.0} to {:.0} host radii as the wind gusts; the moon at {:.1}), so it feels the host's field ({:.0} nT) and plasma, not the star's wind.", h.address, s[0] / h.radius_m, s[2] / h.radius_m, h.orbit_m / h.radius_m, bf * 1e9),
            "crosses" => format!("Its orbit at {:.1} host radii lies where {}'s bubble edge swings between {:.0} and {:.0} as the wind gusts: it crosses in and out, in the wind on some days and in the host's field on others. Shown here for the mean wind.", h.orbit_m / h.radius_m, h.address, s[0] / h.radius_m, s[2] / h.radius_m),
            _ => format!("It orbits outside {}'s magnetic bubble (edge at most {:.0} host radii), in the star's wind.", h.address, s[2] / h.radius_m),
        });
        inside = place == "inside" || (place == "crosses" && h.orbit_m < s[1]);
        if inside {
            outside_b = bf;
            plasma = h.plasma_pa;
        }
        host_bubble = Some(HostBubble {
            host: h.address.clone(),
            place: place.into(),
            host_standoff_m: s,
            host_field_t: bf,
        });
    }
    let ram = b.wind.ram();
    let outside_pressure = if inside {
        outside_b * outside_b / (2.0 * MU0) + plasma
    } else {
        ram
    };
    let r = b.radius_m;
    let mut candidates = vec![Obstacle {
        kind: "surface".into(),
        standoff_m: r,
        standoff_radii: 1.0,
    }];
    // Own field.
    if b.moment_a_m2 > 0.0 {
        let s = if inside {
            used.push("sse.field.standoff_in_field");
            fb::standoff_in_field(b.moment_a_m2, outside_b, plasma)
        } else {
            magnetopause_radius(b.moment_a_m2, b.wind.density_kg_m3, b.wind.speed_m_s)
        };
        candidates.push(Obstacle {
            kind: "field".into(),
            standoff_m: s,
            standoff_radii: s / r,
        });
    }
    // Air.
    if let Some(a) = &b.air {
        used.push("sse.field.ionopause_altitude");
        let h = fb::ionopause_altitude(
            a.surface_pressure_pa,
            a.scale_height_m,
            outside_pressure,
            set.ionopause_k,
        );
        candidates.push(Obstacle {
            kind: "air".into(),
            standoff_m: r + h,
            standoff_radii: (r + h) / r,
        });
        stand_ins.push("the air's boundary is where an isothermal upper atmosphere's pressure meets the outside pressure; the real ionopause needs the ionosphere's own profile".into());
    }
    let obstacle = candidates.iter().fold(candidates[0].clone(), |best, c| {
        if c.standoff_m > best.standoff_m {
            c.clone()
        } else {
            best
        }
    });
    let pressed_by = if inside { "host field" } else { "wind" };
    why.push(match obstacle.kind.as_str() {
        "field" => format!("Its own magnetic field holds the {} off {} out from the centre.", pressed_by, radii(obstacle.standoff_m, r)),
        "air" => format!("It has no field strong enough, so the {} presses on the top of its air, {:.0} km up: an induced bubble draped round it.", pressed_by, (obstacle.standoff_m - r) / 1e3),
        _ => format!("Nothing holds the {} off: it strikes the bare surface.", pressed_by),
    });
    if b.moment_a_m2 > 0.0 && obstacle.kind != "field" {
        why.push("Its own field is too weak to stand above its air or surface here.".into());
    }
    let flaring = if inside || ram <= 0.0 {
        None
    } else {
        used.push("sse.field.shue_flaring");
        Some(fb::shue_alpha(ram, b.wind.bz_t))
    };
    // Induction.
    let mut induced = vec![];
    if let Some(h) = &b.host {
        if h.varying_field_t > 0.0 && h.varying_period_s > 0.0 {
            used.push("sse.induction.response");
            for l in &b.layers {
                let a = fb::induction_amplitude(
                    l.conductivity_s_m,
                    l.thickness_m,
                    l.outer_radius_m,
                    h.varying_period_s,
                );
                let delta = fb::skin_depth(
                    l.conductivity_s_m.max(1e-30),
                    2.0 * m::PI / h.varying_period_s,
                );
                // Induced dipole: B_ind at the pole = A B_vary / 2 ... moment = −A B 2π r³ / μ₀.
                let moment = a
                    * h.varying_field_t
                    * 2.0
                    * m::PI
                    * l.outer_radius_m
                    * l.outer_radius_m
                    * l.outer_radius_m
                    / MU0;
                why.push(format!("Its {} ({:.2} S/m, {:.0} km thick) answers the host's field swinging by {:.0} nT every {:.1} hours with a field {:.0} % of a perfect conductor's.", l.name, l.conductivity_s_m, l.thickness_m / 1e3, h.varying_field_t * 1e9, h.varying_period_s / 3600.0, a * 100.0));
                induced.push(Induced {
                    layer: l.name.clone(),
                    amplitude: a,
                    skin_depth_m: delta,
                    moment_a_m2: moment,
                });
            }
            stand_ins.push("induction uses the thin-shell response with the thickness capped at the skin depth; one frequency (the host's spin as seen by the moon)".into());
        }
    }
    Ok(FieldRead {
        address: b.address.clone(),
        pressed_by: pressed_by.into(),
        outside_pressure_pa: outside_pressure,
        candidates,
        obstacle,
        flaring,
        host_bubble,
        induced,
        why,
        stand_ins,
        references: refs_of(&used),
    })
}

#[cfg(test)]
mod tests;
