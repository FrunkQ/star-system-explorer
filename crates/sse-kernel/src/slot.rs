//! GZ13: a star slot for every system (grand zoom design 6.3 point 2).
//!
//! A system grown from a drawn star (`from_star`) has its slot already: its
//! address is the star's. An authored or imported system is given one when
//! it is adopted (`sse_cosmos::slot::find`): a drawn star of the home
//! galaxy, by its V3 map position or the nearest one that fits its own star.
//! That star is pinned with the system's authored star laid over it, so it
//! claims its share of its cell by the pin rule (nothing counted twice),
//! and the sky, the cell and the event layer all read the same star. The
//! authored facts stay authored; where they differ from what the place
//! would have made, the slot says so, as advice, never a refusal.

use crate::adopt::{as_f64, DEFAULT_Z};
use crate::cosmos::STAR_DESCRIPTION;
use sse_chronicle::State;
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::Address;
use sse_cosmos::slot::Wanted;
use sse_cosmos::stars::{Companion, StarSystem};
use std::collections::BTreeMap;

/// On the system: the star address of its slot.
pub const STAR_SLOT: &str = "system.star_slot";
/// On the system: how it was found ("map", "fit" or "gm").
pub const STAR_SLOT_BY: &str = "system.star_slot_by";
/// On the system: the slot meets every tolerance (or the GM chose it).
pub const STAR_SLOT_FITS: &str = "system.star_slot_fits";
/// On the slot's star: the system it stands for.
pub const SLOT_OF: &str = "star.slot_of";
/// On the slot's star: what of the drawn star the authored one replaced.
pub const AUTHORED_OVER: &str = "star.authored_over";
/// On the slot's star: the drawn star as today's rules draw it (JSON).
pub const DRAWN: &str = "star.drawn";
pub const SLOT_KIND: &str = "system.star_slot";

use crate::adopt::M_SUN_KG;
use sse_core::quantity::consts::{AU as AU_M, JULIAN_YEAR as JULIAN_YEAR_S};
/// One light year in parsecs (IAU: 1 pc = 648000/pi au, 1 ly = c x Julian year).
const LY_PC: f64 = 0.306_601_394;

/// The system's own star as authored, and its [Fe/H] and companions.
pub(crate) struct Authored {
    pub wanted: Wanted,
    pub feh: Option<f64>,
    pub companions: Vec<Companion>,
}

fn is_star(r: &BTreeMap<String, Value>) -> bool {
    r.contains_key("star.effective_temperature") && r.contains_key("body.mass")
}

/// Read a system's authored star from the facts: the primary is the star
/// with no host (the heaviest if several), companions are its other stars.
pub(crate) fn authored(state: &State, system: &Address) -> Authored {
    let mut stars: Vec<(&Address, &BTreeMap<String, Value>)> = state
        .facts
        .iter()
        .filter(|(a, r)| system.contains(a) && is_star(r))
        .collect();
    let mass = |r: &BTreeMap<String, Value>| r.get("body.mass").and_then(as_f64).unwrap_or(0.0);
    stars.sort_by(|a, b| {
        let ha = a.1.contains_key("orbit.host");
        let hb = b.1.contains_key("orbit.host");
        ha.cmp(&hb)
            .then(mass(b.1).total_cmp(&mass(a.1)))
            .then(a.0.cmp(b.0))
    });
    let row = state.facts.get(system);
    let age_gyr = row
        .and_then(|r| r.get("system.age"))
        .and_then(as_f64)
        .map(|s| s / JULIAN_YEAR_S / 1.0e9);
    let feh = row
        .and_then(|r| r.get("star.metallicity_z"))
        .and_then(as_f64)
        .filter(|z| *z > 0.0)
        .map(|z| m::log10(z / DEFAULT_Z));
    let primary = stars.first().map(|(_, r)| mass(r) / M_SUN_KG);
    let companions: Vec<Companion> = stars
        .iter()
        .skip(1)
        .map(|(_, r)| Companion {
            mass_msun: mass(r) / M_SUN_KG,
            a_au: r
                .get("orbit.semi_major_axis")
                .and_then(as_f64)
                .map_or(0.0, |a| a / AU_M),
        })
        .collect();
    Authored {
        wanted: Wanted {
            mass_msun: primary.filter(|x| *x > 0.0),
            age_gyr: age_gyr.filter(|x| *x >= 0.0),
            companions: primary.map(|_| companions.len()),
        },
        feh,
        companions,
    }
}

fn num(v: Option<&Value>) -> Option<f64> {
    v.and_then(as_f64)
}

/// A V3 starmap position as an offset in pc (V3: map pixels over
/// `scale.pixelsPerUnit`, default 25, in `distanceUnit`, ly or pc). None
/// for a map drawn diagrammatically or in another unit, or no position.
pub(crate) fn map_pc(state: &State, root: &Address, system: &Address) -> Option<[f64; 3]> {
    let pos = match state.get(system, "system.position_map_units")? {
        Value::Map(p) => p,
        _ => return None,
    };
    let get = |k: &str| num(pos.get(k)).unwrap_or(0.0);
    let text = |k: &str| match state.get(root, k) {
        Some(Value::Text(t)) => Some(t.as_str()),
        _ => None,
    };
    if text("v3.mapMode").is_some_and(|m| m != "scaled") {
        return None;
    }
    let unit_pc = match text("v3.distanceUnit")? {
        "ly" => LY_PC,
        "pc" => 1.0,
        _ => return None,
    };
    let ppu = match state.get(root, "v3.scale") {
        Some(Value::Map(s)) => num(s.get("pixelsPerUnit")).filter(|x| *x > 0.0),
        _ => None,
    }
    .unwrap_or(25.0);
    let k = unit_pc / ppu;
    Some([get("x") * k, get("y") * k, get("z") * k])
}

/// The system's slot star address: the stored slot, or the system itself
/// when it was grown from a drawn star (its address is the star's).
pub(crate) fn slot_of(state: &State, system: &Address) -> Option<(Address, String)> {
    if let Some(Value::Text(t)) = state.get(system, STAR_SLOT) {
        let by = match state.get(system, STAR_SLOT_BY) {
            Some(Value::Text(b)) => b.clone(),
            _ => "fit".into(),
        };
        return t.parse().ok().map(|a| (a, by));
    }
    if matches!(state.get(system, STAR_DESCRIPTION), Some(Value::Text(_))) {
        return Some((system.clone(), "drawn".into()));
    }
    None
}

/// The slot's star as stored (the pin).
pub(crate) fn pinned(state: &State, star: &Address) -> Option<StarSystem> {
    match state.get(star, STAR_DESCRIPTION) {
        Some(Value::Text(t)) => serde_json::from_str(t).ok(),
        _ => None,
    }
}

/// The drawn star with the system's authored star laid over it: mass,
/// birth, [Fe/H] and companions where authored. The rest (its place, orbit,
/// cluster, the other abundances) is the place's own. Returns the names of
/// what was laid over.
pub(crate) fn lay_over(
    drawn: &StarSystem,
    a: &Authored,
    now_gyr: f64,
) -> (StarSystem, Vec<String>) {
    let mut s = drawn.clone();
    s.pinned = true;
    let mut over = vec![];
    if let Some(x) = a.wanted.mass_msun {
        s.mass_msun = x;
        over.push("mass_msun".to_string());
    }
    if let Some(age) = a.wanted.age_gyr {
        s.birth_gyr = now_gyr - age;
        over.push("birth_gyr".to_string());
    }
    if let Some(f) = a.feh {
        s.feh = f;
        over.push("feh".to_string());
    }
    if a.wanted.mass_msun.is_some() {
        // Separations not authored keep the drawn ones, in order.
        s.companions = a
            .companions
            .iter()
            .enumerate()
            .map(|(i, c)| Companion {
                mass_msun: c.mass_msun,
                a_au: if c.a_au > 0.0 {
                    c.a_au
                } else {
                    drawn.companions.get(i).map_or(50.0, |d| d.a_au)
                },
            })
            .collect();
        over.push("companions".to_string());
    }
    (s, over)
}

/// The facts that pin a slot star: the description (read back as stored),
/// its searchable copies, the system it stands for, what was laid over and
/// the drawn star it replaced.
pub(crate) fn pin_facts(
    star: &StarSystem,
    drawn: &StarSystem,
    system: &Address,
    over: &[String],
) -> BTreeMap<String, Value> {
    let mut f = BTreeMap::new();
    f.insert(
        STAR_DESCRIPTION.to_string(),
        Value::Text(serde_json::to_string(star).expect("plain data")),
    );
    f.insert("star.mass_msun".to_string(), Value::Float(star.mass_msun));
    f.insert("star.birth_gyr".to_string(), Value::Float(star.birth_gyr));
    f.insert("star.feh".to_string(), Value::Float(star.feh));
    f.insert(
        "star.companions".to_string(),
        Value::Int(star.companions.len() as i128),
    );
    f.insert(SLOT_OF.to_string(), Value::Text(system.to_string()));
    f.insert(
        AUTHORED_OVER.to_string(),
        Value::Array(over.iter().cloned().map(Value::Text).collect()),
    );
    f.insert(
        DRAWN.to_string(),
        Value::Text(serde_json::to_string(drawn).expect("plain data")),
    );
    f
}

/// The facts that release a slot star back to the fill.
pub(crate) fn unpin_facts() -> BTreeMap<String, Value> {
    [
        STAR_DESCRIPTION,
        "star.mass_msun",
        "star.birth_gyr",
        "star.feh",
        "star.companions",
        SLOT_OF,
        AUTHORED_OVER,
        DRAWN,
    ]
    .iter()
    .map(|k| (k.to_string(), Value::Null))
    .collect()
}

/// Galactocentric radius (kpc) and azimuth (degrees) of a position in pc.
pub(crate) fn polar(xyz: [f64; 3]) -> (f64, f64) {
    (
        m::sqrt(xyz[0] * xyz[0] + xyz[1] * xyz[1]) / 1000.0,
        m::atan2(xyz[1], xyz[0]) * 180.0 / m::PI,
    )
}
