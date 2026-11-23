//! F9: imported V3 systems gain a reconstructed formation history. Each
//! system's star and planets become pins (star mass and age, each planet's
//! mass and orbit); the formation suite searches for a history that makes
//! them and labels it reconstructed, so the time ruler works on imports too.

use crate::Import;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use sse_core::Address;
use sse_formation::pins::{solve, PinnedRun, Pins, WorldPin};
use sse_formation::pipeline::Block;
use sse_formation::Manifest;

const M_SUN_KG: f64 = sse_core::sheet::M_SUN;
const M_EARTH_KG: f64 = sse_core::sheet::M_EARTH;
const AU_M: f64 = sse_core::sheet::AU;
const GYR_S: f64 = 1e9 * 365.25 * 86_400.0;

/// What an import says about one system, as pins.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KnownSystem {
    pub system: Address,
    pub star: Address,
    pub star_mass: f64,
    pub stars: u32,
    pub age_gyr: Option<f64>,
    pub planets: Vec<(Address, WorldPin)>,
    /// Bodies left out (moons, belts, bodies with no mass or orbit), by address.
    pub skipped: Vec<Address>,
}

fn num(f: &std::collections::BTreeMap<String, crate::Fact>, k: &str) -> Option<f64> {
    f.get(k).and_then(|x| x.value.as_f64())
}

fn text<'a>(f: &'a std::collections::BTreeMap<String, crate::Fact>, k: &str) -> Option<&'a str> {
    f.get(k).and_then(|x| x.value.as_str())
}

/// Read every system's star and planets from an import.
pub fn known_systems(im: &Import) -> Vec<KnownSystem> {
    let mut out = vec![];
    let systems: Vec<&Address> = im
        .facts
        .keys()
        .filter(|a| {
            a.to_string()
                .rsplit('/')
                .next()
                .is_some_and(|s| s.starts_with("sys:"))
        })
        .collect();
    for sa in systems {
        let prefix = format!("{sa}/");
        let bodies: Vec<(&Address, _)> = im
            .facts
            .iter()
            .filter(|(a, _)| {
                a.to_string().starts_with(&prefix)
                    && a.to_string()[prefix.len()..].starts_with("body:")
            })
            .collect();
        let is_star = |f: &std::collections::BTreeMap<String, crate::Fact>| {
            text(f, "v3.roleHint") == Some("star") || f.contains_key("star.effective_temperature")
        };
        let stars: Vec<_> = bodies.iter().filter(|(_, f)| is_star(f)).collect();
        let Some((star, sf)) = stars.iter().max_by(|x, y| {
            num(x.1, "body.mass")
                .unwrap_or(0.0)
                .total_cmp(&num(y.1, "body.mass").unwrap_or(0.0))
        }) else {
            continue;
        };
        let Some(ms) = num(sf, "body.mass") else {
            continue;
        };
        let star_s = star.to_string();
        let mut planets = vec![];
        let mut skipped = vec![];
        for (a, f) in &bodies {
            if is_star(f) {
                continue;
            }
            let around_star = text(f, "parent") == Some(star_s.as_str())
                || text(f, "orbit.host") == Some(star_s.as_str());
            match (
                around_star,
                num(f, "body.mass"),
                num(f, "orbit.semi_major_axis"),
            ) {
                (true, Some(mk), Some(am)) if mk > 0.0 && am > 0.0 => planets.push((
                    (*a).clone(),
                    WorldPin {
                        label: f
                            .get("v3.name")
                            .and_then(|x| x.value.as_str())
                            .map_or_else(|| a.to_string(), str::to_owned),
                        mass_earth: mk / M_EARTH_KG,
                        a_au: am / AU_M,
                        e: num(f, "orbit.eccentricity"),
                    },
                )),
                _ => skipped.push((*a).clone()),
            }
        }
        let age_gyr = im
            .facts
            .get(sa)
            .and_then(|f| f.get("system.age"))
            .and_then(|x| x.value.as_f64())
            .map(|s| s / GYR_S);
        out.push(KnownSystem {
            system: sa.clone(),
            star: (*star).clone(),
            star_mass: ms / M_SUN_KG,
            stars: stars.len() as u32,
            age_gyr,
            planets,
            skipped,
        });
    }
    out
}

/// A reconstructed history for one known system. The patch is the stand-in
/// solar-neighbourhood envelope until the import says where the system is.
pub fn reconstruct(k: &KnownSystem, tries: u32) -> Result<PinnedRun, String> {
    let mut man = Manifest::new(
        &format!("reconstruct:{}", k.system),
        k.system.clone(),
        sse_formation::scenario::envelope(),
    );
    man.star_pins.primary_mass = Some(k.star_mass);
    man.star_pins.count = Some(k.stars.clamp(1, 3));
    man.inherit_pins.age_gyr = k.age_gyr;
    let pins = Pins {
        worlds: k.planets.iter().map(|p| p.1.clone()).collect(),
        tries,
        reconstruct: true,
    };
    solve(man, &pins, Block::Modern).map_err(|e| e.0)
}

/// Value as JSON, for reports.
pub fn summary(k: &KnownSystem, r: &PinnedRun) -> Json {
    serde_json::json!({
        "system": k.system.to_string(),
        "star_mass": k.star_mass,
        "pins": k.planets.iter().map(|p| serde_json::json!({"address": p.0.to_string(), "label": p.1.label})).collect::<Vec<_>>(),
        "skipped": k.skipped.len(),
        "attempt": r.attempt,
        "fit": r.fit,
        "worlds": r.run.protoplanets.len(),
        "removed": r.removed.len(),
        "notes": r.run.notes,
    })
}
