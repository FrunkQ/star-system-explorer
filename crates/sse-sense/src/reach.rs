//! RK12: effect at range (Atomic Rockets plan section 13). With this weapon,
//! against that target, at this distance, what arrives and is it enough?
//! Nothing is fired and no damage is kept: the answer is a reading (in
//! range, marginal, out of range), the chance of a hit, and the distances
//! where the reading changes, for a ring on the chart.
//!
//! One mechanism for every weapon: each family gives the energy per square
//! metre the target's skin takes in (`sse.range.beam_spot`,
//! `sse.range.burst_fluence`, `sse.range.impact_fluence`); the skin's own
//! figure comes from the master data sheet (`sse.range.burn_through`); the
//! two meet in `sse.range.reading`. The chance of a hit is light lag plus
//! the shot's flight time against how far the target can move
//! (`sse.range.hit_odds`). A missile's reach is the same sum NAV does for any
//! ship, so it is left to NAV: a burst is read as if it arrived.
//!
//! A beam weapon is also a load on the power ledger (RK9): it draws its beam
//! power over its efficiency, and the rest goes to the heat bus.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::{Atom, Reference};
use sse_reference::Reference as Sheet;

/// A formula block from the base packs.
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

fn one() -> f64 {
    1.0
}

/// What the weapon sends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Weapon {
    /// A laser, or any beam that spreads by diffraction (a particle beam
    /// with its divergence given as `jitter_rad` and `spread` 1).
    Beam {
        beam_power_w: f64,
        wavelength_m: f64,
        aperture_m: f64,
        /// Beam quality M² (the setting's `spread_over_diffraction`): 1 is a
        /// perfect beam; below 1 is past diffraction, a fictional advance.
        #[serde(default = "one")]
        spread: f64,
        #[serde(default)]
        jitter_rad: f64,
        #[serde(default = "one")]
        dwell_s: f64,
        /// Wall plug to beam; left out, the load cannot be stated.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        efficiency: Option<f64>,
        /// Temperature its waste heat is rejected at.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reject_k: Option<f64>,
    },
    /// A burst in space (a warhead), read as if it arrived at the distance.
    Burst {
        yield_j: f64,
        /// Share leaving as X-rays (Glasstone and Dolan section 1.77: 0.7 to 0.8).
        xray_share: f64,
    },
    /// A slug from a gun or coilgun.
    Projectile {
        mass_kg: f64,
        muzzle_m_s: f64,
        face_m2: f64,
    },
}

/// The target's skin: a material on the sheet, or a figure typed by hand.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Skin {
    /// A mixture or substance id on the master data sheet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default)]
    pub thickness_m: f64,
    /// "boil" (through, the default) or "melt".
    #[serde(default = "boil")]
    pub effect: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_k: Option<f64>,
    /// Share of arriving energy the skin takes in; left out, all of it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absorbed_share: Option<f64>,
    /// What it can take, typed by hand (a game pack's armour figure): kept
    /// as authored and used only when the sheet cannot give one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub takes_j_m2: Option<f64>,
}

fn boil() -> String {
    "boil".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub skin: Skin,
    /// Half its width as the shooter sees it.
    pub radius_m: f64,
    /// How hard it can accelerate to dodge.
    #[serde(default)]
    pub accel_m_s2: f64,
    /// Closing speed between the ships, added to a projectile's muzzle speed.
    #[serde(default)]
    pub closing_m_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ask {
    pub weapon: Weapon,
    pub target: Target,
    pub distance_m: f64,
}

/// Distances where the reading changes, for a ring on the chart. `None`:
/// the energy that arrives does not fall with distance (a slug), so only the
/// odds limit it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rings {
    pub in_range_m: Option<f64>,
    pub enough_m: Option<f64>,
    pub marginal_m: Option<f64>,
    /// Where the chance of a hit falls to one in two.
    pub even_odds_m: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    /// "in range", "marginal", "out of range", or "unknown" when the target's
    /// figure cannot be found.
    pub reading: String,
    pub arrives_j_m2: f64,
    pub takes_j_m2: Option<f64>,
    /// "sheet" (worked out from the material's row) or "authored".
    pub takes_from: Option<String>,
    pub ratio: Option<f64>,
    /// Chance of a hit, 0 to 1; `None` for a burst (its delivery is a
    /// missile's reach, NAV's sum).
    pub odds: Option<f64>,
    pub delay_s: Option<f64>,
    pub wander_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spot_radius_m: Option<f64>,
    pub rings: Rings,
    /// The weapon as a load on the power ledger (beams with an efficiency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<sse_nav::power::Load>,
    /// Sheet figures that were wanted and not found.
    pub missing: Vec<String>,
    pub why: Vec<String>,
    pub references: Vec<Reference>,
}

const READINGS: [&str; 3] = ["out of range", "marginal", "in range"];

/// The skin's figures from the sheet: density, heat capacity near 298 K,
/// heat to the melting point, heat of melting, heat from melting to boiling,
/// heat of boiling (NIST-JANAF increments, so a heat capacity that climbs
/// with temperature is counted).
fn skin_rows(sheet: &Sheet, id: &str, missing: &mut Vec<String>) -> Option<[f64; 6]> {
    let row = sheet
        .row("mixtures", id)
        .or_else(|| sheet.row("substances", id))?;
    // Mixtures are flat; substances keep these under phases and thermal.
    let get = |k: &str| -> Option<f64> {
        let v = row
            .get(k)
            .or_else(|| row.get("phases").and_then(|p| p.get(k)))
            .or_else(|| row.get("thermal").and_then(|p| p.get(k)))?;
        v.get("v").and_then(|x| x.as_f64()).or_else(|| v.as_f64())
    };
    let mut want = |k: &str, v: Option<f64>| -> f64 {
        v.unwrap_or_else(|| {
            missing.push(format!("{id}.{k}"));
            f64::NAN
        })
    };
    let rho = want("density", get("density").or_else(|| get("density_solid")));
    let c = want("heat_capacity", get("heat_capacity"));
    // A body that turns straight to vapour (graphite) has no liquid: its
    // whole climb is the heat to sublime, then the heat of sublimation.
    let (hm, lf, hb, lv) = match (get("latent_sublimation"), get("latent_fusion")) {
        (Some(ls), None) => (
            want("heat_to_sublime", get("heat_to_sublime")),
            0.0,
            0.0,
            ls,
        ),
        _ => (
            want("heat_to_melt", get("heat_to_melt")),
            want("latent_fusion", get("latent_fusion")),
            want("heat_to_boil", get("heat_to_boil")),
            want("latent_vaporisation", get("latent_vaporisation")),
        ),
    };
    let all = [rho, c, hm, lf, hb, lv];
    all.iter().all(|v| v.is_finite()).then_some(all)
}

/// Energy per square metre arriving and taken in at distance `d`, the spot
/// radius for a beam.
fn arrives(w: &Weapon, t: &Target, d: f64, absorbed: f64) -> Result<(f64, Option<f64>), String> {
    match w {
        Weapon::Beam {
            beam_power_w,
            wavelength_m,
            aperture_m,
            spread,
            jitter_rad,
            dwell_s,
            ..
        } => {
            let o = run(
                "sse.range.beam_spot",
                &[
                    *beam_power_w,
                    *wavelength_m,
                    *aperture_m,
                    *spread,
                    *jitter_rad,
                    d,
                    *dwell_s,
                    absorbed,
                ],
            )?;
            Ok((o[2], Some(o[0])))
        }
        Weapon::Burst {
            yield_j,
            xray_share,
        } => Ok((
            run(
                "sse.range.burst_fluence",
                &[*yield_j, *xray_share, d, absorbed],
            )?[0],
            None,
        )),
        Weapon::Projectile {
            mass_kg,
            muzzle_m_s,
            face_m2,
        } => {
            let v = muzzle_m_s + t.closing_m_s.max(0.0);
            let o = run("sse.range.impact_fluence", &[*mass_kg, v, *face_m2])?;
            Ok((absorbed * o[1], None))
        }
    }
}

fn shot_speed(w: &Weapon, t: &Target, c: f64) -> Option<f64> {
    match w {
        Weapon::Beam { .. } => Some(c),
        Weapon::Projectile { muzzle_m_s, .. } => Some(muzzle_m_s + t.closing_m_s.max(0.0)),
        Weapon::Burst { .. } => None,
    }
}

/// Furthest distance at which `f(d)` is still at least `goal`, with `f`
/// falling with distance; `None` when it never falls below (no limit).
fn furthest(f: &dyn Fn(f64) -> Result<f64, String>, goal: f64) -> Result<Option<f64>, String> {
    let (mut lo, mut hi) = (1e-3_f64, 1e16_f64);
    if f(hi)? >= goal {
        return Ok(None);
    }
    if f(lo)? < goal {
        return Ok(Some(0.0));
    }
    // Bisect in the logarithm: 80 halvings take 19 decades to well under a
    // part in a billion.
    for _ in 0..80 {
        let mid = m::sqrt(lo * hi);
        if f(mid)? >= goal {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(Some(lo))
}

/// With this weapon, against that target, at this distance: the reading.
pub fn effect_at_range(ask: &Ask, sheet: &Sheet) -> Result<Effect, String> {
    let c = sheet
        .constant("c")
        .ok_or("the sheet has no speed of light")?;
    if !(ask.distance_m.is_finite() && ask.distance_m > 0.0) {
        return Err(format!(
            "distance must be above zero, got {}",
            ask.distance_m
        ));
    }
    let (w, t) = (&ask.weapon, &ask.target);
    let mut why = vec![];
    let mut missing = vec![];
    let mut refs: Vec<&str> = vec![];
    let absorbed = t.skin.absorbed_share.unwrap_or(1.0);
    if t.skin.absorbed_share.is_none() {
        why.push("The skin is taken to absorb all that reaches it; give its share absorbed for a shiny or ablative skin.".into());
    }
    let family = match w {
        Weapon::Beam { .. } => "sse.range.beam_spot",
        Weapon::Burst { .. } => "sse.range.burst_fluence",
        Weapon::Projectile { .. } => "sse.range.impact_fluence",
    };
    refs.push(family);
    let (got, spot) = arrives(w, t, ask.distance_m, absorbed)?;

    // What the skin can take: the sheet first, a typed figure only if not.
    let mut takes = None;
    let mut takes_from = None;
    if let Some(id) = &t.skin.material {
        match skin_rows(sheet, id, &mut missing) {
            Some([rho, cp, hm, lf, hb, lv]) => {
                let t0 = t.skin.start_k.unwrap_or(300.0);
                let o = run(
                    "sse.range.burn_through",
                    &[rho, t.skin.thickness_m, cp, t0, hm, lf, hb, lv],
                )?;
                let melt = t.skin.effect == "melt";
                takes = Some(if melt { o[0] } else { o[1] });
                takes_from = Some("sheet".to_string());
                refs.push("sse.range.burn_through");
                why.push(format!(
                    "To {} through {:.3} m of {id} takes {:.3e} J/m² (heat, melt{}) from the sheet's heats, with no heat lost while it happens: the least it could take.",
                    if melt { "melt" } else { "boil" },
                    t.skin.thickness_m,
                    takes.unwrap_or(0.0),
                    if melt { "" } else { ", boil" }
                ));
            }
            None if sheet.row("mixtures", id).is_none()
                && sheet.row("substances", id).is_none() =>
            {
                return Err(format!("{id} is not a material on the sheet"));
            }
            None => why.push(format!(
                "The sheet does not yet hold every figure {id} needs ({}).",
                missing.join(", ")
            )),
        }
    }
    if takes.is_none() {
        if let Some(v) = t.skin.takes_j_m2 {
            takes = Some(v);
            takes_from = Some("authored".to_string());
            why.push(format!("What the target can take, {v:.3e} J/m², is a figure typed by hand, kept as authored."));
        }
    }

    // The margin factor is the reading block's own model number.
    let margin = run("sse.range.reading", &[1.0, 1.0])?[2];
    let (reading, ratio) = match takes {
        Some(n) if n > 0.0 => {
            let o = run("sse.range.reading", &[got, n])?;
            refs.push("sse.range.reading");
            (READINGS[o[1] as usize].to_string(), Some(o[0]))
        }
        _ => ("unknown".to_string(), None),
    };

    // The chance of a hit.
    let speed = shot_speed(w, t, c);
    let odds_at = |d: f64| -> Result<[f64; 3], String> {
        let v = speed.unwrap_or(c);
        let o = run(
            "sse.range.hit_odds",
            &[d, v, t.accel_m_s2.max(0.0), t.radius_m],
        )?;
        Ok([o[0], o[1], o[2]])
    };
    let (odds, delay, wander) = match speed {
        Some(_) => {
            let o = odds_at(ask.distance_m)?;
            refs.push("sse.range.hit_odds");
            why.push(format!(
                "The target is seen as it was and the shot takes time to arrive: {:.3} s in all, in which it could move {:.3e} m against its own {:.3e} m, so the chance of a hit is {:.0}%.",
                o[0], o[1], t.radius_m, o[2] * 100.0
            ));
            (Some(o[2]), Some(o[0]), Some(o[1]))
        }
        None => {
            why.push("A burst is read as if it arrived: whether the missile gets there is NAV's sum for any ship.".into());
            (None, None, None)
        }
    };

    // Rings: where the reading changes, and where the odds fall to even.
    let ratio_at = |d: f64| -> Result<f64, String> {
        let n = takes.unwrap_or(f64::NAN);
        Ok(arrives(w, t, d, absorbed)?.0 / n)
    };
    let falls = !matches!(w, Weapon::Projectile { .. });
    let ring = |goal: f64| -> Result<Option<f64>, String> {
        match takes {
            Some(n) if n > 0.0 && falls => furthest(&ratio_at, goal),
            _ => Ok(None),
        }
    };
    let rings = Rings {
        in_range_m: ring(margin)?,
        enough_m: ring(1.0)?,
        marginal_m: ring(1.0 / margin)?,
        even_odds_m: match speed {
            Some(_) => furthest(&|d| Ok(odds_at(d)?[2]), 0.5)?,
            None => None,
        },
    };

    // A beam on the power ledger.
    let load = match w {
        Weapon::Beam {
            beam_power_w,
            efficiency: Some(e),
            reject_k,
            ..
        } if *e > 0.0 => {
            let draw = beam_power_w / e;
            why.push(format!(
                "It draws {:.3e} W from the electric bus and puts {:.3e} W on the heat bus: a warship is radiators with a gun attached.",
                draw,
                draw - beam_power_w
            ));
            Some(sse_nav::power::Load {
                name: "beam weapon".into(),
                electric_w: draw,
                reject_k: *reject_k,
                mass_t: None,
                off: false,
            })
        }
        _ => None,
    };

    let mut references = vec![];
    for id in refs {
        for r in &block(id).decl().references {
            if !references.contains(r) {
                references.push(r.clone());
            }
        }
    }
    Ok(Effect {
        reading,
        arrives_j_m2: got,
        takes_j_m2: takes,
        takes_from,
        ratio,
        odds,
        delay_s: delay,
        wander_m: wander,
        spot_radius_m: spot,
        rings,
        load,
        missing,
        why,
        references,
    })
}
