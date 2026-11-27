//! Fog of war from sensors (RK14, Atomic Rockets plan section 17), the
//! light side. What a craft gives off is read from its power flow (the
//! ledger's heat by temperature, its drive plumes, its transmitters), never
//! set up; every craft gets a default passive sensor fit by tech and size;
//! and each contact gets one of three marks for a seat: ink (detected and
//! resolved), dotted (detected, not resolved) or unseen, with a last-seen
//! ring when it was seen before. The kernel writes the marks as dated
//! `observe.grant` events (the secrets design's one ladder); this module
//! only does the sums.
//!
//! Formula blocks: `sse.fog.radiator_area`, `sse.fog.plume_temperature`,
//! `sse.fog.plume_light`, `sse.fog.last_seen_ring`
//! (packs/base-physics/formulas/fog.json).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sse_flow::{Atom, Reference};
use sse_light::emit::{self, Model, Radiator, Source, Transmitter};
use sse_light::{LightDescription, Part};

use crate::{detect, Detection, Seen, Sensor};

fn block(id: &str) -> std::sync::Arc<dyn Atom> {
    // ER2: the block in force, with the settings this universe gives it.
    sse_physics::block(id).unwrap_or_else(|| panic!("base formula block {id} is missing"))
}

fn run(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    block(id)
        .eval(inputs, &[])
        .map_err(|e| format!("{id}: {e}"))
}

fn block_refs(id: &str) -> Vec<Reference> {
    block(id).decl().references.clone()
}

/// A drive plume, from the ledger's engine reading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plume {
    /// Power carried away in the exhaust, W (½ thrust × exhaust speed).
    pub jet_power_w: f64,
    pub exhaust_m_s: f64,
    /// Mass of one exhaust particle, kg (default two atomic mass units:
    /// molecular hydrogen, the usual working fluid).
    #[serde(default)]
    pub particle_kg: Option<f64>,
}

/// A craft as sensors meet it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Craft {
    pub address: String,
    pub radius_m: f64,
    #[serde(default)]
    pub bond_albedo: f64,
    /// Stars lighting it (its reflected light and its sunlit warmth).
    #[serde(default)]
    pub lit_by: Vec<Source>,
    /// Waste heat by the temperature it leaves at (the power ledger's
    /// `heat_by_k`: kelvin to watts).
    #[serde(default)]
    pub heat_by_k: BTreeMap<u32, f64>,
    #[serde(default)]
    pub plumes: Vec<Plume>,
    #[serde(default)]
    pub transmitters: Vec<Transmitter>,
}

/// What a craft gives off and reflects, from its power flow.
pub fn signature(c: &Craft, model: &Model) -> Result<LightDescription, String> {
    let mut radiators = vec![];
    for (t, p) in &c.heat_by_k {
        if *p > 0.0 && *t > 0 {
            let a = run("sse.fog.radiator_area", &[*p, *t as f64])?[0];
            radiators.push(Radiator {
                t_k: *t as f64,
                area_m2: a,
            });
        }
    }
    let cond = emit::Conditions {
        address: c.address.clone(),
        radius_m: c.radius_m,
        bond_albedo: c.bond_albedo,
        lit_by: c.lit_by.clone(),
        radiators,
        transmitters: c.transmitters.clone(),
        ..Default::default()
    };
    let (mut d, _) = emit::describe(&cond, model)?;
    if !c.heat_by_k.is_empty() {
        d.references.extend(block_refs("sse.fog.radiator_area"));
    }
    for p in &c.plumes {
        if p.jet_power_w <= 0.0 || p.exhaust_m_s <= 0.0 {
            continue;
        }
        let mp = p.particle_kg.unwrap_or(2.0 * amu());
        let t = run("sse.fog.plume_temperature", &[p.exhaust_m_s, mp])?[0];
        let l = run("sse.fog.plume_light", &[p.jet_power_w])?[0];
        let a = run("sse.fog.radiator_area", &[l, t])?[0];
        d.parts.push(Part::Thermal {
            t_k: t,
            area_m2: a,
            emissivity: 0.9,
            cause: "drive plume".into(),
        });
        d.why.push(format!(
            "A drive plume carrying {:.2e} W at {:.0} km/s is about {:.2e} K and gives off {:.2e} W of light.",
            p.jet_power_w,
            p.exhaust_m_s / 1000.0,
            t,
            l
        ));
        d.stand_ins.push(
            "a plume gives off a fixed share of its jet power as a black body at its chamber temperature (F56)"
                .into(),
        );
        d.references.extend(block_refs("sse.fog.plume_temperature"));
        d.references.extend(block_refs("sse.fog.plume_light"));
    }
    Ok(d)
}

fn sheet() -> &'static sse_reference::Reference {
    static SHEET: std::sync::OnceLock<sse_reference::Reference> = std::sync::OnceLock::new();
    SHEET.get_or_init(sse_reference::Reference::base)
}

fn amu() -> f64 {
    sheet()
        .constant("m_u")
        .expect("the sheet has the atomic mass unit")
}

/// One instrument of a default fit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FitInstrument {
    pub sensor: Sensor,
    /// Aperture as a share of the hull's length, held between a floor and a cap.
    pub aperture_share: f64,
    pub aperture_min_m: f64,
    pub aperture_max_m: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fit {
    pub tech: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub sources: Vec<String>,
    pub instruments: Vec<FitInstrument>,
}

#[derive(Deserialize)]
struct Sensing {
    default_fits: Vec<Fit>,
}

/// The default fits base-light carries.
pub fn base_fits() -> Result<Vec<Fit>, String> {
    let v: serde_json::Value =
        serde_json::from_str(sse_light::table::BASE_PACK).map_err(|e| e.to_string())?;
    let s = v
        .pointer("/provides/other/sensing")
        .ok_or("base-light has no sensing table")?;
    let s: Sensing = serde_json::from_value(s.clone()).map_err(|e| e.to_string())?;
    Ok(s.default_fits)
}

/// A craft's free passive sensors, by tech and hull length.
pub fn default_fit(fits: &[Fit], tech: &str, hull_length_m: f64) -> Result<Vec<Sensor>, String> {
    let f = fits.iter().find(|f| f.tech == tech).ok_or_else(|| {
        let known: Vec<&str> = fits.iter().map(|f| f.tech.as_str()).collect();
        format!(
            "no default sensor fit for tech {tech}; fits: {}",
            known.join(", ")
        )
    })?;
    Ok(f.instruments
        .iter()
        .map(|i| {
            let mut s = i.sensor.clone();
            s.aperture_m = (i.aperture_share * hull_length_m)
                .max(i.aperture_min_m)
                .min(i.aperture_max_m);
            s
        })
        .collect())
}

/// A contact as one seat meets it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    pub craft: Craft,
    pub distance_m: f64,
    #[serde(default)]
    pub extinction_av: f64,
    /// When it was last seen, if it was: seconds since, and how fast it was
    /// moving and could accelerate, for the last-seen ring.
    #[serde(default)]
    pub lost_for_s: Option<f64>,
    #[serde(default)]
    pub speed_m_s: f64,
    #[serde(default)]
    pub accel_m_s2: f64,
}

/// What a seat makes of a contact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub address: String,
    /// "ink" (detected and resolved: what it is and where), "dotted"
    /// (detected, not resolved: something warm, a radio source) or "unseen".
    pub mark: String,
    /// Instruments that detect it, and which of them resolve it.
    pub detected_by: Vec<String>,
    pub resolved_by: Vec<String>,
    /// Furthest any of the seat's instruments would detect it, m.
    pub noticed_to_m: f64,
    /// Light-travel delay: it is drawn where it was this long ago, s.
    pub seen_as_of_s: f64,
    /// Unseen but seen before: radius it could be anywhere inside, m.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_m: Option<f64>,
    pub why: Vec<String>,
    pub detections: Vec<Detection>,
    pub references: Vec<Reference>,
}

/// The mark one seat's sensors give one contact.
pub fn mark(contact: &Contact, sensors: &[Sensor], model: &Model) -> Result<Mark, String> {
    let light = signature(&contact.craft, model)?;
    let seen = Seen {
        address: contact.craft.address.clone(),
        light: light.clone(),
        distance_m: contact.distance_m,
        extinction_av: contact.extinction_av,
        neighbours: vec![],
        magnetic_moment_a_m2: 0.0,
        mass_kg: 0.0,
        radar_cross_section_m2: None,
    };
    let size = 2.0 * contact.craft.radius_m / contact.distance_m;
    let c = sheet()
        .constant("c")
        .ok_or("the sheet has no speed of light")?;
    let mut detected = vec![];
    let mut resolved = vec![];
    let mut detections = vec![];
    let mut furthest: f64 = 0.0;
    let mut why = vec![];
    for s in sensors {
        let d = detect(&seen, s, &model.table)?;
        furthest = furthest.max(d.range_m);
        if d.detectable {
            detected.push(s.name.clone());
            if size >= s.resolution() {
                resolved.push(s.name.clone());
            }
        }
        detections.push(d);
    }
    let mark = if !resolved.is_empty() {
        why.push(format!(
            "Known: {} {} its shape ({:.1e} rad across, finer than the instrument's {:.1e}).",
            resolved.join(" and "),
            if resolved.len() == 1 {
                "makes out"
            } else {
                "make out"
            },
            size,
            sensors
                .iter()
                .filter(|s| resolved.contains(&s.name))
                .map(|s| s.resolution())
                .fold(f64::INFINITY, f64::min)
        ));
        "ink"
    } else if !detected.is_empty() {
        why.push(format!(
            "Noticed, not identified: {} {} it up, too small to make out at this distance.",
            detected.join(" and "),
            if detected.len() == 1 { "picks" } else { "pick" }
        ));
        "dotted"
    } else {
        why.push(format!(
            "Not seen: it would be noticed within {:.3e} m, and it is {:.3e} m away.",
            furthest, contact.distance_m
        ));
        "unseen"
    };
    for w in &light.why {
        why.push(w.clone());
    }
    let mut references = light.references.clone();
    let ring_m = match (mark, contact.lost_for_s) {
        ("unseen", Some(t)) => {
            references.extend(block_refs("sse.fog.last_seen_ring"));
            let r = run(
                "sse.fog.last_seen_ring",
                &[contact.speed_m_s, contact.accel_m_s2, t],
            )?[0];
            why.push(format!(
                "Last seen {t:.0} s ago; it could now be anywhere within {r:.3e} m of where it was."
            ));
            Some(r)
        }
        _ => None,
    };
    for d in &detections {
        for r in &d.references {
            if !references.contains(r) {
                references.push(r.clone());
            }
        }
    }
    Ok(Mark {
        address: contact.craft.address.clone(),
        mark: mark.into(),
        detected_by: detected,
        resolved_by: resolved,
        noticed_to_m: furthest,
        seen_as_of_s: contact.distance_m / c,
        ring_m,
        why,
        detections,
        references,
    })
}

/// How far a craft like this one would be noticed by these sensors: the
/// soft ring round the crew's own ship (section 17.1). It swells when the
/// drive is lit.
pub fn noticed_to(craft: &Craft, sensors: &[Sensor], model: &Model) -> Result<f64, String> {
    let probe = Contact {
        craft: craft.clone(),
        distance_m: 1.0e9,
        extinction_av: 0.0,
        lost_for_s: None,
        speed_m_s: 0.0,
        accel_m_s2: 0.0,
    };
    Ok(mark(&probe, sensors, model)?.noticed_to_m)
}
