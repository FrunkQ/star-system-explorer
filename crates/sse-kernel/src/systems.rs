//! Kernel ops backed by the physics track: `import_v3` (P3, F9) and
//! `run_formation` (formation pipeline, F9 pins, S0 surface envelopes).
//! Both are pure: the same request gives the same response on every device,
//! and nothing is written to an open universe.

use crate::err;
use crate::protocol::Response;
use sse_core::Address;
use sse_formation::history::{FormationHistory, Manifest, Source};
use sse_formation::pins::{self, Pins};
use sse_formation::pipeline::{self, Block};
use sse_surface::scenario as surface;

type Out = Result<Response, Response>;

fn to_json<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).expect("plain data")
}

pub(crate) fn import_v3(save: &str, universe_id: Option<&str>, reconstruct: Option<u32>) -> Out {
    let im = sse_import_v3::import_json(save, universe_id.unwrap_or("v3-import"))
        .map_err(|e| err("import_failed", e.0))?;
    let mut reconstructed = vec![];
    if let Some(tries) = reconstruct {
        let tries = tries.clamp(1, 64);
        for k in sse_import_v3::reconstruct::known_systems(&im) {
            match sse_import_v3::reconstruct::reconstruct(&k, tries) {
                Ok(r) => {
                    let mut v = sse_import_v3::reconstruct::summary(&k, &r);
                    v["run"] = to_json(&r.run);
                    v["matches"] = to_json(&r.matches);
                    reconstructed.push(v);
                }
                Err(e) => reconstructed.push(serde_json::json!({
                    "system": k.system.to_string(), "error": e,
                })),
            }
        }
    }
    let facts: serde_json::Map<String, serde_json::Value> = im
        .facts
        .iter()
        .map(|(a, f)| (a.to_string(), to_json(f)))
        .collect();
    Ok(Response::V3Imported {
        universe: im.universe.to_string(),
        facts: serde_json::Value::Object(facts),
        report: to_json(&im.report),
        reconstructed,
    })
}

pub(crate) struct FormArgs {
    pub seed: String,
    pub system: Option<String>,
    pub block: Option<String>,
    pub birth: Option<serde_json::Value>,
    pub now_gyr: Option<f64>,
    pub envelope: Option<serde_json::Value>,
    pub inherit_pins: Option<serde_json::Value>,
    pub star_pins: Option<serde_json::Value>,
    pub pins: Option<serde_json::Value>,
}

fn parse<T: serde::de::DeserializeOwned>(what: &str, v: serde_json::Value) -> Result<T, Response> {
    serde_json::from_value(v).map_err(|e| err("bad_request", format!("{what}: {e}")))
}

impl From<crate::protocol::FormationRequest> for FormArgs {
    fn from(r: crate::protocol::FormationRequest) -> Self {
        FormArgs {
            seed: r.seed,
            system: r.system,
            block: r.block,
            birth: r.birth,
            now_gyr: r.now_gyr,
            envelope: r.envelope,
            inherit_pins: r.inherit_pins,
            star_pins: r.star_pins,
            pins: r.pins,
        }
    }
}

pub(crate) fn run_formation(a: FormArgs) -> Out {
    let (system, run, pinned) = form(a)?;
    let s = surface::settings();
    let surfaces = sse_surface::envelopes_of(&run, &s)
        .iter()
        .map(to_json)
        .collect();
    Ok(Response::FormationRan {
        system: system.to_string(),
        pipeline: pipeline::PIPELINE_VERSION.to_string(),
        run: to_json(&run),
        pinned,
        surfaces,
    })
}

/// The pipeline run behind `run_formation` and `adopt`.
pub(crate) fn form(
    a: FormArgs,
) -> Result<(Address, pipeline::SystemRun, Option<serde_json::Value>), Response> {
    let system: Address = a
        .system
        .as_deref()
        .unwrap_or("u:generated/sys:0")
        .parse()
        .map_err(|e| err("bad_address", format!("{e}")))?;
    let block = match a.block.as_deref().unwrap_or("modern") {
        "modern" => Block::Modern,
        "classic" => Block::Classic,
        "classic-corrected" => Block::ClassicCorrected,
        other => {
            return Err(err(
                "bad_request",
                format!("block {other}: use modern, classic or classic-corrected"),
            ))
        }
    };
    let mut man = match (a.birth, a.envelope) {
        (Some(_), Some(_)) => {
            return Err(err("bad_request", "give a birth or an envelope, not both"))
        }
        (Some(b), None) => {
            let birth: sse_cosmos::SystemBirth = parse("birth", b)?;
            let mut m = Manifest::new(&a.seed, system.clone(), sse_formation::scenario::envelope());
            m.source = Source::Birth {
                birth,
                now_gyr: a.now_gyr.unwrap_or(13.8),
            };
            m
        }
        (None, Some(e)) => Manifest::new(&a.seed, system.clone(), parse("envelope", e)?),
        (None, None) => Manifest::new(&a.seed, system.clone(), sse_formation::scenario::envelope()),
    };
    if let Some(p) = a.inherit_pins {
        man.inherit_pins = parse("inherit_pins", p)?;
    }
    if let Some(p) = a.star_pins {
        man.star_pins = parse("star_pins", p)?;
    }
    let (run, pinned) = match a.pins {
        Some(p) => {
            let pins: Pins = parse("pins", p)?;
            let r = pins::solve(man, &pins, block).map_err(|e| err("formation_failed", e.0))?;
            let fit = serde_json::json!({
                "attempt": r.attempt, "seed": r.seed, "fit": r.fit, "fit_first": r.fit_first,
                "matches": to_json(&r.matches), "removed": to_json(&r.removed),
            });
            (r.run, Some(fit))
        }
        None => {
            let h = FormationHistory::form(man).map_err(|e| err("formation_failed", e.0))?;
            (pipeline::run(h, block), None)
        }
    };
    Ok((system, run, pinned))
}

/// A body's world stack, its ladder and physics notes (P4 fixtures for now).
pub(crate) fn read_world_stack(
    body: Option<&str>,
    fixture: Option<&str>,
    stack: Option<serde_json::Value>,
) -> Out {
    let names: Vec<&str> = sse_world::fixtures().iter().map(|f| f.0).collect();
    let (ws, source, raw) = if let Some(v) = stack {
        let ws: sse_world::WorldStack = serde_json::from_value(v)
            .map_err(|e| err("bad_world_stack", format!("not a world stack: {e}")))?;
        // Echo the stack as parsed (numbers as f64), so every caller gets the same bytes.
        let raw = serde_json::to_value(&ws).expect("plain data");
        (ws, "given".to_string(), raw)
    } else {
        let all: Vec<(&str, sse_world::WorldStack, &str)> = sse_world::fixtures()
            .iter()
            .map(|(n, t)| {
                (
                    *n,
                    sse_world::WorldStack::from_json(t).expect("fixture stacks parse"),
                    *t,
                )
            })
            .collect();
        let hit = all.into_iter().find(|(n, w, _)| {
            fixture.is_some_and(|f| f == *n)
                || (fixture.is_none() && body.is_some_and(|b| b == w.body))
        });
        match hit {
            Some((n, w, t)) => (
                w,
                format!("fixture:{n}"),
                serde_json::from_str(t).expect("fixture stacks parse"),
            ),
            None => {
                return Err(err(
                    "no_world_stack",
                    format!(
                        "no world stack for {}; fixtures: {}",
                        fixture.or(body).unwrap_or("(nothing asked)"),
                        names.join(", ")
                    ),
                ))
            }
        }
    };
    if ws.format != sse_world::FORMAT {
        return Err(err(
            "bad_world_stack",
            format!("format {} is not {}", ws.format, sse_world::FORMAT),
        ));
    }
    let issue = |i: &sse_world::Issue| serde_json::json!({"at": i.at, "message": i.message});
    let (notes, problems) = ws.physics_notes();
    Ok(Response::WorldStackRead {
        body: ws.body.clone(),
        source,
        ladder: ws
            .ladder()
            .iter()
            .map(|l| serde_json::to_value(l).expect("plain data"))
            .collect(),
        invalid: ws.validate().iter().map(issue).collect(),
        problems: problems.iter().map(issue).collect(),
        notes,
        stack: raw,
    })
}

/// Light from bodies (O2): the Solar System fixture or given conditions.
pub(crate) fn read_light(fixture: Option<&str>, bodies: Option<Vec<serde_json::Value>>) -> Out {
    let model = sse_light::emit::Model::base();
    let (source, conds) = match (fixture, bodies) {
        (_, Some(list)) => {
            let mut out = vec![];
            for (i, v) in list.into_iter().enumerate() {
                out.push(
                    serde_json::from_value::<sse_light::emit::Conditions>(v)
                        .map_err(|e| err("bad_conditions", format!("body {i}: {e}")))?,
                );
            }
            ("given".to_string(), out)
        }
        (Some("solar-system"), None) => (
            "fixture:solar-system".to_string(),
            sse_light::emit::solar_system().bodies,
        ),
        (f, None) => {
            return Err(err(
                "no_light",
                format!(
                    "no light for {}; fixtures: solar-system, or give bodies",
                    f.unwrap_or("(nothing asked)")
                ),
            ))
        }
    };
    let lit = sse_light::emit::publish_all(&conds, model).map_err(|e| err("bad_conditions", e))?;
    let t = &model.table;
    Ok(Response::LightRead {
        source,
        table: serde_json::json!({ "bands": t.bands, "visible_slices": t.visible_slices, "lines": t.lines }),
        bodies: lit
            .iter()
            .map(|b| serde_json::to_value(b).expect("plain data"))
            .collect(),
    })
}

fn parse_list<T: serde::de::DeserializeOwned>(
    what: &str,
    v: Option<Vec<serde_json::Value>>,
) -> Result<Vec<T>, Response> {
    v.unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            serde_json::from_value(x).map_err(|e| err("bad_nebula", format!("{what} {i}: {e}")))
        })
        .collect()
}

/// Nebulae at a date (O3).
pub(crate) fn read_nebulae(
    at_s: f64,
    seed: Option<&str>,
    deaths: Option<Vec<serde_json::Value>>,
    planetary: Option<Vec<serde_json::Value>>,
    remnants: Option<Vec<serde_json::Value>>,
    glows: Option<Vec<serde_json::Value>>,
) -> Out {
    use sse_nebula::*;
    let seed = sse_core::Seed::from_text(seed.unwrap_or("sse4"));
    let deaths: Vec<Death> = parse_list("death", deaths)?;
    let mut pns: Vec<PnInput> = deaths.iter().map(PnInput::from_death).collect();
    pns.extend(parse_list::<PnInput>("planetary", planetary)?);
    let snrs: Vec<SnrInput> = parse_list("remnant", remnants)?;
    let gl: Vec<GlowInput> = parse_list("glow", glows)?;
    let (mut nebulae, mut absent) = (vec![], vec![]);
    let mut put = |addr: &str, d: Result<Option<Descriptor>, String>| -> Result<(), Response> {
        match d.map_err(|e| err("bad_nebula", e))? {
            Some(d) => nebulae.push(serde_json::to_value(d).expect("plain data")),
            None => absent.push(addr.to_string()),
        }
        Ok(())
    };
    for p in &pns {
        put(
            &p.address,
            planetary_nebula(&seed, p, &PnSettings::default(), at_s),
        )?;
    }
    for p in &snrs {
        put(
            &p.address,
            supernova_remnant(&seed, p, &SnrSettings::default(), at_s),
        )?;
    }
    for p in &gl {
        put(&p.address, star_forming_glow(&seed, p, at_s))?;
    }
    Ok(Response::NebulaeRead {
        at_s,
        nebulae,
        absent,
    })
}
