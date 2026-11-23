//! A whole system from a coded world (X3b): the code line's pins become a
//! formation request, the formation pipeline (F9 pins) builds the system
//! around the main world, and the coded facts are written on top with
//! honesty marks where physics cannot square them.
//!
//! Nothing is silently fixed: a code that physics finds impossible is kept,
//! its facts are written as authored-impossible with the reason, and
//! [`sound`] proposes the smallest code change that would clear the mark.

use crate::{pins, typical_world, CodeSheet, Codes, SegmentPins};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::Address;
use sse_formation::history::Manifest;
use sse_formation::pins::{Pins, WorldPin};
use sse_formation::pipeline::{Block, SystemRun};
use std::collections::BTreeMap;

/// Physical constants (CODATA 2018 exact or recommended values).
const G: f64 = sse_core::sheet::G;
const M_EARTH: f64 = sse_core::sheet::M_EARTH;
const R_EARTH: f64 = sse_core::sheet::R_EARTH_MEAN;

/// What the sheet says about a world's system (pack data, `CodeSheet.system`).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct SystemHints {
    /// Segment counting gas giants in the system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub giants: Option<String>,
    /// Segment counting planetoid belts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belts: Option<String>,
    /// Source tag every generated fact carries (`traveller-generated`).
    pub tag: String,
    /// A grown world counts as a gas giant from this mass (Earth masses),
    /// whatever its envelope; the code's own reading rule decides on readback.
    #[serde(default = "ten")]
    pub giant_min_mass_earth: f64,
}
fn ten() -> f64 {
    10.0
}

/// One star as stellar data writes it ("G2 V", "M3 V", "D", "BD").
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StarCode {
    pub text: String,
    /// Mass, solar masses, from the dwarf table.
    pub mass: f64,
    /// Luminosity class as written ("V" when absent).
    pub class: String,
    /// The formation pipeline grows dwarfs; anything else is noted.
    pub note: Option<String>,
}

/// Main-sequence masses by spectral type (solar masses). Pecaut and Mamajek
/// (2013), ApJS 208, 9, doi:10.1088/0067-0049/208/1/9, with the online
/// "modern mean dwarf" table; rounded. Interpolated linearly in subtype.
const DWARF: &[(char, [f64; 3])] = &[
    // (letter, [subtype 0, subtype 5, next letter's 0])
    ('O', [60.0, 40.0, 18.0]),
    ('B', [18.0, 5.4, 2.2]),
    ('A', [2.2, 1.85, 1.6]),
    ('F', [1.6, 1.33, 1.06]),
    ('G', [1.06, 0.95, 0.88]),
    ('K', [0.88, 0.70, 0.57]),
    ('M', [0.57, 0.16, 0.08]),
];

/// Read one star from stellar data.
pub fn star(text: &str) -> Option<StarCode> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let up = t.to_ascii_uppercase();
    if up == "D" || up.starts_with("D ") || up == "WD" {
        return Some(StarCode {
            text: t.into(),
            mass: 0.6,
            class: "D".into(),
            note: Some("white dwarf: the pipeline grows main-sequence stars, so this star is grown as a dwarf of the same mass and marked".into()),
        });
    }
    if up == "BD" {
        return Some(StarCode {
            text: t.into(),
            mass: 0.05,
            class: "BD".into(),
            note: Some("brown dwarf: not yet grown by the pipeline; kept as a note".into()),
        });
    }
    let mut cs = up.chars();
    let letter = cs.next()?;
    let row = DWARF.iter().find(|(l, _)| *l == letter)?;
    let rest: String = cs.collect();
    let sub: f64 = rest
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(5.0)
        .clamp(0.0, 9.9);
    let [a, b, c] = row.1;
    let mass = if sub <= 5.0 {
        a + (b - a) * sub / 5.0
    } else {
        b + (c - b) * (sub - 5.0) / 5.0
    };
    let class = rest.split_whitespace().nth(1).unwrap_or("V").to_string();
    let note = (class != "V").then(|| {
        format!("luminosity class {class}: grown as a main-sequence star of the same type; its age is not yet chosen to make it {class}")
    });
    Some(StarCode {
        text: t.into(),
        mass,
        class,
        note,
    })
}

/// Main-sequence luminosity from mass (solar units): the textbook piecewise
/// power law (Duric 2004, Advanced Astrophysics, section 4.3).
pub fn luminosity(mass: f64) -> f64 {
    if mass < 0.43 {
        0.23 * m::pow(mass, 2.3)
    } else if mass < 2.0 {
        m::pow(mass, 4.0)
    } else if mass < 55.0 {
        1.4 * m::pow(mass, 3.5)
    } else {
        32_000.0 * mass
    }
}

/// A rocky world's mass from its radius (Earth units): Zeng, Sasselov and
/// Jacobsen (2016), ApJ 819, 127, doi:10.3847/0004-637X/819/2/127 (Earth-like
/// composition, M = R^3.7 to within a few per cent below 1.5 Earth radii).
pub fn rocky_mass(radius_earth: f64) -> f64 {
    m::pow(radius_earth.max(1e-4), 3.7)
}

/// Escape speed, m/s.
pub fn escape_speed(mass_kg: f64, radius_m: f64) -> f64 {
    m::sqrt(2.0 * G * mass_kg / radius_m)
}

/// Exosphere temperature used for the air-retention check (docs/fudges.md).
pub const EXOSPHERE_K: f64 = 1000.0; // not-sheet: a model setting (docs/fudges.md), not a measured value
/// The Jeans parameter above which a gas stays for billions of years, as the
/// base-physics atom `sse.atmosphere.jeans_parameter` explains it.
pub const JEANS_KEEP: f64 = 30.0;

/// Jeans parameter of a gas of `amu` at the exobase (taken at the surface
/// radius), through the base-physics atom.
pub fn jeans(mass_kg: f64, radius_m: f64, amu: f64) -> Option<f64> {
    let reg = sse_atoms::registry();
    let a = reg.get("sse.atmosphere.jeans_parameter")?;
    a.eval(&[mass_kg, radius_m, EXOSPHERE_K, amu], &[])
        .ok()
        .and_then(|v| v.first().copied())
}

/// Mean molecular mass of an air by its main gas (atomic mass units).
fn molar(main: Option<&str>) -> f64 {
    match main {
        Some("co2") => 44.0,
        Some("h2") => 2.0,
        Some("he") => 4.0,
        Some("ch4") => 16.0,
        Some("so2") => 64.0,
        Some("h2o") => 18.0,
        _ => 28.0,
    }
}

/// Surface temperature a coded world is placed for, K, and why.
fn target_temperature(f: &BTreeMap<String, Value>) -> (f64, &'static str) {
    let p = num(f, "atmosphere.surface_pressure").unwrap_or(0.0) / 1e5;
    let liquid = num(f, "surface.liquid_fraction").unwrap_or(0.0);
    if liquid > 0.0 && p < 0.1 {
        (
            220.0,
            "surface liquid under next to no air: placed cold, where it would be ice",
        )
    } else if liquid > 0.0 {
        (
            288.0,
            "liquid on the surface: placed for an Earth-like mean temperature",
        )
    } else {
        (
            300.0,
            "no surface liquid: placed in the middle of the warm zone",
        )
    }
}

fn num(f: &BTreeMap<String, Value>, k: &str) -> Option<f64> {
    match f.get(k)? {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// Infrared optical depth assumed for a coded air when placing the world:
/// Earth's 0.83 (the base-physics climate flow's own test value) scaled by
/// surface pressure in bar (docs/fudges.md).
pub fn optical_depth(p_bar: f64) -> f64 {
    0.83 * p_bar.max(0.0)
}

/// Equilibrium temperature that gives surface temperature `ts` under the
/// grey atmosphere of atom `sse.climate.grey_surface_temperature`
/// (T_s = T_eq (1 + 3/4 tau)^(1/4)), solved for T_eq.
pub fn teq_for(ts: f64, tau: f64) -> f64 {
    ts / m::pow(1.0 + 0.75 * tau, 0.25)
}

/// Orbit (AU) where a world of albedo 0.3 has this equilibrium temperature.
pub fn orbit_for(teq: f64, lum: f64) -> f64 {
    let t1 = 278.6 * m::pow(0.7, 0.25);
    let r = t1 / teq.max(30.0);
    m::sqrt(lum) * r * r
}

/// A physics objection to a coded value.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Objection {
    /// Segments whose codes clash.
    pub segments: Vec<String>,
    /// Facts written as authored-impossible because of it.
    pub fields: Vec<String>,
    pub reason: String,
}

/// Physics checks on a coded world's plain facts.
pub fn objections(f: &BTreeMap<String, Value>) -> Vec<Objection> {
    let mut out = Vec::new();
    let p = num(f, "atmosphere.surface_pressure").unwrap_or(0.0);
    let r = num(f, "body.radius").unwrap_or(0.0);
    if p >= 0.1e5 && r > 0.0 {
        let mass = rocky_mass(r / R_EARTH) * M_EARTH;
        let mu = molar(match f.get("atmosphere.main") {
            Some(Value::Text(t)) => Some(t.as_str()),
            _ => None,
        });
        let lambda = jeans(mass, r, mu).unwrap_or(f64::INFINITY);
        if lambda < JEANS_KEEP {
            out.push(Objection {
                segments: vec!["size".into(), "atmosphere".into()],
                fields: [
                    "atmosphere.surface_pressure",
                    "atmosphere.pp.o2",
                    "atmosphere.pp.co2",
                    "atmosphere.pp.so2",
                ]
                .iter()
                .filter(|k| f.contains_key(**k))
                .map(|k| k.to_string())
                .collect(),
                reason: format!(
                    "a world this small cannot keep air this dense: its Jeans parameter for the main gas is {lambda:.1} at a {EXOSPHERE_K} K exosphere (escape speed {:.1} km/s), and a gas stays for billions of years only above about {JEANS_KEEP}",
                    escape_speed(mass, r) / 1000.0
                ),
            });
        }
    }
    let liquid = num(f, "surface.liquid_fraction").unwrap_or(0.0);
    if liquid > 0.0 && p < 0.006e5 {
        out.push(Objection {
            segments: vec!["atmosphere".into(), "hydrographics".into()],
            fields: vec!["surface.liquid_fraction".into()],
            reason: "no liquid can stand on a surface under less than water's triple-point pressure (0.006 bar): it boils or freezes away; read it as ice cover".into(),
        });
    }
    out
}

/// One fact to author on the main world after the system is grown.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Edit {
    pub field: String,
    pub value: Value,
    pub impossible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A coded world turned into a formation request and edits.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub line: String,
    /// The gas mix chosen for the air, if the stack has one that fits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mix: Option<String>,
    pub seed: String,
    pub stars: Vec<StarCode>,
    pub pin: WorldPin,
    /// Why the orbit was chosen.
    pub placed: String,
    pub want_giants: Option<u32>,
    pub want_belts: Option<u32>,
    /// Facts written on the main world (codes win).
    pub edits: Vec<Edit>,
    pub objections: Vec<Objection>,
    pub tag: String,
    /// From the sheet's hints: the mass from which a grown world counts as a giant.
    pub giant_min: f64,
}

/// Plan a system for a world: `lines` maps layout id to code line (at
/// least the profile layout), `extra` holds facts the line does not carry
/// (name, zone, bases, stellar data as text), `seed` names the system.
pub fn plan(
    codes: &Codes,
    sheet: &CodeSheet,
    lines: &[(&str, &str)],
    stars_text: &[&str],
    extra: &BTreeMap<String, Value>,
    seed: &str,
) -> Result<Plan, String> {
    let hints = sheet
        .system
        .clone()
        .ok_or_else(|| format!("code sheet {} says nothing about systems", sheet.id))?;
    let mut all: Vec<SegmentPins> = Vec::new();
    for (layout, line) in lines {
        all.extend(pins(codes, sheet, layout, line)?);
    }
    if let Some(u) = all.iter().find(|p| p.unknown) {
        return Err(format!(
            "{}: symbol {} is not one the sheet knows",
            u.segment, u.symbol
        ));
    }
    let mut world = typical_world(sheet, &all, &BTreeMap::new());
    // A named gas mix for the air code, picked by the seed (handoff 7A part 5).
    let fitting: Vec<&crate::mix::GasMix> = codes
        .mixes
        .iter()
        .filter(|m| all.iter().any(|p| m.fits(&p.segment, &p.symbol)))
        .collect();
    let u = sse_core::draw::draw(
        &sse_core::draw::Seed::from_text(seed),
        &"u:codes".parse().expect("a fixed address"),
        &sse_core::draw::Stream::new("codes.gas_mix.v1").expect("a fixed stream"),
        0,
    ) as f64
        / 18_446_744_073_709_551_616.0;
    let mix = crate::mix::pick(&fitting, u).cloned();
    if let Some(m) = &mix {
        m.apply(&mut world);
    }
    let stars: Vec<StarCode> = stars_text.iter().filter_map(|s| star(s)).collect();
    let primary = stars.first().map_or(1.0, |s| s.mass);
    let lum = luminosity(primary);
    let r = num(&world, "body.radius").unwrap_or(0.0);
    let (t, why) = target_temperature(&world);
    let p_bar = num(&world, "atmosphere.surface_pressure").unwrap_or(0.0) / 1e5;
    let teq = teq_for(t, optical_depth(p_bar));
    let a_au = orbit_for(teq, lum);
    let mass_earth = if r > 0.0 {
        rocky_mass(r / R_EARTH)
    } else {
        1e-4
    };
    let count = |seg: &Option<String>| {
        seg.as_ref().and_then(|s| {
            all.iter()
                .find(|p| &p.segment == s)
                .and_then(|p| p.value)
                .map(|v| v as u32)
        })
    };
    let objections = objections(&world);
    let mut edits: Vec<Edit> = world
        .iter()
        .map(|(k, v)| {
            let o = objections.iter().find(|o| o.fields.contains(k));
            Edit {
                field: k.clone(),
                value: v.clone(),
                impossible: o.is_some(),
                reason: o.map(|o| o.reason.clone()),
            }
        })
        .collect();
    for (k, v) in extra {
        edits.push(Edit {
            field: k.clone(),
            value: v.clone(),
            impossible: false,
            reason: None,
        });
    }
    edits.push(Edit {
        field: "provenance.generator".into(),
        value: Value::Text(hints.tag.clone()),
        impossible: false,
        reason: None,
    });
    let line: Vec<&str> = lines.iter().map(|(_, l)| *l).collect();
    edits.push(Edit {
        field: "provenance.code".into(),
        value: Value::Text(line.join(" ")),
        impossible: false,
        reason: None,
    });
    edits.sort_by(|a, b| a.field.cmp(&b.field));
    Ok(Plan {
        mix: mix.map(|m| m.id),
        line: line.join(" "),
        seed: seed.into(),
        stars,
        pin: WorldPin {
            label: "main world".into(),
            mass_earth,
            a_au,
            e: Some(0.02),
        },
        placed: format!(
            "{why}: {t:.0} K at the surface under a grey atmosphere of optical depth {:.2} needs {teq:.0} K at albedo 0.3, at {a_au:.3} au from a star of luminosity {lum:.3}",
            optical_depth(p_bar)
        ),
        want_giants: count(&hints.giants),
        want_belts: count(&hints.belts),
        edits,
        objections,
        giant_min: hints.giant_min_mass_earth,
        tag: hints.tag,
    })
}

/// The grown system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grown {
    pub plan: Plan,
    /// The formation request that reproduces this system.
    pub rerun: Rerun,
    pub run: SystemRun,
    /// The main world's address in the run.
    pub main_world: Address,
    pub giants: u32,
    pub belts: u32,
    /// Which candidate seed won, and what was asked but could not be met.
    pub attempt: u32,
    pub unmet: Vec<String>,
}

fn giants_in(run: &SystemRun, min: f64) -> u32 {
    run.worlds
        .iter()
        .filter(|w| w.envelope == sse_formation::early::Envelope::Giant || w.mass_earth >= min)
        .count() as u32
}

fn belts_in(run: &SystemRun) -> u32 {
    run.leftovers.as_ref().map_or(0, |l| {
        l.clouds
            .iter()
            .filter(|c| c.around.is_none() && c.kind.contains("belt"))
            .count() as u32
    })
}

/// Grow the system: formation searches `tries` histories per candidate
/// around the main-world pin; candidates are tried until the giant and belt
/// counts match (at most `candidates`), else the closest is kept. Every
/// step is seeded, so the same plan grows the same system on any device.
/// The formation request a grown system reruns from: the candidate's seed,
/// the star pins and the main-world pin. Running it again (for example
/// through the kernel's `adopt`) gives the same system.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rerun {
    pub seed: String,
    pub star_pins: sse_formation::stars::StarPins,
    pub pins: Pins,
}

impl Plan {
    /// Candidate `k`'s formation request (`k` = 0 is the plan's own seed).
    pub fn rerun(&self, k: u32, tries: u32) -> Rerun {
        let seed = if k == 0 {
            self.seed.clone()
        } else {
            format!("{}#counts-{k}", self.seed)
        };
        Rerun {
            seed,
            star_pins: sse_formation::stars::StarPins {
                primary_mass: self.stars.first().map(|s| s.mass),
                count: Some((self.stars.len().clamp(1, 3)) as u32),
            },
            pins: Pins {
                worlds: vec![self.pin.clone()],
                tries,
                reconstruct: false,
            },
        }
    }
}

pub fn grow(plan: Plan, system: Address, candidates: u32, tries: u32) -> Result<Grown, String> {
    let mut best: Option<(u32, u32, sse_formation::pins::PinnedRun)> = None;
    for k in 0..candidates.max(1) {
        let r = plan.rerun(k, tries);
        let mut man = Manifest::new(&r.seed, system.clone(), sse_formation::scenario::envelope());
        man.star_pins = r.star_pins.clone();
        let pr = sse_formation::pins::solve(man, &r.pins, Block::Modern).map_err(|e| e.0)?;
        let g = giants_in(&pr.run, plan.giant_min);
        let b = belts_in(&pr.run);
        let miss = plan.want_giants.map_or(0, |w| w.abs_diff(g))
            + plan.want_belts.map_or(0, |w| w.abs_diff(b));
        if best.as_ref().is_none_or(|x| miss < x.0) {
            best = Some((miss, k, pr));
        }
        if miss == 0 {
            break;
        }
    }
    let (_, attempt, pr) = best.expect("at least one candidate");
    let main_world = pr
        .matches
        .first()
        .map(|mm| mm.world.clone())
        .ok_or("the pin was not honoured")?;
    let giants = giants_in(&pr.run, plan.giant_min);
    let belts = belts_in(&pr.run);
    let mut unmet = Vec::new();
    if let Some(w) = plan.want_giants.filter(|w| *w != giants) {
        unmet.push(format!(
            "{w} gas giants asked for; the closest history found has {giants}"
        ));
    }
    if let Some(w) = plan.want_belts.filter(|w| *w != belts) {
        unmet.push(format!(
            "{w} planetoid belts asked for; the closest history found has {belts}"
        ));
    }
    unmet.extend(plan.stars.iter().filter_map(|s| s.note.clone()));
    Ok(Grown {
        rerun: plan.rerun(attempt, tries),
        plan,
        run: pr.run,
        main_world,
        giants,
        belts,
        attempt,
        unmet,
    })
}

/// "Make this physically sound": the smallest change to the code that
/// clears every objection, trying one segment at a time, nearest symbols
/// first. Returns the new line and what changed, or None when no single
/// change clears them.
pub fn sound(
    codes: &Codes,
    sheet: &CodeSheet,
    layout: &str,
    line: &str,
) -> Result<Option<(String, String)>, String> {
    let segs = crate::parse(sheet, layout, line)?;
    let p = pins(codes, sheet, layout, line)?;
    let world = typical_world(sheet, &p, &BTreeMap::new());
    let obj = objections(&world);
    if obj.is_empty() {
        return Ok(Some((line.into(), "already physically sound".into())));
    }
    let involved: Vec<String> = {
        let mut v: Vec<String> = obj.iter().flat_map(|o| o.segments.clone()).collect();
        v.sort();
        v.dedup();
        v
    };
    let template = &sheet.layouts[layout];
    let mut best: Option<(i64, String, String)> = None;
    for seg in &involved {
        let Some(g) = sheet.segments.iter().find(|g| &g.id == seg) else {
            continue;
        };
        let crate::Source::Classify { symbols, .. } = &g.source else {
            continue;
        };
        let cur = segs.iter().find(|(s, _)| s == seg).map(|(_, y)| y.clone());
        let cur_v = symbols
            .iter()
            .find(|y| Some(&y.symbol) == cur.as_ref())
            .and_then(|y| y.value)
            .unwrap_or(0.0);
        for y in symbols {
            let Some(v) = y.value else { continue };
            if Some(&y.symbol) == cur.as_ref() {
                continue;
            }
            let trial: Vec<crate::SegmentReading> = segs
                .iter()
                .map(|(s, sym)| crate::SegmentReading {
                    id: s.clone(),
                    symbol: if s == seg {
                        y.symbol.clone()
                    } else {
                        sym.clone()
                    },
                    value: None,
                    why: crate::Why::NoSystem { tag: String::new() },
                })
                .collect();
            let tl = crate::fill(template, &trial);
            let tp = pins(codes, sheet, layout, &tl)?;
            if objections(&typical_world(sheet, &tp, &BTreeMap::new())).is_empty() {
                let cost = ((v - cur_v).abs() * 100.0) as i64;
                let what = format!(
                    "{} {} to {}",
                    g.label,
                    cur.clone().unwrap_or_default(),
                    y.symbol
                );
                if best
                    .as_ref()
                    .is_none_or(|b| cost < b.0 || (cost == b.0 && tl < b.1))
                {
                    best = Some((cost, tl, what));
                }
            }
        }
    }
    Ok(best.map(|(_, l, w)| (l, w)))
}
