//! Outflows from stars (O3b, gap R26): a cool giant's dusty wind and a
//! young star's jets, published as nebulae so the renderer draws them the
//! way it draws a planetary nebula. The physics is the Equations in
//! `packs/base-physics/formulas/outflow.json` (`sse.outflow.*`).

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Seed;
use sse_flow::Atom;
use sse_light::{LightDescription, LineEmission, Part, Reference, SIGMA};

use crate::{
    decoration_seed, line_colours, Cause, Descriptor, Jet, Kind, Knot, Lobes, Shell, PC_M, YEAR_S,
};

fn block(id: &str) -> &'static dyn Atom {
    sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .map(|a| a as &dyn Atom)
        .unwrap_or_else(|| panic!("base formula block {id} is missing"))
}

fn run(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    block(id)
        .eval(inputs, &[])
        .map_err(|e| format!("{id}: {e}"))
}

fn refs(id: &str) -> Vec<Reference> {
    block(id).decl().references.clone()
}

/// Speed of a giant's dusty wind when none is given, m/s: the middle of the
/// 5 to 30 km/s measured round AGB stars (Höfner and Olofsson 2018, section 2).
const WIND_M_S: f64 = 15e3;
/// Full opening of a young star's jet when none is given, rad (a few
/// degrees: Bally 2016, section 3).
const JET_OPENING_RAD: f64 = 0.1;
/// Most knots listed per jet.
const MAX_KNOTS: usize = 12;

/// A cool giant losing mass, from its life line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GiantInput {
    pub address: String,
    pub star: String,
    pub luminosity_w: f64,
    pub radius_m: f64,
    pub mass_kg: f64,
    pub teff_k: f64,
    /// When its strong mass loss began (it swelled to a giant), s.
    pub started_s: f64,
    /// When it stopped (it left the giant branch), s; open if still going.
    #[serde(default)]
    pub ends_s: Option<f64>,
    #[serde(default)]
    pub wind_m_s: Option<f64>,
    #[serde(default)]
    pub ambient_h_m3: f64,
}

/// A giant's dusty wind at a date: a filled shell while it blows, a
/// detached one drifting out once it stops; gone when its inner edge passes
/// a parsec.
pub fn dust_shell(seed: &Seed, p: &GiantInput, at_s: f64) -> Result<Option<Descriptor>, String> {
    if at_s < p.started_s {
        return Ok(None);
    }
    let v = p.wind_m_s.unwrap_or(WIND_M_S);
    let md = run(
        "sse.outflow.giant_mass_loss",
        &[p.luminosity_w, p.radius_m, p.mass_kg, p.teff_k],
    )?[0];
    let d = run(
        "sse.outflow.dust_shell",
        &[p.luminosity_w, p.radius_m, p.teff_k, md, v],
    )?;
    let (rc, tau_c, tc) = (d[0], d[1], d[3]);
    let stopped = p.ends_s.filter(|e| at_s >= *e);
    let blew_until = stopped.unwrap_or(at_s);
    let inner = match stopped {
        Some(e) => rc + v * (at_s - e),
        None => rc,
    };
    let outer = (rc + v * (at_s - p.started_s)).max(inner);
    if inner > PC_M {
        return Ok(None);
    }
    let fades = match p.ends_s {
        Some(e) => e + PC_M / v,
        None => f64::MAX,
    };
    // Looking out from the star the wind's depth falls as 1/r from the
    // condensation radius; a detached shell's dust has cooled as
    // T ∝ r^-0.4 (the same radiative equilibrium, p = 1).
    let tau = tau_c * rc / inner;
    let ld = p.luminosity_w * (1.0 - m::exp(-tau));
    let td = tc * m::pow(rc / inner, 0.4);
    let mut light = LightDescription::new(&p.address);
    if ld > 0.0 {
        light.parts.push(Part::Thermal {
            t_k: td,
            area_m2: ld / (SIGMA * td * td * td * td),
            emissivity: 1.0,
            cause: "dust shell".into(),
        });
    }
    let mass = md * (blew_until - p.started_s);
    let mut why = vec![
        format!(
            "{} sheds {:.1e} solar masses a year into a wind of {:.0} km/s (Schröder and Cuntz's law).",
            p.star,
            md * YEAR_S / crate::MSUN_KG,
            v / 1e3
        ),
        format!(
            "Dust forms {:.1} au out, where the wind has cooled to {:.0} K; the dust takes in {:.0} % of the starlight and glows at {:.0} K in the infrared.",
            rc / sse_core::sheet::AU,
            tc,
            100.0 * (1.0 - m::exp(-tau)),
            td
        ),
    ];
    if stopped.is_some() {
        why.push(format!(
            "The mass loss has stopped, so the shell has come away from the star: it now lies {:.3} to {:.3} pc out and keeps drifting.",
            inner / PC_M,
            outer / PC_M
        ));
    }
    let mut references = refs("sse.outflow.giant_mass_loss");
    references.extend(refs("sse.outflow.dust_shell"));
    Ok(Some(Descriptor {
        address: p.address.clone(),
        kind: Kind::DustShell,
        cause: Cause {
            event: "giant_mass_loss".into(),
            source: p.star.clone(),
            at_s: p.started_s,
        },
        at_s,
        age_s: at_s - p.started_s,
        fades_s: fades,
        phase: if stopped.is_some() { "detached" } else { "blowing" }.into(),
        shells: vec![Shell {
            label: "dusty wind".into(),
            radius_m: outer,
            speed_m_s: v,
            thickness_m: outer - inner,
            mass_kg: mass,
            n_e_m3: None,
        }],
        lobes: None,
        jet: None,
        elongation: 1.0,
        ambient_h_m3: p.ambient_h_m3,
        lighting: vec![p.star.clone()],
        decoration_seed: decoration_seed(seed, &p.address)?,
        line_colours: line_colours(&light)?,
        light,
        why,
        stand_ins: vec![
            "a steady, round wind at one speed; real AGB winds come in pulses (rings) and arcs, and binaries wind them into spirals".into(),
            "dust is grey: one opacity for all starlight, re-emitted as a black body at one temperature (F57)".into(),
        ],
        references,
    }))
}

/// A young star still feeding from its disc.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct YoungStarInput {
    pub address: String,
    pub star: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub accretion_kg_s: f64,
    /// When accretion (formation stage 3) began and ends, s.
    pub started_s: f64,
    pub ends_s: f64,
    /// The star's spin axis, a unit vector in the system frame.
    pub axis: [f64; 3],
    /// Where the jet leaves the disc, m (default five star radii, near the
    /// inner disc edge).
    #[serde(default)]
    pub launch_radius_m: Option<f64>,
    /// Time between bursts of accretion, s (default 20 years).
    #[serde(default)]
    pub knot_period_s: Option<f64>,
    #[serde(default)]
    pub ambient_h_m3: f64,
}

/// A young star's jets at a date, while it accretes.
pub fn young_jet(seed: &Seed, p: &YoungStarInput, at_s: f64) -> Result<Option<Descriptor>, String> {
    if at_s < p.started_s || at_s >= p.ends_s || p.accretion_kg_s <= 0.0 {
        return Ok(None);
    }
    let rl = p.launch_radius_m.unwrap_or(5.0 * p.radius_m);
    let o = run("sse.outflow.young_jet", &[p.accretion_kg_s, p.mass_kg, rl])?;
    let (mj, vj, pj, ls) = (o[0], o[1], o[2], o[3]);
    let age = at_s - p.started_s;
    let length = vj * age;
    let period = p.knot_period_s.unwrap_or(20.0 * YEAR_S);
    let mut knots = vec![];
    let mut k = 1.0;
    while knots.len() < MAX_KNOTS && k * period <= age {
        knots.push(Knot {
            distance_m: vj * k * period,
            age_s: k * period,
        });
        k += 1.0;
    }
    let mut light = LightDescription::new(&p.address);
    light.lines.push(LineEmission {
        line: "ha".into(),
        power_w: ls,
        cause: "shocks in the jets".into(),
        zone: None,
    });
    let why = vec![
        format!(
            "{} takes in {:.1e} solar masses a year from its disc and fires a tenth of it out in two jets at {:.0} km/s.",
            p.star,
            p.accretion_kg_s * YEAR_S / crate::MSUN_KG,
            vj / 1e3
        ),
        format!(
            "After {:.0} years each jet reaches {:.3} pc; bursts of feeding every {:.0} years leave {} knots along it, glowing red where they hit slower gas ({:.1e} W in hydrogen alpha).",
            age / YEAR_S,
            length / PC_M,
            period / YEAR_S,
            knots.len(),
            ls
        ),
    ];
    Ok(Some(Descriptor {
        address: p.address.clone(),
        kind: Kind::Jet,
        cause: Cause {
            event: "accretion".into(),
            source: p.star.clone(),
            at_s: p.started_s,
        },
        at_s,
        age_s: age,
        fades_s: p.ends_s,
        phase: "accreting".into(),
        shells: vec![],
        lobes: Some(Lobes {
            axis: p.axis,
            ratio: 2.0 / JET_OPENING_RAD,
            why: "the jets leave along the star's spin axis".into(),
        }),
        jet: Some(Jet {
            length_m: length,
            speed_m_s: vj,
            opening_rad: JET_OPENING_RAD,
            mass_rate_kg_s: mj,
            knots,
        }),
        elongation: 1.0,
        ambient_h_m3: p.ambient_h_m3,
        lighting: vec![],
        decoration_seed: decoration_seed(seed, &p.address)?,
        line_colours: line_colours(&light)?,
        light,
        why,
        stand_ins: vec![
            format!("the jets run out at one speed for ever while accretion lasts ({pj:.1e} W), never slowed by the gas they push into"),
            "knots come at one steady period, not from the star's own accretion history".into(),
            "a fixed share of the jets' power leaves as hydrogen alpha (F57)".into(),
        ],
        references: refs("sse.outflow.young_jet"),
    }))
}
