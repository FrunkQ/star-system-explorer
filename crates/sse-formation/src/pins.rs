//! F9: pins and reconstruction (formation design section 6).
//!
//! A pinned world is honoured and the stages are solved around it, by a
//! deterministic search over candidate histories: the pipeline is run for a
//! fixed list of seeds derived from the manifest's own, each run is scored by
//! how close its worlds come to the pins, and the best is kept. The pins are
//! then written onto the worlds that matched them (or placed where nothing
//! matched), and generated worlds that would sit on top of a pinned one are
//! removed. Everything the search fixed is labelled: the history as
//! reconstructed when the pins came from outside (an imported system),
//! pinned values as authored.

use crate::early::{early_life, Protoplanet};
use crate::history::{FormationHistory, How, Manifest};
use crate::hurley::Fits;
use crate::pipeline::{self, Block, SystemRun};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Address;

/// A world the GM or an import fixed.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldPin {
    /// Name or source address, for the record.
    pub label: String,
    pub mass_earth: f64,
    pub a_au: f64,
    #[serde(default)]
    pub e: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pins {
    pub worlds: Vec<WorldPin>,
    /// Candidate histories searched (seeds), at least 1.
    pub tries: u32,
    /// The pins describe a system that exists elsewhere (an import): the
    /// history found for it is labelled reconstructed.
    #[serde(default)]
    pub reconstruct: bool,
}

/// How a pin was honoured.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PinMatch {
    pub pin: usize,
    pub world: Address,
    /// "matched" (a generated world moved onto the pin) or "placed" (none was near).
    pub how: String,
    /// The matched world's own mass and orbit before the pin was applied.
    #[serde(default)]
    pub was: Option<(f64, f64)>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PinnedRun {
    pub run: SystemRun,
    /// Which candidate history won (0 is the manifest's own seed).
    pub attempt: u32,
    pub seed: String,
    /// Score of the winner (0 is a perfect fit) and of the manifest's own seed.
    pub fit: f64,
    pub fit_first: f64,
    pub matches: Vec<PinMatch>,
    /// Generated worlds removed because they sat on top of a pinned one.
    pub removed: Vec<Address>,
}

/// A match closer than this (score units, about 1 sigma in log a and log m
/// combined) moves a generated world; worse, the pin is placed as new.
pub const MATCH_LIMIT: f64 = 9.0;
/// Generated worlds within this many mutual Hill radii of a pinned one go.
pub const CLEAR_HILL: f64 = 8.0;

/// Score of one world against one pin: log distance in orbit (0.1 dex scale)
/// and mass (0.5 dex scale), squared.
fn score(mass: f64, a: f64, pin: &WorldPin) -> f64 {
    let da = m::log10(a.max(1e-6) / pin.a_au) / 0.1;
    let dm = m::log10(mass.max(1e-9) / pin.mass_earth) / 0.5;
    da * da + dm * dm
}

/// Greedy assignment, pins in order of mass (heaviest first), each to the
/// best remaining world. Returns (total score, per pin the world index or None).
fn assign(protos: &[Protoplanet], gas: &[f64], pins: &[WorldPin]) -> (f64, Vec<Option<usize>>) {
    let mut order: Vec<usize> = (0..pins.len()).collect();
    order.sort_by(|&x, &y| {
        pins[y]
            .mass_earth
            .total_cmp(&pins[x].mass_earth)
            .then(x.cmp(&y))
    });
    let mut used = vec![false; protos.len()];
    let mut out = vec![None; pins.len()];
    let mut total = 0.0;
    for i in order {
        let mut best: Option<(f64, usize)> = None;
        for (j, p) in protos.iter().enumerate() {
            if used[j] {
                continue;
            }
            let s = score(p.mass + gas[j], p.a_au, &pins[i]);
            if best.is_none_or(|b| s < b.0) {
                best = Some((s, j));
            }
        }
        match best {
            Some((s, j)) if s <= MATCH_LIMIT => {
                used[j] = true;
                out[i] = Some(j);
                total += s;
            }
            _ => total += MATCH_LIMIT * 2.0,
        }
    }
    (total, out)
}

fn seed_for(base: &str, k: u32) -> String {
    if k == 0 {
        base.to_string()
    } else {
        format!("{base}#pin-search-{k}")
    }
}

/// Solve a system around its pins.
pub fn solve(
    manifest: Manifest,
    pins: &Pins,
    block: Block,
) -> Result<PinnedRun, crate::FormationError> {
    let base = manifest.seed.clone();
    let mut best: Option<(f64, u32, SystemRun, Vec<Option<usize>>)> = None;
    let mut fit_first = f64::INFINITY;
    for k in 0..pins.tries.max(1) {
        let mut man = manifest.clone();
        man.seed = seed_for(&base, k);
        let h = FormationHistory::form(man)?;
        let r = pipeline::run(h, block);
        let gas: Vec<f64> = r
            .worlds
            .iter()
            .zip(&r.protoplanets)
            .map(|(w, p)| {
                if w.envelope == crate::early::Envelope::Giant {
                    p.gas
                } else {
                    0.0
                }
            })
            .collect();
        let (s, a) = assign(&r.protoplanets, &gas, &pins.worlds);
        if k == 0 {
            fit_first = s;
        }
        if best.as_ref().is_none_or(|b| s < b.0) {
            best = Some((s, k, r, a));
        }
    }
    let (fit, attempt, mut run, assigned) = best.expect("at least one try");
    let system = run.history.manifest.system.clone();
    let host_mass = run.history.stars[0].mass;
    let fits = Fits::new(run.history.inheritance.z);
    let mut protos = run.protoplanets.clone();
    let mut matches = vec![];
    let mut pinned_idx = vec![];
    for (i, pin) in pins.worlds.iter().enumerate() {
        match assigned[i] {
            Some(j) => {
                let p = &mut protos[j];
                let was = Some((p.mass + p.gas, p.a_au));
                apply(p, pin);
                matches.push(PinMatch {
                    pin: i,
                    world: p.address.clone(),
                    how: "matched".into(),
                    was,
                });
                pinned_idx.push(j);
            }
            None => {
                // Placed: material from the generated world nearest in orbit.
                let near = protos
                    .iter()
                    .min_by(|x, y| {
                        m::ln(x.a_au / pin.a_au)
                            .abs()
                            .total_cmp(&m::ln(y.a_au / pin.a_au).abs())
                    })
                    .cloned();
                let n = protos.len() as u64 + matches.len() as u64;
                let address = system.child("pinned", n).expect("valid address");
                let mut p = near.unwrap_or(Protoplanet {
                    address: address.clone(),
                    mass: 1.0,
                    a_au: pin.a_au,
                    e: 0.0,
                    formed_myr: 50.0,
                    by_group: [0.32, 0.68, 0.0, 0.0, 0.0, 0.0],
                    gas: 0.0,
                });
                p.address = address.clone();
                apply(&mut p, pin);
                protos.push(p);
                pinned_idx.push(protos.len() - 1);
                matches.push(PinMatch {
                    pin: i,
                    world: address,
                    how: "placed".into(),
                    was: None,
                });
            }
        }
    }
    // Clear generated worlds sitting on top of pinned ones.
    let hill = |p: &Protoplanet, q: &Protoplanet| {
        let mu = (p.mass + p.gas + q.mass + q.gas) / (3.0 * 332_946.0 * host_mass);
        let rh = m::cbrt(mu) * 0.5 * (p.a_au + q.a_au);
        (p.a_au - q.a_au).abs() / rh
    };
    let mut keep = vec![true; protos.len()];
    for j in 0..protos.len() {
        if pinned_idx.contains(&j) {
            continue;
        }
        if pinned_idx
            .iter()
            .any(|&k| hill(&protos[j], &protos[k]) < CLEAR_HILL)
        {
            keep[j] = false;
        }
    }
    let removed: Vec<Address> = protos
        .iter()
        .zip(&keep)
        .filter(|(_, k)| !**k)
        .map(|(p, _)| p.address.clone())
        .collect();
    let protos: Vec<Protoplanet> = protos
        .into_iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|(p, _)| p)
        .collect();
    let mut protos = protos;
    protos.sort_by(|x, y| x.a_au.total_cmp(&y.a_au));
    // Early life and heat again for the final set.
    let host = &run.history.stars[0].life;
    let worlds = protos.iter().map(|p| early_life(p, &fits, host)).collect();
    run.protoplanets = protos;
    run.worlds = worlds;
    pipeline::rerun_heat(&mut run);
    let ev = &mut run.history.evidence;
    for i in 0..pins.worlds.len() {
        ev.insert(format!("pins.worlds.{i}"), How::Authored);
    }
    if pins.reconstruct {
        ev.insert("history".into(), How::Reconstructed);
    }
    run.notes.push(format!(
        "solved around {} pinned world(s): candidate history {attempt} of {} (fit {fit:.2}; the manifest's own seed scored {fit_first:.2}); {} generated world(s) removed to make room",
        pins.worlds.len(),
        pins.tries.max(1),
        removed.len()
    ));
    Ok(PinnedRun {
        seed: run.history.manifest.seed.clone(),
        run,
        attempt,
        fit,
        fit_first,
        matches,
        removed,
    })
}

/// Write a pin onto a protoplanet: mass split in the world's own proportions
/// (solids by group, gas), orbit set.
fn apply(p: &mut Protoplanet, pin: &WorldPin) {
    let total = p.mass + p.gas;
    let k = if total > 0.0 {
        pin.mass_earth / total
    } else {
        1.0
    };
    p.mass *= k;
    p.gas *= k;
    for g in p.by_group.iter_mut() {
        *g *= k;
    }
    p.a_au = pin.a_au;
    if let Some(e) = pin.e {
        p.e = e;
    }
}
