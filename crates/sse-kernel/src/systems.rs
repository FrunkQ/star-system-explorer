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
