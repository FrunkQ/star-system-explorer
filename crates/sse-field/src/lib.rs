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
use sse_atoms::aurora as au;
use sse_atoms::field as fb;
use sse_atoms::light::{magnetopause_radius, MU0};
use sse_atoms::shield as sh;
use sse_core::math as m;
use sse_core::quantity::consts::{DAY, JULIAN_YEAR};
use sse_flow::Reference;

mod belts;
pub mod glow;
mod sol;
pub use belts::{BeltDose, BeltGap, BeltSettings, BeltShell, Belts, ShieldData};
pub use glow::{AirLevel, Emission, GasShare, Glow, UpperAir};
pub use sol::{solar_system, MeasuredField, SolField, WindField};
pub use sse_atoms::belts::Electron;

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
    /// Surface gravity, m/s²: turns the air's pressure into its column.
    /// Without it a body with air gets no dose.
    #[serde(default)]
    pub surface_gravity_m_s2: f64,
    /// Cosmic-ray dose in open space at this body's orbit, mSv a day; left
    /// out, the settings' value (measured near 1 AU) is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_space_dose_msv_day: Option<f64>,
    /// Orbits (distance from the centre, m) to give a dose for.
    #[serde(default)]
    pub orbits_m: Vec<f64>,
    /// Angle between the magnetic and spin axes, degrees.
    #[serde(default)]
    pub magnetic_tilt_deg: f64,
    /// Plasma added each second by moons' volcanoes or geysers, kg/s (Io
    /// gives Jupiter about a tonne), and the ionosphere's effective
    /// Pedersen conductance of one hemisphere, S. Both make the spin-driven oval.
    #[serde(default)]
    pub plasma_source_kg_s: f64,
    #[serde(default)]
    pub pedersen_conductance_s: f64,
    /// Moons whose field lines may land as aurora spots and that sweep
    /// gaps in the belts.
    #[serde(default)]
    pub moons: Vec<MoonOrbit>,
    /// Time for one turn, s (0 if unknown): the spin-driven oval's power.
    #[serde(default)]
    pub spin_period_s: f64,
    /// The high air where aurorae glow: its gases and density profile.
    /// Without it an oval has a place and a power but no lines.
    #[serde(default)]
    pub upper_air: Option<UpperAir>,
    /// Outer edge of rings dense enough to absorb trapped particles, m (0: none).
    #[serde(default)]
    pub rings_outer_m: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoonOrbit {
    pub address: String,
    pub orbit_m: f64,
    #[serde(default)]
    pub radius_m: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Air pressure at the induced boundary over the wind's ram pressure.
    pub ionopause_k: f64,
    /// The wind's gusts: ram pressure from `gust[0]` to `gust[1]` times its
    /// mean (the Sun's wind varies by about this much).
    pub gust: [f64; 2],
    /// Cosmic-ray dose in open space, mSv a day, when a body gives none
    /// (MSL in cruise near 1 AU, Zeitlin et al. 2013).
    pub free_space_dose_msv_day: f64,
    /// `sse.shield.space_share`: turnover rigidity, GV, and index.
    pub space_r0_gv: f64,
    pub space_gamma: f64,
    /// `sse.shield.ground_share`: Dorman α and k.
    pub dorman_alpha: f64,
    pub dorman_k: f64,
    /// `sse.shield.air_dose`: attenuation lengths, kg/m², and muon share.
    pub lambda_n_kg_m2: f64,
    pub lambda_mu_kg_m2: f64,
    pub muon_share: f64,
    /// MF7 part 2: radiation belts.
    pub belts: BeltSettings,
    /// MF9 part 2b: the energy of the electrons each driver rains in.
    pub aurora: AuroraSettings,
    /// Electron mass, charge and ε₀, read from the master data sheet by the
    /// caller; without them no belts are computed.
    #[serde(skip)]
    pub electron: Option<Electron>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            ionopause_k: 1.0,
            gust: [0.25, 4.0],
            free_space_dose_msv_day: 1.84,
            space_r0_gv: 2.0,
            space_gamma: 1.7,
            dorman_alpha: 10.0,
            dorman_k: 1.0,
            lambda_n_kg_m2: 1400.0,
            lambda_mu_kg_m2: 6000.0,
            muon_share: 0.0045,
            belts: BeltSettings::default(),
            aurora: AuroraSettings::default(),
            electron: None,
        }
    }
}

/// The falling electrons' energy, eV, for each driver of aurorae.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AuroraSettings {
    /// A wind-driven arc: a few to about ten keV (Evans 1974).
    pub wind_electron_ev: f64,
    /// A giant's spin-driven main oval: tens of keV (Gustin et al. 2004).
    pub spin_electron_ev: f64,
}

impl Default for AuroraSettings {
    fn default() -> Self {
        AuroraSettings {
            wind_electron_ev: 8.0e3,
            spin_electron_ev: 5.0e4,
        }
    }
}

/// Cut-off and dose at one magnetic latitude on the ground.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LatitudeShield {
    pub magnetic_latitude_deg: f64,
    pub cutoff_gv: f64,
    pub surface_dose_msv_yr: f64,
}

/// Dose in a circular orbit, unshielded by any hull.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrbitDose {
    pub distance_m: f64,
    pub cutoff_equator_gv: f64,
    pub equator_msv_yr: f64,
    pub pole_msv_yr: f64,
    /// Averaged over every direction of the orbit's plane (a sphere).
    pub mean_msv_yr: f64,
}

/// An auroral oval: where one driver's currents come down into the air.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Oval {
    /// "wind" (open and closed field lines meet) or "spin" (corotation fails).
    pub driver: String,
    /// Where its field lines cross the magnetic equator, m.
    pub maps_to_m: f64,
    /// Magnetic latitude of the oval, degrees, for the mean wind.
    pub magnetic_latitude_deg: f64,
    /// For a wind oval: from a gusty (strong) to a quiet wind, degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude_range_deg: Option<[f64; 2]>,
    /// Nearest and furthest the oval comes to the spin equator, degrees of
    /// spin latitude (the magnetic axis is tilted).
    pub spin_latitude_range_deg: [f64; 2],
    /// Magnetic latitudes of its equatorward and poleward edges, degrees.
    pub edges_deg: [f64; 2],
    /// Full width, degrees of latitude.
    pub width_deg: f64,
    /// Its power, lines, heights and colour (absent without a power).
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub glow: Option<Glow>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Footprint {
    pub moon: String,
    pub magnetic_latitude_deg: f64,
}

/// Where aurorae glow: ovals and moon spots, or a diffuse glow over the
/// night side for a world with air and no field holding the wind off.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aurora {
    pub ovals: Vec<Oval>,
    pub footprints: Vec<Footprint>,
    pub diffuse: bool,
    /// The diffuse glow's power, lines, heights and colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diffuse_glow: Option<Glow>,
}

/// Cosmic-ray shielding: what the field turns aside and the air absorbs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shield {
    pub free_space_msv_yr: f64,
    pub column_kg_m2: f64,
    /// Equatorial cut-off of the host's field at a moon's orbit, GV (0 if none).
    pub host_cutoff_gv: f64,
    pub by_latitude: Vec<LatitudeShield>,
    /// Averaged over the whole surface by area.
    pub surface_mean_msv_yr: f64,
    pub orbits: Vec<OrbitDose>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shield: Option<Shield>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aurora: Option<Aurora>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belts: Option<Belts>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

/// The field blocks in the order their references are listed.
const FIELD_BLOCKS: &[&str] = &[
    "sse.field.ionopause_altitude",
    "sse.field.shue_flaring",
    "sse.field.standoff_in_field",
    "sse.induction.response",
    "sse.light.effective_temperature",
    "sse.light.reflected_area",
    "sse.light.stellar_wind",
    "sse.light.magnetopause_radius",
    "sse.light.auroral_radio_power",
    "sse.light.cyclotron_frequency",
    "sse.shield.cutoff",
    "sse.shield.space_share",
    "sse.shield.ground_share",
    "sse.shield.air_dose",
    "sse.aurora.footprint",
    "sse.aurora.hill_distance",
    "sse.belts.kp_limit",
    "sse.belts.electron_range",
    "sse.belts.radiation_yield",
    "sse.glow.electron_range",
    "sse.glow.wind_power",
    "sse.glow.spin_power",
    "sse.glow.line_survival",
];

fn refs_of(ids: &[&str]) -> Vec<Reference> {
    let mut out: Vec<Reference> = vec![];
    for d in sse_atoms::decls_of(FIELD_BLOCKS) {
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
    // Cosmic rays: the field's cut-off, then the air.
    let own_reach = candidates
        .iter()
        .find(|c| c.kind == "field")
        .map(|c| c.standoff_m)
        .unwrap_or(0.0);
    let host_cut = match (&b.host, inside) {
        (Some(h), true) => sh::cutoff_rigidity(h.moment_a_m2, h.orbit_m, 0.0),
        _ => 0.0,
    };
    let shield = shield_of(b, set, own_reach, host_cut, &mut why, &mut stand_ins);
    if shield.is_some() {
        used.extend([
            "sse.shield.cutoff",
            "sse.shield.space_share",
            "sse.shield.ground_share",
            "sse.shield.air_dose",
        ]);
    }
    let aurora = aurora_of(
        b,
        set,
        &obstacle,
        inside,
        &mut why,
        &mut stand_ins,
        &mut used,
    )?;
    let belts = belts::belts_of(
        b,
        set,
        own_reach,
        set.electron.as_ref(),
        &mut why,
        &mut stand_ins,
        &mut used,
    );
    Ok(FieldRead {
        address: b.address.clone(),
        pressed_by: pressed_by.into(),
        outside_pressure_pa: outside_pressure,
        candidates,
        obstacle,
        flaring,
        host_bubble,
        induced,
        shield,
        aurora,
        belts,
        why,
        stand_ins,
        references: refs_of(&used),
    })
}

fn deg(rad: f64) -> f64 {
    rad * 180.0 / m::PI
}

/// The air base for an upper air with no profile: pressure, scale height
/// and gravity, from the upper air itself or the body's air.
fn glow_base(b: &Body) -> Option<(f64, f64, f64)> {
    let u = b.upper_air.as_ref()?;
    let p = u
        .base_pressure_pa
        .or(b.air.as_ref().map(|a| a.surface_pressure_pa))?;
    let h = u
        .scale_height_m
        .or(b.air.as_ref().map(|a| a.scale_height_m))?;
    let g = u.gravity_m_s2.unwrap_or(b.surface_gravity_m_s2);
    Some((p, h, g))
}

/// The lines, heights and colour for `power` coming down at `ev`, said in
/// `why`; none without an upper air.
fn shine(
    b: &Body,
    power: f64,
    ev: f64,
    what: &str,
    why: &mut Vec<String>,
    stand_ins: &mut Vec<String>,
) -> Result<Option<Glow>, String> {
    let Some(air) = &b.upper_air else {
        return Ok(None);
    };
    let g = glow::glow(power, ev, air, glow_base(b))?;
    let mut lines: Vec<&Emission> = g.emissions.iter().collect();
    lines.sort_by(|x, y| y.power_w.total_cmp(&x.power_w));
    let said: Vec<String> = lines
        .iter()
        .filter(|e| e.colour != [0.0; 3])
        .take(3)
        .map(|e| {
            format!(
                "{} at {:.0} km ({:.2} % of the power)",
                e.line,
                e.peak_height_m / 1e3,
                100.0 * e.share
            )
        })
        .collect();
    if said.is_empty() {
        stand_ins.push(format!("{what}: none of the air's gases gives off a line the light table knows in the eye's range"));
    } else {
        why.push(format!(
            "The {what} brings down {:.1e} W as {:.0} keV electrons, which stop {:.0} km up; it shines in {}{}.",
            power,
            ev / 1e3,
            g.stops_at_m / 1e3,
            said.join(", "),
            g.colour
                .as_ref()
                .map(|c| format!(", {} to the eye", c.hex))
                .unwrap_or_default()
        ));
    }
    Ok(Some(g))
}

/// Ovals from each driver that lands inside the bubble, moon spots, or a
/// diffuse glow, with their power, lines and colours where the air is known.
fn aurora_of(
    b: &Body,
    set: &Settings,
    obstacle: &Obstacle,
    inside: bool,
    why: &mut Vec<String>,
    stand_ins: &mut Vec<String>,
    used: &mut Vec<&'static str>,
) -> Result<Option<Aurora>, String> {
    let r = b.radius_m;
    let ev = set.aurora.wind_electron_ev;
    if obstacle.kind != "field" {
        if b.air.is_some() && obstacle.kind == "air" {
            why.push(format!("Aurora: with no field holding the {} off, it lights the top of the air as a faint diffuse glow over the night side, not a ring.", if inside { "host field" } else { "wind" }));
            let mut diffuse_glow = None;
            if !inside && b.wind.ram() > 0.0 {
                used.extend([
                    "sse.glow.wind_power",
                    "sse.glow.electron_range",
                    "sse.glow.line_survival",
                ]);
                let p = glow::run(
                    "sse.glow.wind_power",
                    &[b.wind.density_kg_m3, b.wind.speed_m_s, obstacle.standoff_m],
                )?[0];
                diffuse_glow = shine(b, p, ev, "diffuse glow", why, stand_ins)?;
            }
            return Ok(Some(Aurora {
                ovals: vec![],
                footprints: vec![],
                diffuse: true,
                diffuse_glow,
            }));
        }
        return Ok(None);
    }
    used.push("sse.aurora.footprint");
    let tilt = b.magnetic_tilt_deg.abs();
    // An oval at magnetic colatitude c round a pole tilted t from the spin
    // pole spans spin colatitudes |t − c| to t + c.
    let spin_range = |lat: f64| {
        let c = 90.0 - lat;
        [90.0 - (tilt + c), 90.0 - (tilt - c).abs()]
    };
    // The band of field lines an oval's currents run on: for the wind, the
    // closed lines from half the nose's distance out to the nose; for the
    // spin, a quarter either side of the corotation limit (F59).
    let edges = |inner: f64, outer: f64| {
        [
            deg(au::footprint_latitude(inner, r)),
            deg(au::footprint_latitude(outer, r)),
        ]
    };
    let mut ovals = vec![];
    let nose = obstacle.standoff_m;
    let lat = deg(au::footprint_latitude(nose, r));
    let mut range = None;
    if !inside && b.wind.ram() > 0.0 {
        let at = |gust: f64| {
            let s =
                magnetopause_radius(b.moment_a_m2, b.wind.density_kg_m3 * gust, b.wind.speed_m_s);
            deg(au::footprint_latitude(s, r))
        };
        range = Some([at(set.gust[1]), at(set.gust[0])]);
    }
    let we = edges(0.5 * nose, nose);
    why.push(match range {
        Some([lo, hi]) => format!("Aurora: the {} drives an oval at {:.0}° magnetic latitude, where the field line through the bubble's nose comes down; it moves from {:.0}° in a quiet wind to {:.0}° in a gusty one.", if inside { "host's flow" } else { "wind" }, lat, hi, lo),
        None => format!("Aurora: an oval at {:.0}° magnetic latitude, where the field line through the bubble's nose comes down.", lat),
    });
    let mut wind_glow = None;
    if !inside && b.wind.ram() > 0.0 {
        used.extend([
            "sse.glow.wind_power",
            "sse.glow.electron_range",
            "sse.glow.line_survival",
        ]);
        let p = glow::run(
            "sse.glow.wind_power",
            &[b.wind.density_kg_m3, b.wind.speed_m_s, nose],
        )?[0];
        wind_glow = Some(match shine(b, p, ev, "wind's oval", why, stand_ins)? {
            Some(g) => g,
            None => Glow {
                power_w: p,
                power_by_hemisphere: [p / 2.0, p / 2.0],
                electron_ev: ev,
                stops_at_m: 0.0,
                emissions: vec![],
                colour: None,
            },
        });
    }
    ovals.push(Oval {
        driver: "wind".into(),
        maps_to_m: nose,
        magnetic_latitude_deg: lat,
        latitude_range_deg: range,
        spin_latitude_range_deg: spin_range(lat),
        edges_deg: we,
        width_deg: we[1] - we[0],
        glow: wind_glow,
    });
    if b.plasma_source_kg_s > 0.0 && b.pedersen_conductance_s > 0.0 {
        used.push("sse.aurora.hill_distance");
        let b_eq = MU0 * b.moment_a_m2 / (4.0 * m::PI * r * r * r);
        let h = au::hill_distance(b.pedersen_conductance_s, b_eq, r, b.plasma_source_kg_s);
        if h > r && h < nose {
            let l = deg(au::footprint_latitude(h, r));
            why.push(format!("Its moons load {:.0} kg of gas a second; beyond {:.0} of its radii that gas lags the spinning field, and the currents that drag it round light a main oval at {:.0}°, inside the wind's.", b.plasma_source_kg_s, h / r, l));
            let se = edges((0.8 * h).max(r), (1.25 * h).min(nose));
            let mut spin_glow = None;
            if b.spin_period_s > 0.0 {
                used.extend([
                    "sse.glow.spin_power",
                    "sse.glow.electron_range",
                    "sse.glow.line_survival",
                ]);
                let p = glow::run(
                    "sse.glow.spin_power",
                    &[b.plasma_source_kg_s, b.spin_period_s, h],
                )?[0];
                let sev = set.aurora.spin_electron_ev;
                spin_glow = Some(
                    match shine(b, p, sev, "spin's main oval", why, stand_ins)? {
                        Some(g) => g,
                        None => Glow {
                            power_w: p,
                            power_by_hemisphere: [p / 2.0, p / 2.0],
                            electron_ev: sev,
                            stops_at_m: 0.0,
                            emissions: vec![],
                            colour: None,
                        },
                    },
                );
            } else {
                stand_ins.push(
                    "no spin period, so the spin-driven oval has a place but no power".into(),
                );
            }
            ovals.push(Oval {
                driver: "spin".into(),
                maps_to_m: h,
                magnetic_latitude_deg: l,
                latitude_range_deg: None,
                spin_latitude_range_deg: spin_range(l),
                edges_deg: se,
                width_deg: se[1] - se[0],
                glow: spin_glow,
            });
        }
    }
    if b.upper_air.is_none() {
        stand_ins.push("no upper air given (its gases and how it thins with height), so the ovals have a power but no lines, heights or colour".into());
    }
    if tilt > 0.0 {
        why.push(format!("Its magnetic axis is tilted {tilt:.0}° from its spin axis, so the ovals circle the magnetic poles, swinging between {:.0}° and {:.0}° of spin latitude.", ovals[0].spin_latitude_range_deg[0], ovals[0].spin_latitude_range_deg[1]));
    }
    let mut footprints = vec![];
    for mo in &b.moons {
        if mo.orbit_m > r && mo.orbit_m < nose {
            let l = deg(au::footprint_latitude(mo.orbit_m, r));
            why.push(format!("{}'s field line lands at {:.0}°: a spot and trail in the aurora that follows the moon.", mo.address, l));
            footprints.push(Footprint {
                moon: mo.address.clone(),
                magnetic_latitude_deg: l,
            });
        }
    }
    if !footprints.is_empty() {
        stand_ins.push("moon spots have a place but no power or lead angle yet".into());
    }
    Ok(Some(Aurora {
        ovals,
        footprints,
        diffuse: false,
        diffuse_glow: None,
    }))
}

fn msv_yr(sv_s: f64) -> f64 {
    sv_s * 1e3 * JULIAN_YEAR
}

/// Cut-off by latitude and the dose on the ground and in orbit. The own
/// field turns particles aside only inside its own bubble (`reach`); a moon
/// inside its host's bubble also has the host's cut-off at its orbit.
fn shield_of(
    b: &Body,
    set: &Settings,
    reach: f64,
    host_cut: f64,
    why: &mut Vec<String>,
    stand_ins: &mut Vec<String>,
) -> Option<Shield> {
    let column = match &b.air {
        Some(a) if b.surface_gravity_m_s2 > 0.0 => a.surface_pressure_pa / b.surface_gravity_m_s2,
        Some(_) => {
            why.push("No surface gravity was given, so the air's column and the dose under it are not computed.".into());
            return None;
        }
        None => 0.0,
    };
    let free = b
        .free_space_dose_msv_day
        .unwrap_or(set.free_space_dose_msv_day)
        .max(0.0)
        * 1e-3
        / DAY;
    let air = sh::AirDose {
        lambda_n: set.lambda_n_kg_m2,
        lambda_mu: set.lambda_mu_kg_m2,
        muon_share: set.muon_share,
    };
    let cut = |r: f64, lat: f64| {
        let own = if r <= reach {
            sh::cutoff_rigidity(b.moment_a_m2, r, lat)
        } else {
            0.0
        };
        own.max(host_cut)
    };
    let dose = |rc: f64, x: f64, sky: f64| {
        sh::air_dose(
            free,
            x,
            sh::space_share(rc, set.space_r0_gv, set.space_gamma),
            sh::ground_share(rc, set.dorman_alpha, set.dorman_k),
            sky,
            &air,
        )
    };
    let r = b.radius_m;
    let mut by_latitude = vec![];
    for d in [0.0, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0] {
        let rc = cut(r, d * m::PI / 180.0);
        by_latitude.push(LatitudeShield {
            magnetic_latitude_deg: d,
            cutoff_gv: rc * 1e-9,
            surface_dose_msv_yr: msv_yr(dose(rc, column, 0.5)),
        });
    }
    // Area mean: midpoints in latitude weighted by cos λ.
    let mean_at = |dist: f64, x: f64, sky: f64| {
        let n = 90;
        let (mut sum, mut wsum) = (0.0, 0.0);
        for i in 0..n {
            let lat = (i as f64 + 0.5) / n as f64 * m::PI / 2.0;
            let w = m::cos(lat);
            sum += w * dose(cut(dist, lat), x, sky);
            wsum += w;
        }
        sum / wsum
    };
    let surface_mean = msv_yr(mean_at(r, column, 0.5));
    let mut orbits = vec![];
    for &d in &b.orbits_m {
        if d <= r {
            continue;
        }
        let q = r / d;
        let sky = 1.0 - (1.0 - m::sqrt(1.0 - q * q)) / 2.0;
        orbits.push(OrbitDose {
            distance_m: d,
            cutoff_equator_gv: cut(d, 0.0) * 1e-9,
            equator_msv_yr: msv_yr(dose(cut(d, 0.0), 0.0, sky)),
            pole_msv_yr: msv_yr(dose(cut(d, m::PI / 2.0), 0.0, sky)),
            mean_msv_yr: msv_yr(mean_at(d, 0.0, sky)),
        });
    }
    let eq = &by_latitude[0];
    why.push(format!(
        "Cosmic rays: open space here gives about {:.0} mSv a year. {} {} On the ground that leaves about {:.2} mSv a year on average ({:.2} at the magnetic equator, {:.2} at the poles).",
        msv_yr(free),
        if eq.cutoff_gv > 0.05 {
            format!("The field turns aside those below {:.1} GV at the magnetic equator, none at the poles.", eq.cutoff_gv)
        } else {
            "No field turns any aside.".into()
        },
        if column > 0.0 {
            format!("The air above each square metre weighs {:.0} kg and absorbs most of the rest.", column)
        } else {
            "There is no air to absorb them.".into()
        },
        surface_mean,
        eq.surface_dose_msv_yr,
        by_latitude[6].surface_dose_msv_yr,
    ));
    stand_ins.push("dose from galactic cosmic rays only, for one level of the star's activity; the star's particle storms come later; belt electrons are given apart (belts). Vertical Størmer cut-off of a centred dipole".into());
    Some(Shield {
        free_space_msv_yr: msv_yr(free),
        column_kg_m2: column,
        host_cutoff_gv: host_cut * 1e-9,
        by_latitude,
        surface_mean_msv_yr: surface_mean,
        orbits,
    })
}

#[cfg(test)]
mod tests;
