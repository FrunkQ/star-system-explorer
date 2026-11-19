//! Golden scenario, run natively and in every browser by the harness: a
//! fixture set of systems formed from the stand-in envelope.

use crate::history::{FormationHistory, Manifest};
use crate::hurley::Fits;
use crate::inherit::PatchEnvelope;
use sse_core::Address;

pub const ENVELOPE_JSON: &str =
    include_str!("../../../tests/fixtures/formation/solar-neighbourhood.json");

pub fn envelope() -> PatchEnvelope {
    PatchEnvelope::from_json(ENVELOPE_JSON).expect("fixture envelope is valid")
}

pub fn system(n: u64) -> Address {
    Address::universe("f0")
        .expect("valid")
        .child("sys", n)
        .expect("valid")
}

/// 24 systems: their history hashes, star counts and a few slice values.
pub fn golden() -> serde_json::Value {
    let env = envelope();
    let mut out = vec![];
    for n in 0..24 {
        let h = FormationHistory::form(Manifest::new("f0-golden", system(n), env.clone()));
        out.push(match h {
            Ok(h) => serde_json::json!({
                "system": n,
                "stars": h.stars.len(),
                "primary_mass": h.stars[0].mass,
                "events": h.events.len(),
                "slices": h.slices.len(),
                "hash": h.hash().hex(),
            }),
            Err(e) => serde_json::json!({ "system": n, "refused": e.0 }),
        });
    }
    serde_json::json!({ "systems": out, "solar_patch": solar_patch_digest() })
}

/// Every system of G0's solar patch (the Sun's neighbourhood now), formed
/// from its birth facts. Returns the histories in patch order, refusals as
/// `Err` with their reasons.
pub fn solar_patch() -> Vec<Result<FormationHistory, crate::FormationError>> {
    use sse_cosmos::{form_patch, PatchModel, Site, SystemBirth};
    let model = PatchModel::milky_way_like();
    let site = Site::solar(&model);
    let mut c = sse_chronicle::Chronicle::new();
    let formed = form_patch(
        &mut c,
        &sse_cosmos::scenario::seed(),
        &model,
        sse_cosmos::scenario::address(),
        &site,
    )
    .expect("the G0 fixture patch forms");
    formed
        .systems
        .iter()
        .map(|(a, r)| {
            let birth = SystemBirth::from_facts(a, &r.facts).expect("G0 systems carry birth facts");
            FormationHistory::form(Manifest::from_birth("g0", birth, site.epoch_gyr()))
        })
        .collect()
}

/// Digest of the solar patch's formation: counts and one hash over all histories.
pub fn solar_patch_digest() -> serde_json::Value {
    let all = solar_patch();
    let mut text = String::new();
    let (mut ok, mut refused, mut multiple, mut wd, mut bd) = (0, 0, 0, 0, 0);
    for h in &all {
        match h {
            Ok(h) => {
                ok += 1;
                let now = h.state_at(h.inheritance.age_gyr * 1000.0);
                multiple += usize::from(h.stars.len() > 1);
                wd += usize::from(now.iter().any(|s| s.phase == crate::Phase::WhiteDwarf));
                bd += usize::from(now[0].phase == crate::Phase::BrownDwarf);
                text.push_str(&h.hash().hex());
            }
            Err(e) => {
                refused += 1;
                text.push_str(&e.0);
            }
        }
        text.push('\n');
    }
    serde_json::json!({
        "formed": ok,
        "refused": refused,
        "multiple": multiple,
        "with_white_dwarf_now": wd,
        "brown_dwarf_primaries": bd,
        "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
    })
}

/// F6 reference worlds round the Sun. A world of `mass` Earth masses with solar-mix solids from inside the water
/// line (`wet` adds water-ice band material as a mass fraction).
pub fn world(n: u64, mass: f64, a_au: f64, wet: f64, gas: f64) -> crate::early::Protoplanet {
    // Metal to rock in the solar mix; carbon at bulk Earth's 730 ppm
    // (McDonough 2003, Treatise on Geochemistry 2, 547).
    let x = crate::disc::mix(0.0, 0.0, 0.55);
    let carbon = 7.3e-4;
    let mut by = [0.0; 6];
    by[0] = mass * (1.0 - wet) * (1.0 - carbon) * x[0] / (x[0] + x[1]);
    by[1] = mass * (1.0 - wet) * (1.0 - carbon) * x[1] / (x[0] + x[1]);
    by[2] = mass * (1.0 - wet) * carbon;
    by[3] = mass * wet;
    crate::early::Protoplanet {
        address: system(1).child("body", n).expect("valid"),
        mass,
        a_au,
        e: 0.02,
        formed_myr: 50.0,
        by_group: by,
        gas,
    }
}

/// F6 golden: Earth, Venus, Mars, a stripped and a kept 5 Earth-mass world, a water world.
pub fn early_golden() -> serde_json::Value {
    let f = Fits::new(0.02);
    let l = crate::lifeline::LifeLine::new(&f, 1.0).expect("the Sun");
    let worlds = [
        world(3, 1.0, 1.0, 0.001, 0.0),
        world(2, 0.815, 0.723, 0.001, 0.0),
        world(4, 0.107, 1.524, 0.001, 0.0),
        world(1, 5.0, 0.05, 0.0, 0.15),
        world(5, 5.0, 0.5, 0.0, 0.15),
        world(6, 2.0, 1.2, 0.5, 0.0),
    ];
    let out: Vec<_> = worlds
        .iter()
        .map(|p| crate::early::early_life(p, &f, &l))
        .collect();
    serde_json::to_value(out).expect("serialisable")
}

/// F2 golden: the Classic Accrete block on System Lab's default solar request
/// and on SSE4 seeds, summarised (bodies, masses, ledger sizes).
pub fn accrete_golden() -> serde_json::Value {
    use crate::accrete::{run, Controls, Request};
    let mut out = vec![];
    for (seed, mass, l) in [
        ("golden-system-lab-solar", 1.0, 1.0),
        ("u:f0/sys:1", 1.0, 1.0),
        ("u:f0/sys:2", 0.8, 0.35),
        ("u:f0/sys:3", 0.31, 0.015),
    ] {
        let r = run(&Request {
            master_seed: seed.into(),
            star_mass: mass,
            star_luminosity: l,
            controls: Controls::DEFAULT,
            rule: crate::accrete::Rule::SystemLab,
        })
        .expect("valid request");
        let text = serde_json::to_string(&r).expect("serialisable");
        out.push(serde_json::json!({
            "seed": seed,
            "iterations": r.iterations,
            "bodies": r.bodies.iter().map(|b| serde_json::json!({
                "id": b.id, "a_au": b.a_au, "earth_mass": b.earth_mass(), "gas_giant": b.gas_giant,
                "ledger_entries": b.ledger.entries.len(),
            })).collect::<Vec<_>>(),
            "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
        }));
    }
    serde_json::Value::Array(out)
}

/// F3 golden: the modern growth block on six Sun-like discs.
pub fn growth_golden() -> serde_json::Value {
    let seed = sse_core::Seed::from_text("f3-golden");
    let fits = Fits::new(0.02);
    let mut out = vec![];
    for n in 0..6u64 {
        let sys = system(n);
        let disc = crate::disc::Disc::new(crate::disc::disc_params(
            &seed,
            &sys,
            1.0,
            fits.l_zams(1.0),
            0.0,
            0.0,
            0.55,
            300.0,
            None,
        ));
        let g = crate::growth::grow(&seed, &sys, &disc);
        let text = serde_json::to_string(&g).expect("serialisable");
        out.push(serde_json::json!({
            "system": n,
            "bodies": g.bodies.iter().map(|b| serde_json::json!({
                "id": b.id, "a_au": b.a_au, "solid_earth": b.solid_earth(), "gas_earth": b.gas_earth(),
            })).collect::<Vec<_>>(),
            "chains": g.chains().len(),
            "events": g.events.len(),
            "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
        }));
    }
    serde_json::Value::Array(out)
}

/// F4 golden: the violent phase on the six F3 golden systems.
pub fn upheaval_golden() -> serde_json::Value {
    let seed = sse_core::Seed::from_text("f3-golden");
    let fits = Fits::new(0.02);
    let mut out = vec![];
    for n in 0..6u64 {
        let sys = system(n);
        let disc = crate::disc::Disc::new(crate::disc::disc_params(
            &seed,
            &sys,
            1.0,
            fits.l_zams(1.0),
            0.0,
            0.0,
            0.55,
            300.0,
            None,
        ));
        let g = crate::growth::grow(&seed, &sys, &disc);
        let u = crate::violent::upheave(&seed, &sys, &disc, &g);
        let text = serde_json::to_string(&u).expect("serialisable");
        out.push(serde_json::json!({
            "system": n,
            "break_myr": u.break_myr,
            "worlds": u.worlds.iter().map(|w| serde_json::json!({
                "id": w.id, "a_au": w.a_au, "e": w.e, "mass_earth": w.mass_earth(), "impacts": w.impacts,
            })).collect::<Vec<_>>(),
            "rogues": u.rogues.iter().map(|r| r.address.to_string()).collect::<Vec<_>>(),
            "events": u.events.len(),
            "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
        }));
    }
    serde_json::Value::Array(out)
}

/// F5 golden: the leftovers of the six F3 golden systems, with the first
/// objects of every cloud resolved (the same on every device).
pub fn leftovers_golden() -> serde_json::Value {
    let seed = sse_core::Seed::from_text("f3-golden");
    let fits = Fits::new(0.02);
    let mut out = vec![];
    for n in 0..6u64 {
        let sys = system(n);
        let disc = crate::disc::Disc::new(crate::disc::disc_params(
            &seed,
            &sys,
            1.0,
            fits.l_zams(1.0),
            0.0,
            0.0,
            0.55,
            300.0,
            None,
        ));
        let g = crate::growth::grow(&seed, &sys, &disc);
        let u = crate::violent::upheave(&seed, &sys, &disc, &g);
        let l = crate::leftovers::leftovers(&seed, &sys, &disc, &u);
        let text = serde_json::to_string(&l).expect("serialisable");
        out.push(serde_json::json!({
            "system": n,
            "clouds": l.clouds.iter().map(|c| serde_json::json!({
                "address": c.address.to_string(), "kind": c.kind, "mass_earth": c.mass_earth(),
                "largest_km": c.largest_km,
                "first": (1..=3).map(|k| c.resolve(&seed, k)).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "moons": l.moons.iter().map(|mo| serde_json::json!({
                "address": mo.address.to_string(), "origin": mo.origin, "diameter_km": mo.diameter_km,
            })).collect::<Vec<_>>(),
            "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
        }));
    }
    serde_json::Value::Array(out)
}

/// F2 corrected rule (`accrete-dole1970-v1`) golden: G0's solar-patch systems,
/// each driven by its own inheritance.
pub fn accrete_dole_golden() -> serde_json::Value {
    let mut out = vec![];
    for (i, h) in solar_patch().into_iter().enumerate().take(8) {
        let Ok(h) = h else { continue };
        let req = crate::accrete::Request::from_history(&h, &format!("dole-golden-{i}"));
        let r = crate::accrete::run(&req).expect("valid request");
        let text = serde_json::to_string(&r).expect("serialisable");
        out.push(serde_json::json!({
            "system": h.manifest.system.to_string(),
            "feh": h.inheritance.feh,
            "star_mass": req.star_mass,
            "dust_a": req.controls.A,
            "bodies": r.bodies.iter().map(|b| serde_json::json!({
                "a_au": b.a_au, "earth_mass": b.earth_mass(), "gas_giant": b.gas_giant,
            })).collect::<Vec<_>>(),
            "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
        }));
    }
    serde_json::Value::Array(out)
}

/// Formation pipeline golden: four G0 solar-patch systems through every stage
/// (modern block) and the planet-heat flow.
pub fn pipeline_golden() -> serde_json::Value {
    let out: Vec<_> = solar_patch()
        .into_iter()
        .flatten()
        .take(4)
        .map(|h| {
            let r = crate::pipeline::run(h, crate::pipeline::Block::Modern);
            let text = serde_json::to_string(&r).expect("serialisable");
            serde_json::json!({
                "system": r.history.manifest.system.to_string(),
                "worlds": r.worlds.iter().map(|w| serde_json::json!({
                    "address": w.address.to_string(), "mass_earth": w.mass_earth, "envelope": w.envelope,
                })).collect::<Vec<_>>(),
                "heat": r.heat.iter().map(|x| serde_json::json!({
                    "address": x.address.to_string(), "power_w": x.radiogenic_power_w,
                    "until_gyr": x.active_geology_until_gyr,
                })).collect::<Vec<_>>(),
                "events": r.events.len(),
                "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
            })
        })
        .collect();
    serde_json::Value::Array(out)
}
