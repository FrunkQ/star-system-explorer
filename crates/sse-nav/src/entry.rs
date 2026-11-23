//! A ship at a world: aerocapture, entry and landing, powered landing and
//! take-off, worked from the ship's own numbers and the world's air and
//! gravity. Nothing here is a flag: every answer is a figure against the limit
//! of the part that bears it, how close it runs, what happens if it goes
//! wrong, and a rough chance. The GM decides; SSE4 rolls nothing (Alex, 23
//! Nov 2026: "we just want enough good information for the GM to make those
//! judgement calls"). A risky act is offered beside the safe one, never
//! filtered out.
//!
//! Physics:
//! - The air is isothermal and exponential from a reference level (the
//!   ground, or 1 bar on a giant): ρ(h) = ρ₀ e^(−h/H), ρ₀ = P M / (R T).
//! - Aerocapture is one ballistic grazing pass. Along a pass at nearly
//!   constant height the air a ship meets is ρ_p √(2π r_p H) (the Chapman
//!   grazing column; Vinh, Busemann and Culp 1980, Hypersonic and Planetary
//!   Entry Flight Mechanics, University of Michigan Press), and drag
//!   dv/ds = −ρ v / (2β), β = m / (C_d A), so ln(v_in / v_out) = ρ_p √(2π r_p H) / (2β).
//!   Peak deceleration ρ_p v² / (2β) then depends only on the speed shed and
//!   the air's scale, not on the ship.
//! - Heating at the stagnation point: Sutton and Graves 1971 (NASA TR R-376),
//!   q = k √(ρ / r_n) v³; over the pass the load is q_peak √(4π r_p H) / v.
//! - Entry and landing reuse S5's descent ([`crate::descent::descend`]) with
//!   a lander built from the ship.
//! - Powered landing and take-off: the speed of a low orbit plus a gravity
//!   loss (stand-in, fudge register).
//! - Brief crew tolerance from Eiband 1959 (NASA Memo 5-19-59E), read roughly
//!   from its eyeballs-in curve, in the pack.
//! - Chances are stand-ins (fudge register): a logistic in the ratio of a
//!   figure to its limit, and a normal spread of the pass height for the
//!   corridor.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use std::collections::BTreeMap;

use crate::descent::{descend, DescentPack, Engine, EntryState, Lander, Parachute, World};
use crate::{NavBody, NavSystem, Ship, G0, M_SUN_KG};
use sse_core::quantity::consts::{AU, G};

/// Gas constant, J/(mol K).
const R_GAS: f64 = sse_core::sheet::R_GAS;

/// A world's air as these checks read it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Air {
    /// Pressure at the reference level (the ground, or 1 bar on a giant), Pa.
    pub pressure_pa: f64,
    /// Temperature at the reference level, K.
    pub temperature_k: f64,
    /// Mean molar mass, kg/mol.
    pub molar_kg: f64,
    /// Scale height where passes are flown, m (default R T / (M g)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_height_m: Option<f64>,
    /// The gas mix for the Sutton–Graves constant (N2-O2, CO2, N2-CH4, H2-He).
    #[serde(default)]
    pub gas: String,
    /// Ground at the reference level; false for a giant's envelope.
    #[serde(default = "yes")]
    pub ground: bool,
}
fn yes() -> bool {
    true
}

/// A heat shield (the renderer's ship designer writes it, with its mass).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Shield {
    pub nose_radius_m: f64,
    /// Most heat flux it takes at the stagnation point, W/m².
    pub max_heat_flux_w_m2: f64,
    /// Most heat it takes in one pass, J/m² (none: no limit given).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_heat_load_j_m2: Option<f64>,
    /// ablative or reusable.
    #[serde(default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_t: Option<f64>,
}

/// The ship's shape in air.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Aero {
    /// Area facing the flow, m².
    pub ref_area_m2: f64,
    pub cd: f64,
    /// Wing area, m², and the most lift coefficient it gives (a winged ship).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lift_area_m2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cl_max: Option<f64>,
    /// Lift-to-drag ratio flown on entry.
    #[serde(default)]
    pub l_over_d: f64,
}

/// What the ship lands on.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LandingGear {
    pub legs: bool,
    /// Fastest touchdown the gear takes, m/s.
    pub max_touchdown_m_s: f64,
    /// A parachute, opened on the way down where the air allows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chute: Option<Parachute>,
}

/// Pack settings for these checks (`nav.entry`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EntrySettings {
    /// Volume a hull with no `aero` numbers takes per tonne, m³ (its area is
    /// a sphere's of that volume; stand-in).
    pub hull_m3_per_t: f64,
    /// Drag coefficient of a hull with no `aero` numbers.
    pub hull_cd: f64,
    /// Apoapsis of the orbit an aerocapture aims for, in the world's radii.
    pub capture_apoapsis_radii: f64,
    /// Spread of the pass height a crew flies, in scale heights (stand-in).
    pub pass_sigma_scale_heights: f64,
    /// Width of the chance curve in the ratio of a figure to its limit (stand-in).
    pub harm_width: f64,
    /// Brief crew tolerance: (seconds, g), ascending in time.
    pub brief_g: Vec<(f64, f64)>,
    /// Share of the burn time gravity pulls straight against the climb (stand-in).
    pub ascent_gravity_share: f64,
    /// Touchdown a ship with no landing gear takes, m/s.
    pub no_gear_touchdown_m_s: f64,
    /// Fastest a winged ship can land on a runway, m/s (a Shuttle landed at about 100).
    pub runway_speed_m_s: f64,
    /// Ratio of specific heats for the sound speed.
    pub gamma: f64,
}

/// A rough chance and what it rests on.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Chance {
    /// 0 to 1.
    pub p: f64,
    pub basis: String,
}

/// What a bad outcome does to the ship's kept state (for a rules program to
/// apply; the kernel only says it).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Effect {
    /// The kept field: `ship.readiness`, `ship.discipline`.
    pub kept: String,
    /// cap (no higher than value) or set.
    pub op: String,
    pub value: f64,
}

/// One danger of one act: the figure against its limit, the chance it goes
/// wrong, and what happens then.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Danger {
    /// `aerocapture.heating`, `landing.touchdown`, ...
    pub id: String,
    /// What bears it: crew, hull, shield, drive, ship.
    pub subject: String,
    /// The role whose call it is: pilot, navigator, engineer.
    pub role: String,
    pub figure: f64,
    pub limit: f64,
    pub unit: String,
    /// figure / limit (over 1 is past the limit).
    pub ratio: f64,
    /// ok, caution or danger.
    pub level: String,
    pub chance: Chance,
    pub if_wrong: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<Effect>,
    pub why: String,
}

/// One act at the world.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Check {
    /// aerocapture, entry, powered-landing, runway-landing, take-off.
    pub id: String,
    pub words: String,
    /// safe, risky, dangerous, or not-here (no air to brake in, no ground).
    pub verdict: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub figures: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dangers: Vec<Danger>,
    /// What would make it safe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub would_pass: Vec<String>,
    pub why: Vec<String>,
}

/// Everything one ship can do at one world.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AtWorld {
    pub body: String,
    pub name: String,
    pub gravity_m_s2: f64,
    pub orbit_m_s: f64,
    pub escape_m_s: f64,
    /// Speed arriving from far away, m/s, and where that comes from.
    pub v_inf_m_s: f64,
    pub arrival: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air: Option<Air>,
    pub checks: Vec<Check>,
}

/// The ship as these checks see it.
struct Hull {
    mass_kg: f64,
    area_m2: f64,
    cd: f64,
    nose_m: f64,
    flux_limit: f64,
    load_limit: Option<f64>,
    shielded: bool,
    guessed_shape: bool,
}

fn hull(s: &EntrySettings, ship: &Ship) -> Hull {
    let mass_kg = (ship.dry_t + ship.propellant()) * 1000.0;
    let (area_m2, cd, guessed_shape) = match &ship.aero {
        Some(a) => (a.ref_area_m2.max(1e-6), a.cd.max(1e-3), false),
        None => {
            // A sphere of the hull's volume.
            let r = m::cbrt(3.0 * s.hull_m3_per_t * mass_kg / 1000.0 / (4.0 * m::PI));
            (m::PI * r * r, s.hull_cd, true)
        }
    };
    let (nose_m, flux_limit, load_limit, shielded) = match &ship.shield {
        Some(sh) => (
            sh.nose_radius_m.max(0.01),
            sh.max_heat_flux_w_m2,
            sh.max_heat_load_j_m2,
            true,
        ),
        None => (m::sqrt(area_m2 / m::PI), ship.flux_limit_w_m2, None, false),
    };
    Hull {
        mass_kg,
        area_m2,
        cd,
        nose_m,
        flux_limit,
        load_limit,
        shielded,
        guessed_shape,
    }
}

impl Air {
    pub fn density0(&self) -> f64 {
        self.pressure_pa * self.molar_kg / (R_GAS * self.temperature_k.max(1.0))
    }
    pub fn scale_height(&self, g: f64) -> f64 {
        self.scale_height_m
            .unwrap_or(R_GAS * self.temperature_k / (self.molar_kg.max(1e-4) * g.max(1e-6)))
    }
}

/// The Sutton–Graves key for a mix of gases (mole fractions by formula).
pub fn gas_key(gases: &BTreeMap<String, f64>) -> String {
    let f = |k: &str| gases.get(k).copied().unwrap_or(0.0);
    if f("H2") + f("He") > 0.5 {
        "H2-He".into()
    } else if f("CO2") > 0.5 {
        "CO2".into()
    } else if f("N2") > 0.5 && f("CH4") > f("O2") {
        "N2-CH4".into()
    } else if f("N2") + f("O2") > 0.5 {
        "N2-O2".into()
    } else {
        String::new()
    }
}

/// Brief crew tolerance for a load held `t_s` seconds, g (log-log between the
/// pack's points, never under the ship's own limit for hours).
pub fn brief_g(s: &EntrySettings, t_s: f64, hours_limit: f64) -> f64 {
    let pts = &s.brief_g;
    let g = match pts.len() {
        0 => hours_limit,
        _ if t_s <= pts[0].0 => pts[0].1,
        _ if t_s >= pts[pts.len() - 1].0 => pts[pts.len() - 1].1,
        _ => {
            let i = pts.iter().position(|p| p.0 > t_s).unwrap_or(1).max(1);
            let (a, b) = (pts[i - 1], pts[i]);
            let u = (m::ln(t_s) - m::ln(a.0)) / (m::ln(b.0) - m::ln(a.0));
            m::exp(m::ln(a.1) + u * (m::ln(b.1) - m::ln(a.1)))
        }
    };
    g.max(hours_limit)
}

/// Level for a ratio of figure to limit (as [`crate::descent::level`]).
fn level_of(ratio: f64) -> &'static str {
    crate::descent::level(ratio, 1.0)
}

/// Rough chance it goes wrong at a ratio of figure to limit: a logistic,
/// half at the limit (stand-in).
fn harm(s: &EntrySettings, ratio: f64) -> Chance {
    let w = s.harm_width.max(1e-3);
    Chance {
        p: 1.0 / (1.0 + m::exp(-(ratio - 1.0) / w)),
        basis: format!("stand-in: a logistic in figure over limit, half at the limit, width {w}"),
    }
}

#[allow(clippy::too_many_arguments)]
fn danger(
    s: &EntrySettings,
    id: &str,
    subject: &str,
    role: &str,
    figure: f64,
    limit: f64,
    unit: &str,
    if_wrong: &str,
    effects: Vec<Effect>,
    why: String,
) -> Danger {
    let ratio = if limit > 0.0 {
        figure / limit
    } else {
        f64::INFINITY
    };
    let ratio = if ratio.is_finite() { ratio } else { 1.0e9 };
    Danger {
        id: id.into(),
        subject: subject.into(),
        role: role.into(),
        figure,
        limit,
        unit: unit.into(),
        ratio,
        level: level_of(ratio).into(),
        chance: harm(s, ratio),
        if_wrong: if_wrong.into(),
        effects,
        why,
    }
}

fn damage() -> Vec<Effect> {
    vec![Effect {
        kept: "ship.readiness".into(),
        op: "cap".into(),
        value: 0.5,
    }]
}

fn wreck() -> Vec<Effect> {
    vec![Effect {
        kept: "ship.readiness".into(),
        op: "set".into(),
        value: 0.0,
    }]
}

fn verdict(dangers: &[Danger]) -> String {
    let worst = dangers
        .iter()
        .map(|d| d.level.as_str())
        .fold("ok", |w, l| match (w, l) {
            ("danger", _) | (_, "danger") => "danger",
            ("caution", _) | (_, "caution") => "caution",
            _ => "ok",
        });
    match worst {
        "danger" => "dangerous",
        "caution" => "risky",
        _ => "safe",
    }
    .into()
}

fn not_here(id: &str, words: &str, why: String) -> Check {
    Check {
        id: id.into(),
        words: words.into(),
        verdict: "not-here".into(),
        figures: BTreeMap::new(),
        dangers: vec![],
        would_pass: vec![],
        why: vec![why],
    }
}

/// Speed arriving from far away on a Hohmann transfer from the next body in
/// (or out, for the innermost), m/s, with the neighbour's name.
pub fn hohmann_arrival(sys: &NavSystem, body: &NavBody) -> (f64, String) {
    let mu = G * sys.star_mass_msun * M_SUN_KG;
    let inner = sys
        .bodies
        .iter()
        .filter(|b| b.a_au < body.a_au)
        .max_by(|a, b| a.a_au.total_cmp(&b.a_au));
    let outer = sys
        .bodies
        .iter()
        .filter(|b| b.a_au > body.a_au)
        .min_by(|a, b| a.a_au.total_cmp(&b.a_au));
    let Some(from) = inner.or(outer) else {
        return (0.0, "no neighbour: arriving from rest".into());
    };
    let (r1, r2) = (from.a_au * AU, body.a_au * AU);
    let vt = m::sqrt(mu * (2.0 / r2 - 2.0 / (r1 + r2)));
    let vc = m::sqrt(mu / r2);
    (
        (vt - vc).abs(),
        format!("arriving on a Hohmann transfer from {}", from.name),
    )
}

/// Everything `ship` can do at `body`, arriving at `v_inf` m/s.
pub fn at_world(
    s: &EntrySettings,
    d: &DescentPack,
    ship: &Ship,
    body: &NavBody,
    air: Option<&Air>,
    v_inf: f64,
    arrival: String,
) -> AtWorld {
    let mu = body.mu();
    let r = body.radius_m.max(1.0);
    let g = mu / (r * r);
    let orbit = m::sqrt(mu / r);
    let escape = m::sqrt(2.0 * mu / r);
    let h = hull(s, ship);
    let air = air.filter(|a| a.pressure_pa > 0.0);
    let ground = air.is_none_or(|a| a.ground);
    let k = air
        .and_then(|a| d.sutton_graves.get(&a.gas))
        .copied()
        .unwrap_or(d.sutton_graves_default);
    let mut checks = vec![];
    checks.push(match air {
        Some(a) => aerocapture(s, d, ship, &h, a, mu, r, g, v_inf, k),
        None => not_here(
            "aerocapture",
            "Aerocapture",
            "no air to brake in: capture takes a burn".into(),
        ),
    });
    match (air, ground) {
        (_, false) => {
            for (id, words) in [("entry", "Entry and landing"), ("take-off", "Take-off")] {
                checks.push(not_here(
                    id,
                    words,
                    "no ground: the envelope thickens into fluid with no surface to stand on"
                        .into(),
                ));
            }
        }
        (air, true) => {
            if let Some(a) = air {
                checks.push(entry(s, d, ship, &h, a, &body.name, r, g, k));
                if let Some(c) = runway(s, ship, a, g) {
                    checks.push(c);
                }
            }
            checks.push(powered(s, ship, g, orbit, false));
            checks.push(powered(s, ship, g, orbit, true));
        }
    }
    AtWorld {
        body: body.address.clone(),
        name: body.name.clone(),
        gravity_m_s2: g,
        orbit_m_s: orbit,
        escape_m_s: escape,
        v_inf_m_s: v_inf,
        arrival,
        air: air.cloned(),
        checks,
    }
}

fn crewed(ship: &Ship) -> bool {
    ship.crew.is_none_or(|c| c > 0)
}

#[allow(clippy::too_many_arguments)]
fn aerocapture(
    s: &EntrySettings,
    d: &DescentPack,
    ship: &Ship,
    h: &Hull,
    a: &Air,
    mu: f64,
    r: f64,
    g: f64,
    v_inf: f64,
    k: f64,
) -> Check {
    let big_h = a.scale_height(g);
    let rho0 = a.density0();
    let beta = h.mass_kg / (h.cd * h.area_m2);
    let r_a = s.capture_apoapsis_radii.max(1.1) * r;
    // Solve the pass height (the periapsis radius enters the speeds and the
    // column; two rounds settle it to well under a scale height).
    let mut r_p = r;
    let mut out = (0.0, 0.0, 0.0, 0.0, 0.0);
    for _ in 0..3 {
        let v_p = m::sqrt(v_inf * v_inf + 2.0 * mu / r_p);
        let v_t = m::sqrt(2.0 * mu * r_a / (r_p * (r_a + r_p)));
        let col = m::sqrt(2.0 * m::PI * r_p * big_h);
        let rho_p = 2.0 * beta * m::ln(v_p / v_t) / col;
        let h_p = big_h * m::ln(rho0 / rho_p);
        out = (v_p, v_t, col, rho_p, h_p);
        r_p = r + h_p.max(0.0);
    }
    let (v_p, v_t, col, rho_p, h_p) = out;
    let v_esc_p = m::sqrt(2.0 * mu / r_p);
    let v_circ_p = m::sqrt(mu / r_p);
    // Corridor: too shallow leaves on an escape path, too deep falls in.
    let rho_of = |v_out: f64| 2.0 * beta * m::ln(v_p / v_out) / col;
    let h_of = |rho: f64| big_h * m::ln(rho0 / rho);
    let h_shallow = if v_p > v_esc_p {
        h_of(rho_of(v_esc_p))
    } else {
        f64::INFINITY
    };
    let h_deep = h_of(rho_of(v_circ_p));
    let sigma = s.pass_sigma_scale_heights.max(1e-3) * big_h;
    let phi = |x: f64| 0.5 * (1.0 + m::erf(x / m::sqrt(2.0)));
    let inside = if h_shallow.is_finite() {
        phi((h_shallow - h_p) / sigma)
    } else {
        1.0
    } - phi((h_deep - h_p) / sigma);
    let q = k * m::sqrt(rho_p / h.nose_m) * v_p * v_p * v_p;
    let load = q * m::sqrt(4.0 * m::PI * r_p * big_h) / v_p;
    let decel_g = rho_p * v_p * v_p / (2.0 * beta) / G0;
    let pass_s = col / v_p;
    let tol = brief_g(s, pass_s, ship.g_limit);
    // A burn to capture into the same orbit instead, and the small burn at
    // apoapsis that lifts the low point out of the air afterwards.
    let capture_dv = v_p - v_t;
    let top = big_h * m::ln(rho0 / d.run.top_density).max(0.0);
    let r_p2 = r + top;
    let va1 = m::sqrt(2.0 * mu * r_p / (r_a * (r_a + r_p)));
    let va2 = m::sqrt(2.0 * mu * r_p2 / (r_a * (r_a + r_p2)));
    let raise_dv = (va2 - va1).max(0.0);
    let saved_t = ship.propellant_for(capture_dv) - ship.propellant_for(raise_dv);

    let mut figures = BTreeMap::new();
    figures.insert("pass_height_m".into(), h_p);
    figures.insert("pass_speed_m_s".into(), v_p);
    figures.insert("speed_shed_m_s".into(), v_p - v_t);
    figures.insert("pass_seconds".into(), pass_s);
    figures.insert("peak_heat_w_m2".into(), q);
    figures.insert("heat_load_j_m2".into(), load);
    figures.insert("peak_g".into(), decel_g);
    figures.insert("ballistic_kg_m2".into(), beta);
    figures.insert("scale_height_m".into(), big_h);
    figures.insert("corridor_low_m".into(), h_deep);
    if h_shallow.is_finite() {
        figures.insert("corridor_high_m".into(), h_shallow);
    }
    figures.insert("burn_instead_m_s".into(), capture_dv);
    figures.insert("raise_burn_m_s".into(), raise_dv);
    figures.insert("propellant_saved_t".into(), saved_t);

    let mut dangers = vec![];
    let mut would = vec![];
    let mut why = vec![format!(
        "one pass at {:.0} km sheds {:.2} km/s of {:.2} and leaves an orbit reaching {:.0} radii out; a burn of {:.0} m/s at apoapsis then lifts the low point out of the air",
        h_p / 1000.0,
        (v_p - v_t) / 1000.0,
        v_p / 1000.0,
        s.capture_apoapsis_radii,
        raise_dv
    )];
    why.push(format!(
        "braking by air instead of a {:.0} m/s burn saves about {:.0} t of propellant",
        capture_dv, saved_t
    ));
    let shield_word = if h.shielded {
        "the shield"
    } else {
        "the bare hull"
    };
    let floor = if a.ground { 0.0 } else { -big_h };
    if h_p < floor + big_h {
        dangers.push(danger(
            s,
            "aerocapture.too-little-drag",
            "ship",
            "pilot",
            big_h,
            (h_p - floor).max(1.0),
            "m",
            "the pass has to dip so deep it meets the ground or the thick air below; the ship enters instead of skipping out",
            wreck(),
            format!(
                "the ship is too dense for the air to slow it in one pass above {:.0} km",
                (floor + big_h) / 1000.0
            ),
        ));
        let need = h.area_m2 * rho_p / (rho0 * m::exp(-(floor + 2.0 * big_h) / big_h));
        would.push(format!(
            "about {need:.0} m² facing the flow (an inflatable or a wider shield), or several passes after a burn into a long orbit"
        ));
    }
    dangers.push(danger(
        s,
        "aerocapture.heating",
        if h.shielded { "shield" } else { "hull" },
        "engineer",
        q,
        h.flux_limit,
        "W/m²",
        &format!("{shield_word} burns through at the hottest point; the hull is scorched and systems behind it fail"),
        damage(),
        format!(
            "peak heating {:.0} kW/m² against {shield_word}'s {:.0}",
            q / 1000.0,
            h.flux_limit / 1000.0
        ),
    ));
    if q > 0.75 * h.flux_limit {
        // q ∝ 1/√r_n.
        let rn = h.nose_m * (q / (0.75 * h.flux_limit)) * (q / (0.75 * h.flux_limit));
        would.push(format!(
            "a blunter nose of {rn:.1} m radius, or a shield rated for {:.0} kW/m²",
            q / 750.0
        ));
    }
    if let Some(lim) = h.load_limit {
        dangers.push(danger(
            s,
            "aerocapture.heat-load",
            "shield",
            "engineer",
            load,
            lim,
            "J/m²",
            "the shield ablates through before the pass ends",
            damage(),
            format!(
                "{:.0} MJ/m² over the pass against the shield's {:.0}",
                load / 1.0e6,
                lim / 1.0e6
            ),
        ));
    }
    if crewed(ship) {
        dangers.push(danger(
            s,
            "aerocapture.crew-g",
            "crew",
            "pilot",
            decel_g,
            tol,
            "g",
            "the crew grey out or black out mid-pass; injuries; nobody is flying the ship at its worst moment",
            vec![],
            format!(
                "{:.1} g for about {:.0} s; a crew bears about {:.0} g that long",
                decel_g, pass_s, tol
            ),
        ));
        if decel_g > 0.75 * tol {
            would.push(
                "a slower arrival: peak g depends only on the speed shed and the air, not the ship's shape"
                    .into(),
            );
        }
    }
    if let Some(sg) = ship.structure_g {
        dangers.push(danger(
            s,
            "aerocapture.structure",
            "hull",
            "engineer",
            decel_g,
            sg,
            "g",
            "the hull buckles; mountings, tanks or radiators tear away",
            damage(),
            format!("{decel_g:.1} g against the hull's {sg:.0} g"),
        ));
    }
    // The corridor: the chance the pass is flown inside it.
    let width = if h_shallow.is_finite() {
        h_shallow - h_deep
    } else {
        f64::INFINITY
    };
    let miss = (1.0 - inside).clamp(0.0, 1.0);
    dangers.push(Danger {
        id: "aerocapture.corridor".into(),
        subject: "ship".into(),
        role: "navigator".into(),
        figure: if width.is_finite() { width } else { 1.0e12 },
        limit: sigma,
        unit: "m".into(),
        ratio: if width.is_finite() && width > 0.0 {
            sigma / width
        } else {
            0.0
        },
        level: if miss > 0.25 {
            "danger"
        } else if miss > 0.05 {
            "caution"
        } else {
            "ok"
        }
        .into(),
        chance: Chance {
            p: miss,
            basis: format!(
                "stand-in: the pass height spread normally by {:.2} scale heights ({:.1} km)",
                s.pass_sigma_scale_heights,
                sigma / 1000.0
            ),
        },
        if_wrong: format!(
            "too high and the ship skips out still on its way past (a {:.0} m/s burn saves it); too low and it falls into the air and must enter or burn hard",
            capture_dv
        ),
        effects: vec![],
        why: if width.is_finite() {
            format!(
                "the corridor is {:.1} km deep ({:.1} scale heights), from {:.0} to {:.0} km",
                width / 1000.0,
                width / big_h,
                h_deep / 1000.0,
                h_shallow / 1000.0
            )
        } else {
            format!(
                "arriving this slowly any pass above {:.0} km captures",
                h_deep / 1000.0
            )
        },
    });
    if h.guessed_shape {
        why.push(format!(
            "the ship has no shape numbers, so it is taken as a sphere of {:.0} m², {:.1} m³ to the tonne",
            h.area_m2, s.hull_m3_per_t
        ));
    }
    why.push("lift is not flown, so a lifting ship has a wider corridor than this".into());
    Check {
        id: "aerocapture".into(),
        words: "Aerocapture".into(),
        verdict: verdict(&dangers),
        figures,
        dangers,
        would_pass: would,
        why,
    }
}

/// A lander with the ship's numbers.
fn lander(s: &EntrySettings, ship: &Ship, h: &Hull, tol: f64) -> Lander {
    let thrust = ship.accel_g * G0 * h.mass_kg;
    Lander {
        name: ship.name.clone(),
        mass_kg: h.mass_kg,
        area_m2: h.area_m2,
        cd: h.cd,
        nose_radius_m: h.nose_m,
        chute: ship.landing.as_ref().and_then(|l| l.chute.clone()),
        engine: (thrust > 0.0 && ship.exhaust_km_s > 0.0).then(|| Engine {
            thrust_n: thrust,
            isp_s: ship.exhaust_km_s * 1000.0 / G0,
            propellant_kg: ship.propellant() * 1000.0,
        }),
        heat_limit_w_cm2: h.flux_limit / 1.0e4,
        g_limit: tol,
        touchdown_m_s: touchdown(s, ship),
        lift_to_drag: ship.aero.as_ref().map_or(0.0, |a| a.l_over_d),
        bank_deg: 0.0,
        jettison_kg: 0.0,
    }
}

fn touchdown(s: &EntrySettings, ship: &Ship) -> f64 {
    ship.landing
        .as_ref()
        .map_or(s.no_gear_touchdown_m_s, |l| l.max_touchdown_m_s)
}

#[allow(clippy::too_many_arguments)]
fn entry(
    s: &EntrySettings,
    d: &DescentPack,
    ship: &Ship,
    h: &Hull,
    a: &Air,
    name: &str,
    r: f64,
    g: f64,
    k: f64,
) -> Check {
    let big_h = a.scale_height(g);
    let rho0 = a.density0();
    let sound = m::sqrt(s.gamma * R_GAS * a.temperature_k.max(1.0) / a.molar_kg.max(1e-4));
    let air_fn = move |z: f64| (rho0 * m::exp(-z.max(0.0) / big_h), sound);
    let tol = brief_g(s, 60.0, ship.g_limit);
    let v = lander(s, ship, h, tol);
    let w = World {
        name: name.into(),
        radius_m: r,
        gravity_m_s2: g,
        ground_m: 0.0,
        air: &air_fn,
        sutton_graves_k: k,
        surface_wind_m_s: 0.0,
    };
    let top = big_h * m::ln(rho0 / d.run.top_density).max(0.0);
    let mu = g * r * r;
    let e = EntryState {
        altitude_m: top,
        speed_m_s: m::sqrt(mu / (r + top)),
        flight_path_deg: d.entry_fpa_deg,
    };
    let run = descend(&d.run, &w, &v, &e);
    let o = &run.outcome;
    let mut figures = BTreeMap::new();
    figures.insert("peak_heat_w_m2".into(), o.peak_heat_w_cm2 * 1.0e4);
    figures.insert("heat_load_j_m2".into(), o.heat_load_j_cm2 * 1.0e4);
    figures.insert("peak_g".into(), o.peak_g);
    figures.insert("touchdown_m_s".into(), o.touchdown_m_s);
    figures.insert("propellant_used_t".into(), o.propellant_used_kg / 1000.0);
    figures.insert("duration_s".into(), o.duration_s);
    figures.insert("downrange_km".into(), o.downrange_km);
    let shield_word = if h.shielded {
        "the shield"
    } else {
        "the bare hull"
    };
    let mut dangers = vec![danger(
        s,
        "entry.heating",
        if h.shielded { "shield" } else { "hull" },
        "engineer",
        o.peak_heat_w_cm2 * 1.0e4,
        h.flux_limit,
        "W/m²",
        &format!("{shield_word} burns through; the ship breaks up or comes down crippled"),
        wreck(),
        format!(
            "peak heating {:.0} kW/m² against {shield_word}'s {:.0}",
            o.peak_heat_w_cm2 * 10.0,
            h.flux_limit / 1000.0
        ),
    )];
    if let Some(lim) = h.load_limit {
        dangers.push(danger(
            s,
            "entry.heat-load",
            "shield",
            "engineer",
            o.heat_load_j_cm2 * 1.0e4,
            lim,
            "J/m²",
            "the shield ablates through before the ship slows",
            wreck(),
            format!(
                "{:.0} MJ/m² against the shield's {:.0}",
                o.heat_load_j_cm2 / 100.0,
                lim / 1.0e6
            ),
        ));
    }
    if crewed(ship) {
        dangers.push(danger(
            s,
            "entry.crew-g",
            "crew",
            "pilot",
            o.peak_g,
            tol,
            "g",
            "the crew black out; injuries; the landing is flown by whoever is still awake",
            vec![],
            format!(
                "peak {:.1} g; a crew bears about {:.0} g for a minute",
                o.peak_g, tol
            ),
        ));
    }
    let td = touchdown(s, ship);
    let landed_fast = matches!(o.result.as_str(), "crash" | "hard-landing" | "out-of-fuel");
    dangers.push(danger(
        s,
        "entry.touchdown",
        "hull",
        "pilot",
        o.touchdown_m_s,
        td,
        "m/s",
        "a hard landing: gear collapses, the hull cracks; much harder and it is a crash",
        if o.touchdown_m_s > 3.0 * td {
            wreck()
        } else {
            damage()
        },
        format!(
            "touches down at {:.1} m/s; the {} takes {:.0}",
            o.touchdown_m_s,
            if ship.landing.as_ref().is_some_and(|l| l.legs) {
                "legs"
            } else {
                "hull"
            },
            td
        ),
    ));
    let mut would = vec![];
    if landed_fast {
        let tw = ship.accel_g * G0 / g;
        if tw < 1.5 {
            would.push(format!(
                "thrust of at least {:.2} g (now {:.2} g) to hover and set down",
                1.5 * g / G0,
                ship.accel_g
            ));
        }
    }
    let mut why = o.why.clone();
    why.insert(
        0,
        format!(
            "from the top of the air at {:.0} km, {:.1}° below the horizon, at orbital speed",
            top / 1000.0,
            -d.entry_fpa_deg
        ),
    );
    why.push(format!("descent result: {}", o.result.replace('-', " ")));
    Check {
        id: "entry".into(),
        words: "Entry and landing".into(),
        verdict: verdict(&dangers),
        figures,
        dangers,
        would_pass: would,
        why,
    }
}

/// A winged ship landing on a runway at its stall speed in the ground air
/// (lift equal to weight: v = √(2 m g / (ρ C_L,max S))).
fn runway(s: &EntrySettings, ship: &Ship, a: &Air, g: f64) -> Option<Check> {
    let ae = ship.aero.as_ref()?;
    let (area, cl) = (ae.lift_area_m2?, ae.cl_max?);
    let rho0 = a.density0();
    let landing_kg = ship.dry_t * 1000.0;
    let v_s = m::sqrt(2.0 * landing_kg * g / (rho0 * cl * area));
    let accel = ship.accel_g * G0;
    let roll = if accel > 0.0 {
        v_s * v_s / (2.0 * accel)
    } else {
        f64::INFINITY
    };
    let mut figures = BTreeMap::new();
    figures.insert("stall_m_s".into(), v_s);
    if roll.is_finite() {
        figures.insert("take_off_roll_m".into(), roll);
    }
    let dangers = vec![danger(
        s,
        "runway.speed",
        "hull",
        "pilot",
        v_s,
        s.runway_speed_m_s,
        "m/s",
        "touches down too fast for the gear and the runway; it overruns or flips",
        damage(),
        format!(
            "the wings hold it up down to {:.0} m/s at landing mass; runways take about {:.0}",
            v_s, s.runway_speed_m_s
        ),
    )];
    let need = 2.0 * landing_kg * g / (rho0 * cl * s.runway_speed_m_s * s.runway_speed_m_s);
    let would = if v_s > 0.75 * s.runway_speed_m_s {
        vec![format!("about {need:.0} m² of wing")]
    } else {
        vec![]
    };
    Some(Check {
        id: "runway-landing".into(),
        words: "Runway landing".into(),
        verdict: verdict(&dangers),
        figures,
        dangers,
        would_pass: would,
        why: vec!["glides in on its wings with empty tanks and lands at its stall speed".into()],
    })
}

/// Powered landing from a low orbit (`up` false) or take-off into one: the
/// orbit's speed plus a gravity loss g × burn time × a share (stand-in).
fn powered(s: &EntrySettings, ship: &Ship, g: f64, orbit: f64, up: bool) -> Check {
    let accel = ship.accel_g * G0;
    let tw = accel / g;
    // Under a thrust-to-weight of one it never climbs: no Δv reaches orbit.
    let burn_s = if tw > 1.0 {
        orbit / accel
    } else {
        f64::INFINITY
    };
    let loss = if burn_s.is_finite() {
        g * burn_s * s.ascent_gravity_share
    } else {
        f64::INFINITY
    };
    let dv = orbit + loss;
    let usable = ship.usable_dv_m_s();
    let (id, words) = if up {
        ("take-off", "Take-off to orbit")
    } else {
        ("powered-landing", "Powered landing from orbit")
    };
    let mut figures = BTreeMap::new();
    figures.insert("thrust_to_weight".into(), tw);
    figures.insert(
        "delta_v_m_s".into(),
        if dv.is_finite() { dv } else { 1.0e12 },
    );
    figures.insert("usable_delta_v_m_s".into(), usable);
    if dv.is_finite() {
        figures.insert("propellant_t".into(), ship.propellant_for(dv));
        figures.insert("burn_s".into(), burn_s);
    }
    let mut dangers = vec![danger(
        s,
        &format!("{id}.thrust"),
        "drive",
        "pilot",
        g,
        accel,
        "m/s²",
        if up {
            "the drive cannot lift the ship off this ground"
        } else {
            "the drive cannot hold the ship up; it falls the last of the way"
        },
        if up { vec![] } else { wreck() },
        format!(
            "thrust to weight {:.2} here ({:.2} g of thrust against {:.2} g of gravity)",
            tw,
            ship.accel_g,
            g / G0
        ),
    )];
    dangers.push(danger(
        s,
        &format!("{id}.propellant"),
        "drive",
        "engineer",
        if dv.is_finite() { dv } else { 1.0e12 },
        usable,
        "m/s",
        if up {
            "runs dry short of orbit: it falls back, or is left in a decaying arc"
        } else {
            "runs dry before the ground: it falls the rest of the way"
        },
        if up { damage() } else { wreck() },
        if dv.is_finite() {
            format!(
                "needs about {:.2} km/s, has {:.2} km/s to use",
                dv / 1000.0,
                usable / 1000.0
            )
        } else {
            "with thrust under its weight no amount of propellant reaches orbit".into()
        },
    ));
    let tol = brief_g(s, burn_s.min(1.0e6), ship.g_limit);
    let felt = accel / G0;
    if crewed(ship) && felt > 0.0 && burn_s.is_finite() {
        dangers.push(danger(
            s,
            &format!("{id}.crew-g"),
            "crew",
            "pilot",
            felt,
            tol,
            "g",
            "the crew are crushed into their seats; injuries on a long burn",
            vec![],
            format!(
                "{:.1} g for about {:.0} s; a crew bears about {:.0} g that long",
                felt, burn_s, tol
            ),
        ));
    }
    let mut would = vec![];
    if tw < 1.3 {
        would.push(format!(
            "thrust of at least {:.2} g (a thrust-to-weight of 1.3)",
            1.3 * g / G0
        ));
    }
    if dv > 0.75 * usable && dv.is_finite() {
        would.push(format!(
            "{:.0} t more propellant, or a lighter ship",
            (ship.propellant_for(dv) - ship.propellant()).max(0.0)
        ));
    }
    Check {
        id: id.into(),
        words: words.into(),
        verdict: verdict(&dangers),
        figures,
        dangers,
        would_pass: would,
        why: vec![
            format!(
                "a low orbit here is {:.2} km/s; gravity takes about {:.2} km/s more over the burn",
                orbit / 1000.0,
                if loss.is_finite() { loss / 1000.0 } else { 0.0 }
            ),
            "air drag on the way is not counted".into(),
        ],
    }
}
