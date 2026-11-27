//! XO18 part 1: the Hazard map's reading (exotic objects design 3.8). For any
//! body with something dangerous round it, how close can a ship go, and which
//! hazard stops it? One mechanism for every body: each hazard is a row read
//! along a stated line (in the magnetic equator's plane, or over the poles),
//! each row in three tones (clear, harsh, deadly), and two marks for the ship:
//!
//! - **Park**: the closest orbit it can settle into and still climb out of.
//! - **Dive**: the closest pass on a fall from far away that still comes back
//!   out (a fall-through arrives with the speed to leave, so it costs no fuel
//!   and goes far deeper than a park).
//!
//! The rows add almost no physics of their own. Heat is the light block
//! (`sse.climate.insolation`) into the temperature block
//! (`sse.climate.equilibrium_temperature`) for the hull, against what its skin
//! can take from the data sheet. Climbing out is the escape speed block
//! (`sse.planet.escape_velocity`) less the circular speed there. Radiation is
//! the trapped belts that `read_field` already gives (MF7 part 2): each
//! shell's dose behind the ship's own shield, with the moons' swept lanes.
//! Air is the top of the air where the belts' own edge sits.
//!
//! Tide, field and clocks arrive with the gravity blocks (XO1 part 2) and
//! storms, light and debris with later parts, behind the same reading.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_field::Belts;
use sse_flow::{Atom, Reference};
use sse_reference::Reference as Sheet;

#[cfg(test)]
mod tests;

const SETTINGS: &str = include_str!("../data/hazard.json");
const SOL: &str = include_str!("../../../packs/base-sol/pack.json");

/// The blocks the reading runs, in the order their references are listed.
const BLOCKS: &[&str] = &[
    "sse.climate.insolation",
    "sse.climate.equilibrium_temperature",
    "sse.planet.escape_velocity",
    "sse.belts.latitude_falloff",
    "sse.gravity.gravitational_radius",
    "sse.gravity.clock_rate",
    "sse.planet.surface_gravity",
];

/// Points along the line, spaced evenly on a stretched (logarithmic) scale.
const POINTS: usize = 400;
/// Steps along a fall-through, in true anomaly.
const PASS_STEPS: usize = 2000;

fn block(id: &str) -> std::sync::Arc<dyn Atom> {
    // ER2: the block in force, with the settings this universe gives it.
    sse_physics::block(id).unwrap_or_else(|| panic!("base formula block {id} is missing"))
}

fn run(id: &str, inputs: &[f64], params: &[f64]) -> Result<f64, String> {
    block(id)
        .eval(inputs, params)
        .map_err(|e| format!("{id}: {e}"))?
        .first()
        .copied()
        .ok_or_else(|| format!("{id} gave nothing"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Clear,
    Harsh,
    Deadly,
}

/// The line a profile is read along.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Line {
    /// Outward in the plane of the magnetic equator (the spin equator for a
    /// body with no field).
    #[default]
    Equator,
    /// Straight out over a magnetic pole. A fall-through on this line swings
    /// past the pole in the plane holding the magnetic axis.
    Poles,
}

impl Line {
    fn words(self) -> &'static str {
        match self {
            Line::Equator => "in the equator's plane",
            Line::Poles => "over the poles",
        }
    }
}

/// How the hull meets the light.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Faces {
    /// A round hull turning slowly, so every side takes the light in turn
    /// (the heat spreads over the whole skin: f = 1/4).
    #[default]
    Round,
    /// A thin sunshade held face-on to the light, giving its heat off from
    /// both faces (f = 1/2), as Parker Solar Probe flies.
    Shield,
}

impl Faces {
    fn spread(self) -> f64 {
        match self {
            Faces::Round => 0.25,
            Faces::Shield => 0.5,
        }
    }
}

/// What the reading needs of a ship (from its record; the stock ship when
/// none is chosen).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ship {
    pub name: String,
    /// Speed change left in the tanks.
    pub delta_v_m_s: f64,
    /// Aluminium-equivalent shield, kg/m² (10 kg/m² is 1 g/cm²).
    pub shield_kg_m2: f64,
    /// The hull's skin, a substance or mixture on the data sheet; what it can
    /// take is the sheet's melting (or subliming) point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin: Option<String>,
    /// What the skin can take, typed by hand (a pack's rated figure): used
    /// only when the sheet cannot give one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_k: Option<f64>,
    /// The skin's absorptivity over its emissivity (1 for a grey skin; a
    /// white coating is well under 1).
    pub absorbed_over_emitted: f64,
    #[serde(default)]
    pub faces: Faces,
    pub length_m: f64,
}

/// The body, as the kernel knows it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub address: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    /// Its own light (a star; 0 for a world, whose host's light is the same
    /// all over the map).
    #[serde(default)]
    pub luminosity_w: f64,
    /// The top of its air, from the centre; left out, read from the belts'
    /// inner edge when the air sets it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air_top_m: Option<f64>,
    /// The edge of its magnetic bubble, from the centre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bubble_m: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ask {
    pub body: Body,
    /// `read_field`'s belts for this body (none: no field, or nothing fills it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belts: Option<Belts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ship: Option<Ship>,
    #[serde(default)]
    pub line: Line,
}

/// A stretch of a row in one tone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Band {
    pub from_m: f64,
    pub to_m: f64,
    pub tone: Tone,
}

/// One hazard along the line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    /// "heat", "radiation_belts", "air", "climbing_out".
    pub hazard: String,
    pub name: String,
    pub bands: Vec<Band>,
    /// What turns it harsh and deadly here, in plain words.
    pub says: String,
}

/// A fixed mark, from the body alone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub what: String,
    pub distance_m: f64,
}

/// The belts' dose beside a moon, at its orbit (`read_field`'s own figure,
/// which ignores the moon's own sweeping: the swept lane is the moon's width).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoonReading {
    pub moon: String,
    pub distance_m: f64,
    pub sv_per_day: f64,
    pub tone: Tone,
}

/// Park or Dive: where, and which hazard set it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipMark {
    pub distance_m: f64,
    /// The hazard just inside the mark, or "surface" when nothing stops it
    /// short of the ground.
    pub set_by: String,
    pub says: String,
    /// A clock here against one far away: at Park on a circular orbit, at
    /// Dive falling through from far away (XO20).
    pub clock_rate: f64,
    /// The push to hold still here instead, in m/s² (gravity over the
    /// hovering clock rate).
    pub hover_m_s2: f64,
}

/// The totals for one fall-through, at the Dive mark.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pass {
    pub closest_m: f64,
    pub dose_sv: f64,
    pub peak_hull_k: f64,
    /// Hours spent inside the belts' outer edge.
    pub hours_in_belts: f64,
    /// Seconds the ship's clock falls behind a far clock over the pass.
    pub clock_lost_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HazardMap {
    pub address: String,
    pub line: Line,
    pub ship: Ship,
    /// True when no ship was chosen and the stated stock ship stands in.
    pub stock_ship: bool,
    /// The span the rows cover, from the centre.
    pub from_m: f64,
    pub to_m: f64,
    /// Rows that leave "clear" somewhere along the line.
    pub rows: Vec<Row>,
    /// Rows that never do, folded into one line.
    pub quiet: Vec<String>,
    pub marks: Vec<Mark>,
    /// The belts' dose beside each moon inside them, for this ship's shield.
    pub moons: Vec<MoonReading>,
    /// Every span where a ship can stay, innermost first.
    pub safe_bands: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub park: Option<ShipMark>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dive: Option<ShipMark>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass: Option<Pass>,
    pub hull_limit_k: f64,
    /// Where the hull's limit comes from: the sheet's row, or "typed".
    pub hull_limit_from: String,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Deserialize)]
struct Settings {
    dose: DoseLimits,
    margin: f64,
    stock_ship: Ship,
}

#[derive(Clone, Debug, Deserialize)]
struct DoseLimits {
    harsh_sv: f64,
    deadly_sv: f64,
}

fn settings() -> Settings {
    serde_json::from_str(SETTINGS).expect("data/hazard.json parses")
}

/// The stated stock ship (design 3.8, open point 13).
pub fn stock_ship() -> Ship {
    settings().stock_ship
}

/// What the skin can take: the sheet's melting or subliming point first, a
/// typed figure only if the sheet has none.
fn hull_limit(ship: &Ship, sheet: &Sheet) -> Result<(f64, String), String> {
    if let Some(id) = &ship.skin {
        let row = sheet
            .row("mixtures", id)
            .or_else(|| sheet.row("substances", id));
        let get = |k: &str| {
            row.and_then(|r| r.get(k))
                .and_then(|v| v.get("v").and_then(|x| x.as_f64()).or_else(|| v.as_f64()))
        };
        if let Some(t) = get("melting_point_k") {
            return Ok((t, format!("{id}.melting_point_k")));
        }
        if let Some(t) = get("sublimation_point_k") {
            return Ok((t, format!("{id}.sublimation_point_k")));
        }
    }
    match ship.max_k {
        Some(t) if t > 0.0 => Ok((t, "typed".into())),
        _ => Err(format!(
            "the ship's skin ({}) has no melting point on the data sheet and none is typed",
            ship.skin.as_deref().unwrap_or("none given")
        )),
    }
}

/// The belts as a dose rate (Sv an hour) behind one shield, at a distance
/// and magnetic latitude: the field line through the point, on a dipole, is
/// the shell at L = r / cos²λ, and the dose along it is the shell's
/// equatorial dose thinned by `sse.belts.latitude_falloff`.
struct BeltDose<'a> {
    belts: &'a Belts,
    /// ln(dose) per shell behind this ship's shield (−inf where none).
    ln_dose: Vec<f64>,
    /// The belt model's own pitch-angle anisotropy, for the fall-off.
    anisotropy: f64,
}

impl<'a> BeltDose<'a> {
    fn new(belts: &'a Belts, shield: f64) -> Self {
        let ln_dose = belts
            .shells
            .iter()
            .map(|sh| Self::pick(&belts.shields_kg_m2, &sh.dose_sv_h, shield))
            .collect();
        BeltDose {
            belts,
            ln_dose,
            anisotropy: sse_field::Settings::default().belts.anisotropy,
        }
    }

    /// ln(dose) behind `shield`, from doses listed behind shields `s`.
    /// Between two listed shields, ln dose is read straight across ln
    /// shield; outside the list, the nearest one (a lighter shield reads the
    /// thinnest listed, so it reads low; see the stand-ins).
    fn pick(s: &[f64], d: &[f64], shield: f64) -> f64 {
        let n = s.len().min(d.len());
        if n == 0 {
            return f64::NEG_INFINITY;
        }
        let lnd = |i: usize| {
            if d[i] > 0.0 {
                m::ln(d[i])
            } else {
                f64::NEG_INFINITY
            }
        };
        if shield <= s[0] {
            return lnd(0);
        }
        for i in 1..n {
            if shield <= s[i] {
                let (a, b) = (lnd(i - 1), lnd(i));
                if !a.is_finite() || !b.is_finite() {
                    return a.min(b);
                }
                let t = (m::ln(shield) - m::ln(s[i - 1])) / (m::ln(s[i]) - m::ln(s[i - 1]));
                return a + t * (b - a);
            }
        }
        lnd(n - 1)
    }

    /// Sv an hour at the shell through equatorial distance `shell_m`.
    fn on_shell(&self, shell_m: f64) -> f64 {
        let b = self.belts;
        if !(shell_m >= b.inner_m && shell_m <= b.outer_m) {
            return 0.0;
        }
        if b.gaps
            .iter()
            .any(|g| shell_m >= g.from_m && shell_m <= g.to_m)
        {
            return 0.0;
        }
        let sh = &b.shells;
        if sh.is_empty() {
            return 0.0;
        }
        let at = |i: usize| {
            let x = self.ln_dose[i];
            if x.is_finite() {
                m::exp(x)
            } else {
                0.0
            }
        };
        if shell_m <= sh[0].distance_m {
            return at(0);
        }
        for i in 1..sh.len() {
            if shell_m <= sh[i].distance_m {
                let (a, c) = (self.ln_dose[i - 1], self.ln_dose[i]);
                if !a.is_finite() || !c.is_finite() {
                    return at(i - 1).min(at(i));
                }
                let t = (m::ln(shell_m) - m::ln(sh[i - 1].distance_m))
                    / (m::ln(sh[i].distance_m) - m::ln(sh[i - 1].distance_m));
                return m::exp(a + t * (c - a));
            }
        }
        at(sh.len() - 1)
    }

    /// Sv an hour at distance `r` and magnetic latitude given by its cosine
    /// squared.
    fn at(&self, r: f64, cos2_lat: f64) -> f64 {
        if cos2_lat <= 1e-12 {
            // Straight over the pole: the field line there never closes.
            return 0.0;
        }
        let at_equator = self.on_shell(r / cos2_lat);
        if at_equator <= 0.0 || cos2_lat >= 1.0 {
            return at_equator;
        }
        // Away from the equator the belt thins along its line
        // (`sse.belts.latitude_falloff`).
        let lat = m::acos(m::sqrt(cos2_lat));
        at_equator * run("sse.belts.latitude_falloff", &[lat], &[self.anisotropy]).unwrap_or(1.0)
    }
}

/// Grid tones to bands.
fn bands(grid: &[f64], tones: &[Tone]) -> Vec<Band> {
    let mut out: Vec<Band> = vec![];
    for (i, &t) in tones.iter().enumerate() {
        let lo = if i == 0 {
            grid[0]
        } else {
            m::sqrt(grid[i - 1] * grid[i])
        };
        let hi = if i + 1 == grid.len() {
            grid[i]
        } else {
            m::sqrt(grid[i] * grid[i + 1])
        };
        match out.last_mut() {
            Some(b) if b.tone == t => b.to_m = hi,
            _ => out.push(Band {
                from_m: lo,
                to_m: hi,
                tone: t,
            }),
        }
    }
    out
}

fn tone_of(x: f64, harsh: f64, deadly: f64) -> Tone {
    if x >= deadly {
        Tone::Deadly
    } else if x >= harsh {
        Tone::Harsh
    } else {
        Tone::Clear
    }
}

/// A distance in the body's radii, or in au when far out round a star.
fn say_distance(d: f64, radius: f64, au: f64) -> String {
    if au > 0.0 && d > 0.05 * au && radius > 1e8 {
        format!("{:.2} au", d / au)
    } else {
        format!("{:.2} radii", d / radius)
    }
}

/// A span of seconds in the unit that reads best.
fn say_seconds(s: f64) -> String {
    let a = s.abs();
    if a >= 86_400.0 {
        format!("{:.1} days", s / 86_400.0)
    } else if a >= 3600.0 {
        format!("{:.1} hours", s / 3600.0)
    } else if a >= 1.0 {
        format!("{s:.1} s")
    } else if a >= 1e-3 {
        format!("{:.1} ms", s * 1e3)
    } else if a >= 1e-6 {
        format!("{:.1} µs", s * 1e6)
    } else {
        format!("{:.1} ns", s * 1e9)
    }
}

/// How a clock at this rate keeps time against one far away.
fn clock_words(rate: f64, where_: &str) -> String {
    if rate <= 0.0 {
        return format!("A clock {where_} stops against one far away.");
    }
    format!(
        "A clock {where_} loses {} a day against one far away.",
        say_seconds((1.0 - rate) * 86_400.0)
    )
}

/// The hazard map's reading for one body, ship and line.
pub fn hazard_map(ask: &Ask, sheet: &Sheet) -> Result<HazardMap, String> {
    let set = settings();
    let b = &ask.body;
    if !(b.mass_kg > 0.0 && b.radius_m > 0.0) {
        return Err(format!("{} needs a mass and a radius", b.address));
    }
    let (ship, stock) = match &ask.ship {
        Some(s) => (s.clone(), false),
        None => (set.stock_ship.clone(), true),
    };
    let (hull_k, hull_from) = hull_limit(&ship, sheet)?;
    let mut why = vec![];
    let mut stand_ins = vec![];
    let mut used: Vec<&str> = vec!["sse.planet.escape_velocity"];
    let r0 = b.radius_m;
    let mass = b.mass_kg;
    let air_top = b.air_top_m.or_else(|| {
        ask.belts
            .as_ref()
            .filter(|x| x.inner_set_by == "air")
            .map(|x| x.inner_m)
    });
    let margin = set.margin;
    // The astronomical unit, only for saying distances round a star.
    let au = sheet.constant("au").unwrap_or(0.0);

    // Heat: the hull's temperature in the body's own light.
    let albedo = 1.0 - ship.absorbed_over_emitted;
    let hull_t = |r: f64| -> Result<f64, String> {
        if b.luminosity_w <= 0.0 {
            return Ok(0.0);
        }
        let s = run("sse.climate.insolation", &[b.luminosity_w, r], &[])?;
        run(
            "sse.climate.equilibrium_temperature",
            &[s, albedo],
            &[ship.faces.spread()],
        )
    };
    // Climbing out: from a circular orbit at r, the escape speed less the
    // orbital speed (v_esc / √2).
    let climb = |r: f64| -> Result<f64, String> {
        let v = run("sse.planet.escape_velocity", &[mass, r], &[])?;
        Ok(v * (1.0 - 1.0 / m::sqrt(2.0)))
    };
    // Clocks (XO20): the gravitational radius, then the hovering and
    // orbiting rates; a fall from far away (a parabola) runs at the hovering
    // rate squared, 1 − 2rg/r. Holding still takes gravity over the hovering rate.
    let rg = run("sse.gravity.gravitational_radius", &[mass], &[])?;
    let clock = |r: f64| -> Result<(f64, f64, f64), String> {
        let out = block("sse.gravity.clock_rate")
            .eval(&[rg, r], &[])
            .map_err(|e| format!("sse.gravity.clock_rate: {e}"))?;
        let (hover, orbit) = (out[0], out[1]);
        let g = run("sse.planet.surface_gravity", &[mass, r], &[])?;
        let push = if hover > 0.0 {
            g / hover
        } else {
            f64::INFINITY
        };
        Ok((orbit, hover * hover, push))
    };
    used.push("sse.gravity.gravitational_radius");
    used.push("sse.gravity.clock_rate");
    used.push("sse.planet.surface_gravity");

    // The span: out past every hazard's harsh edge.
    let mut far = 30.0 * r0;
    if b.luminosity_w > 0.0 {
        used.push("sse.climate.insolation");
        used.push("sse.climate.equilibrium_temperature");
        // T falls as r^-1/2: where it drops to the harsh share of the limit.
        let t1 = hull_t(r0)?;
        let harsh_t = margin * hull_k;
        if t1 > harsh_t {
            far = far.max(r0 * (t1 / harsh_t) * (t1 / harsh_t));
        }
    }
    // The escape speed falls as r^-1/2 too.
    let c1 = climb(r0)?;
    let harsh_c = margin * ship.delta_v_m_s;
    if harsh_c > 0.0 && c1 > harsh_c {
        far = far.max(r0 * (c1 / harsh_c) * (c1 / harsh_c));
    }
    if let Some(bl) = &ask.belts {
        far = far.max(bl.outer_m);
        used.push("sse.belts.latitude_falloff");
    }
    if let Some(x) = b.bubble_m {
        far = far.max(x);
    }
    let far = 2.0 * far;
    let ln0 = m::ln(r0);
    let ln1 = m::ln(far);
    let grid: Vec<f64> = (0..POINTS)
        .map(|i| m::exp(ln0 + (ln1 - ln0) * (i as f64 + 0.5) / POINTS as f64))
        .collect();

    // The rows along the line.
    let dose = ask
        .belts
        .as_ref()
        .map(|bl| BeltDose::new(bl, ship.shield_kg_m2));
    let line_cos2 = match ask.line {
        Line::Equator => 1.0,
        Line::Poles => 0.0,
    };
    let day = 24.0;
    let mut heat = Vec::with_capacity(POINTS);
    let mut rad = Vec::with_capacity(POINTS);
    let mut air = Vec::with_capacity(POINTS);
    let mut out = Vec::with_capacity(POINTS);
    for &r in &grid {
        heat.push(tone_of(hull_t(r)?, margin * hull_k, hull_k));
        let sv_day = dose.as_ref().map_or(0.0, |d| d.at(r, line_cos2) * day);
        rad.push(tone_of(sv_day, set.dose.harsh_sv, set.dose.deadly_sv));
        air.push(if air_top.is_some_and(|a| r < a) {
            Tone::Deadly
        } else {
            Tone::Clear
        });
        out.push(tone_of(
            climb(r)?,
            margin * ship.delta_v_m_s,
            ship.delta_v_m_s,
        ));
    }
    let shield_g = ship.shield_kg_m2 / 10.0;
    let all: [(&str, &str, &Vec<Tone>, String); 4] = [
        (
            "heat",
            "Heat",
            &heat,
            format!(
                "harsh once the hull passes {:.0} K, deadly at {:.0} K, what its skin can take",
                margin * hull_k,
                hull_k
            ),
        ),
        (
            "radiation_belts",
            "Radiation: trapped belts",
            &rad,
            format!(
                "behind {shield_g:.0} g/cm²: harsh once a day here gives {:.2} Sv, deadly at {:.1} Sv",
                set.dose.harsh_sv, set.dose.deadly_sv
            ),
        ),
        (
            "air",
            "Air",
            &air,
            "a stay inside the air is a fall".to_string(),
        ),
        (
            "climbing_out",
            "Climbing out",
            &out,
            format!(
                "harsh once leaving needs {:.0}% of the {:.1} km/s on board, deadly past all of it",
                margin * 100.0,
                ship.delta_v_m_s / 1e3
            ),
        ),
    ];
    let mut rows = vec![];
    let mut quiet = vec![];
    for (id, name, tones, says) in &all {
        if tones.iter().all(|t| *t == Tone::Clear) {
            quiet.push((*name).to_string());
        } else {
            rows.push(Row {
                hazard: (*id).into(),
                name: (*name).into(),
                bands: bands(&grid, tones),
                says: says.clone(),
            });
        }
    }

    // Park: every span where nothing is deadly.
    let worst = |i: usize| -> Option<&str> {
        all.iter()
            .find(|(_, _, t, _)| t[i] == Tone::Deadly)
            .map(|(id, _, _, _)| *id)
    };
    let mut safe_bands: Vec<[f64; 2]> = vec![];
    let mut open: Option<usize> = None;
    for i in 0..POINTS {
        let ok = worst(i).is_none();
        match (ok, open) {
            (true, None) => open = Some(i),
            (false, Some(s)) => {
                safe_bands.push([edge(&grid, s), edge(&grid, i)]);
                open = None;
            }
            _ => {}
        }
    }
    if let Some(s) = open {
        safe_bands.push([edge(&grid, s), far]);
    }
    let park = safe_bands.first().map(|sb| {
        let at = grid.iter().position(|&g| g >= sb[0]).unwrap_or(0);
        let set_by = if at == 0 {
            "surface".to_string()
        } else {
            worst(at - 1).unwrap_or("surface").to_string()
        };
        let says = match set_by.as_str() {
            "climbing_out" => format!(
                "The closest it can park {} is {}: any lower and its {:.1} km/s will not lift it out.",
                ask.line.words(),
                say_distance(sb[0], r0, au),
                ship.delta_v_m_s / 1e3
            ),
            "surface" => format!(
                "It can park anywhere {} down to the surface.",
                ask.line.words()
            ),
            h => format!(
                "The closest it can park {} is {}: inside that the {} is deadly.",
                ask.line.words(),
                say_distance(sb[0], r0, au),
                word_of(h)
            ),
        };
        let (orbit, _, push) = clock(sb[0])?;
        Ok::<_, String>(ShipMark {
            distance_m: sb[0],
            set_by,
            says: format!("{says} {}", clock_words(orbit, "on an orbit there")),
            clock_rate: orbit,
            hover_m_s2: push,
        })
    });
    let park = park.transpose()?;
    if safe_bands.len() > 1 {
        why.push(format!(
            "There are {} places {} where it can stay: {}.",
            safe_bands.len(),
            ask.line.words(),
            safe_bands
                .iter()
                .map(|s| format!(
                    "{} to {}",
                    say_distance(s[0], r0, au),
                    if s[1] >= far {
                        "beyond".into()
                    } else {
                        say_distance(s[1], r0, au)
                    }
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Dive: a fall from far away (a parabola, its speed at each distance the
    // escape speed) with its closest point on this line.
    let gm = run("sse.planet.escape_velocity", &[mass, 1.0], &[])?;
    let gm = gm * gm / 2.0;
    let belts_out = ask.belts.as_ref().map_or(0.0, |x| x.outer_m);
    let pass_of = |q: f64| -> Result<(f64, f64, f64, f64), String> {
        // r = 2q / (1 + cos ν); dt = r² dν / h, h = √(2 GM q).
        let h = m::sqrt(2.0 * gm * q);
        let nu_out = m::acos((2.0 * q / far - 1.0).clamp(-1.0, 1.0));
        let dnu = 2.0 * nu_out / PASS_STEPS as f64;
        let (mut sv, mut hours, mut lost) = (0.0, 0.0, 0.0);
        for k in 0..PASS_STEPS {
            let nu = -nu_out + (k as f64 + 0.5) * dnu;
            let r = 2.0 * q / (1.0 + m::cos(nu));
            let dt = r * r * dnu / h;
            // Over the pole the swing crosses latitude 90° − |ν|.
            let cos2 = match ask.line {
                Line::Equator => 1.0,
                Line::Poles => {
                    let s = m::sin(nu);
                    s * s
                }
            };
            if let Some(d) = &dose {
                sv += d.at(r, cos2) * dt / 3600.0;
            }
            if r <= belts_out {
                hours += dt / 3600.0;
            }
            lost += (1.0 - clock(r)?.1) * dt;
        }
        Ok((sv, hull_t(q)?, hours, lost))
    };
    let dive_fails = |i: usize| -> Result<Option<&'static str>, String> {
        let q = grid[i];
        if air_top.is_some_and(|a| q < a) {
            return Ok(Some("air"));
        }
        let (sv, t, _, _) = pass_of(q)?;
        if t >= hull_k {
            return Ok(Some("heat"));
        }
        if sv >= set.dose.deadly_sv {
            return Ok(Some("radiation_belts"));
        }
        Ok(None)
    };
    let mut dive_at = None;
    for i in 0..POINTS {
        if dive_fails(i)?.is_none() {
            dive_at = Some(i);
            break;
        }
    }
    let (dive, pass) = match dive_at {
        Some(i) => {
            let q = edge(&grid, i);
            let set_by = if i == 0 {
                "surface"
            } else {
                dive_fails(i - 1)?.unwrap_or("surface")
            };
            let (sv, t, hours, lost) = pass_of(grid[i])?;
            let (_, fall, push) = clock(q)?;
            let says = match set_by {
                "surface" => format!(
                    "A fall-through {} can skim the surface and come back out.",
                    ask.line.words()
                ),
                "air" => format!(
                    "A fall-through {} can go down to the top of the air, {}, and come back out free.",
                    ask.line.words(),
                    say_distance(q, r0, au)
                ),
                "heat" => format!(
                    "A fall-through {} can pass {} and come back out; any deeper and the hull passes {:.0} K.",
                    ask.line.words(),
                    say_distance(q, r0, au),
                    hull_k
                ),
                _ => format!(
                    "A fall-through {} can pass {} and come back out; any deeper and the dose over the pass is deadly behind {shield_g:.0} g/cm².",
                    ask.line.words(),
                    say_distance(q, r0, au)
                ),
            };
            (
                Some(ShipMark {
                    distance_m: q,
                    set_by: set_by.into(),
                    says: format!(
                        "{says} {} Over the pass its clock falls {} behind a far one.",
                        clock_words(fall, "falling through there"),
                        say_seconds(lost)
                    ),
                    clock_rate: fall,
                    hover_m_s2: push,
                }),
                Some(Pass {
                    closest_m: grid[i],
                    dose_sv: sv,
                    peak_hull_k: t,
                    hours_in_belts: hours,
                    clock_lost_s: lost,
                }),
            )
        }
        None => (None, None),
    };

    // Fixed marks.
    let mut marks = vec![Mark {
        what: "surface".into(),
        distance_m: r0,
    }];
    if let Some(a) = air_top {
        marks.push(Mark {
            what: "top of the air".into(),
            distance_m: a,
        });
    }
    if let Some(bl) = &ask.belts {
        marks.push(Mark {
            what: format!("belts' inner edge (set by the {})", bl.inner_set_by),
            distance_m: bl.inner_m,
        });
        marks.push(Mark {
            what: format!("belts' outer edge (set by the {})", bl.outer_set_by),
            distance_m: bl.outer_m,
        });
        for g in &bl.gaps {
            marks.push(Mark {
                what: format!(
                    "lane swept by {}, {:.0} km wide",
                    name_of(&g.by),
                    (g.to_m - g.from_m) / 1e3
                ),
                distance_m: m::sqrt(g.from_m * g.to_m),
            });
        }
        for o in &bl.orbits {
            if let Some(moon) = &o.moon {
                marks.push(Mark {
                    what: format!("{}'s orbit", name_of(moon)),
                    distance_m: o.distance_m,
                });
            }
        }
    }
    if let Some(x) = b.bubble_m {
        marks.push(Mark {
            what: "edge of the magnetic bubble".into(),
            distance_m: x,
        });
    }
    marks.sort_by(|a, c| a.distance_m.total_cmp(&c.distance_m));
    let mut moons = vec![];
    if let Some(bl) = &ask.belts {
        for o in &bl.orbits {
            let Some(moon) = &o.moon else { continue };
            let lnd = BeltDose::pick(&bl.shields_kg_m2, &o.dose_sv_h, ship.shield_kg_m2);
            let sv_per_day = if lnd.is_finite() {
                m::exp(lnd) * day
            } else {
                0.0
            };
            moons.push(MoonReading {
                moon: moon.clone(),
                distance_m: o.distance_m,
                sv_per_day,
                tone: tone_of(sv_per_day, set.dose.harsh_sv, set.dose.deadly_sv),
            });
        }
    }
    for mr in &moons {
        if mr.tone != Tone::Clear {
            why.push(format!(
                "Beside {} the belts give {:.2} Sv a day behind {shield_g:.0} g/cm²: {}.",
                name_of(&mr.moon),
                mr.sv_per_day,
                if mr.tone == Tone::Deadly {
                    "deadly"
                } else {
                    "harsh"
                }
            ));
        }
    }

    if let Some(p) = &park {
        why.push(p.says.clone());
    }
    if let Some(d) = &dive {
        why.push(d.says.clone());
    }
    if !quiet.is_empty() {
        why.push(format!(
            "{}: nothing to worry about here.",
            quiet.join(", ")
        ));
    }
    if stock {
        why.push(format!(
            "No ship chosen: read for the stock ship ({:.0} km/s left, {shield_g:.0} g/cm² of aluminium).",
            ship.delta_v_m_s / 1e3
        ));
    }
    if ask.belts.is_some() {
        stand_ins.push("belts are electrons only until the proton belt lands (fudge register F40), so Earth's inner belt and doses behind thick shields read low".into());
        stand_ins.push("away from the magnetic equator the dose thins by the dipole fall-off for the belt's pitch-angle spread; the loss cone is not cut and the field is taken as an untilted dipole".into());
        if ship.shield_kg_m2
            < ask
                .belts
                .as_ref()
                .map_or(0.0, |x| x.shields_kg_m2.first().copied().unwrap_or(0.0))
        {
            stand_ins.push("this shield is lighter than the thinnest the belts list; it is read as that one, so the dose reads low".into());
        }
    }
    if b.luminosity_w > 0.0 {
        stand_ins.push("the hull is taken as settled at the temperature the light sets, so a quick pass reads a little hot".into());
    }
    stand_ins.push(
        "tide, field, clocks, storms, light from discs and debris come with later parts".into(),
    );
    let references = refs_of(&used);
    Ok(HazardMap {
        address: b.address.clone(),
        line: ask.line,
        ship,
        stock_ship: stock,
        from_m: r0,
        to_m: far,
        rows,
        quiet,
        marks,
        moons,
        safe_bands,
        park,
        dive,
        pass,
        hull_limit_k: hull_k,
        hull_limit_from: hull_from,
        why,
        stand_ins,
        references,
    })
}

/// The boundary below grid point i.
fn edge(grid: &[f64], i: usize) -> f64 {
    if i == 0 {
        grid[0]
    } else {
        m::sqrt(grid[i - 1] * grid[i])
    }
}

fn word_of(h: &str) -> &str {
    match h {
        "heat" => "heat",
        "radiation_belts" => "radiation from the belts",
        "air" => "air",
        _ => h,
    }
}

/// "sol/io" reads "Io".
fn name_of(address: &str) -> String {
    let last = address.rsplit('/').next().unwrap_or(address);
    let mut c = last.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn refs_of(ids: &[&str]) -> Vec<Reference> {
    let mut out: Vec<Reference> = vec![];
    for d in sse_atoms::decls_of(BLOCKS) {
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

/// An ask for a body of the Solar System as measured (`read_field`'s fixture
/// "solar-system"): its mass, radius and light from `packs/base-sol`, its
/// belts and bubble from the measured fields. `id` is "jupiter" or
/// "sol/jupiter".
pub fn solar_system_ask(
    id: &str,
    ship: Option<Ship>,
    line: Line,
    sheet: &Sheet,
) -> Result<Ask, String> {
    let id = id.strip_prefix("sol/").unwrap_or(id);
    let v: serde_json::Value = serde_json::from_str(SOL).map_err(|e| format!("base-sol: {e}"))?;
    let b = v["provides"]["other"]["sol"]["bodies"]
        .as_array()
        .ok_or("base-sol has no bodies")?
        .iter()
        .find(|b| b["id"] == id)
        .ok_or_else(|| format!("no body {id} in the Solar System"))?;
    let val = |k: &str| b["values"][k]["v"].as_f64();
    let set = sse_field::Settings {
        electron: match (
            sheet.constant("m_e"),
            sheet.constant("e"),
            sheet.constant("eps0"),
        ) {
            (Some(mass_kg), Some(charge_c), Some(eps0)) => Some(sse_field::Electron {
                mass_kg,
                charge_c,
                eps0,
            }),
            _ => None,
        },
        ..sse_field::Settings::default()
    };
    let mp = sheet
        .constant("m_p")
        .ok_or("the data sheet has no proton mass")?;
    let fields = sse_field::solar_system(&set, mp)?;
    let f = fields
        .iter()
        .find(|f| f.read.address == format!("sol/{id}"));
    Ok(Ask {
        body: Body {
            address: format!("sol/{id}"),
            mass_kg: val("body.mass").ok_or_else(|| format!("{id} has no mass"))?,
            radius_m: val("body.radius").ok_or_else(|| format!("{id} has no radius"))?,
            luminosity_w: val("star.luminosity").unwrap_or(0.0),
            air_top_m: None,
            bubble_m: f.map(|f| f.read.obstacle.standoff_m),
        },
        belts: f.and_then(|f| f.read.belts.clone()),
        ship,
        line,
    })
}
