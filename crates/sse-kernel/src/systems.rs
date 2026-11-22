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
    let mut v = im.to_value();
    Ok(Response::V3Imported {
        universe: im.universe.to_string(),
        facts: v["facts"].take(),
        report: v["report"].take(),
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

/// Sensing (O9).
pub(crate) fn detect(
    targets: Vec<serde_json::Value>,
    sensors: Vec<serde_json::Value>,
    radar: Option<serde_json::Value>,
) -> Out {
    #[derive(serde::Deserialize)]
    struct Lit {
        #[serde(default)]
        conditions: Option<sse_light::emit::Conditions>,
        #[serde(default)]
        light: Option<sse_light::LightDescription>,
    }
    #[derive(serde::Deserialize)]
    struct Near {
        address: String,
        #[serde(flatten)]
        lit: Lit,
        distance_m: f64,
        separation_rad: f64,
    }
    #[derive(serde::Deserialize)]
    struct T {
        address: String,
        #[serde(flatten)]
        lit: Lit,
        distance_m: f64,
        #[serde(default)]
        extinction_av: f64,
        #[serde(default)]
        neighbours: Vec<Near>,
    }
    let model = sse_light::emit::Model::base();
    let light_of = |l: Lit, what: &str| -> Result<sse_light::LightDescription, Response> {
        match (l.conditions, l.light) {
            (Some(c), _) => Ok(sse_light::emit::describe(&c, model)
                .map_err(|e| err("bad_conditions", e))?
                .0),
            (None, Some(d)) => Ok(d),
            (None, None) => Err(err(
                "bad_sensing",
                format!("{what}: give conditions or light"),
            )),
        }
    };
    let sensors: Vec<sse_sense::Sensor> = sensors
        .into_iter()
        .enumerate()
        .map(|(n, v)| {
            serde_json::from_value(v).map_err(|e| err("bad_sensing", format!("sensor {n}: {e}")))
        })
        .collect::<Result<_, _>>()?;
    let mut detections = vec![];
    for (n, v) in targets.into_iter().enumerate() {
        let t: T = serde_json::from_value(v)
            .map_err(|e| err("bad_sensing", format!("target {n}: {e}")))?;
        let mut neighbours = vec![];
        for nb in t.neighbours {
            neighbours.push(sse_sense::Neighbour {
                light: light_of(nb.lit, &nb.address)?,
                address: nb.address,
                distance_m: nb.distance_m,
                separation_rad: nb.separation_rad,
            });
        }
        let seen = sse_sense::Seen {
            light: light_of(t.lit, &t.address)?,
            address: t.address,
            distance_m: t.distance_m,
            extinction_av: t.extinction_av,
            neighbours,
        };
        for s in &sensors {
            let d = sse_sense::detect(&seen, s, &model.table).map_err(|e| err("bad_sensing", e))?;
            detections.push(serde_json::to_value(d).expect("plain data"));
        }
    }
    let radar = match radar {
        Some(v) => {
            let r: sse_sense::Radar =
                serde_json::from_value(v).map_err(|e| err("bad_sensing", format!("radar: {e}")))?;
            Some(serde_json::to_value(sse_sense::radar_ranges(&r)).expect("plain data"))
        }
        None => None,
    };
    Ok(Response::Detected { detections, radar })
}

/// Air optics (Rayleigh scattering) for planets' air.
pub(crate) fn read_air_optics(
    airs: Vec<serde_json::Value>,
    wavelengths_m: Option<Vec<f64>>,
) -> Out {
    let table = &sse_light::emit::Model::base().table;
    let wl = wavelengths_m.unwrap_or_else(|| vec![680e-9, 550e-9, 440e-9]);
    if wl.iter().any(|&l| !(1e-8..=1e-3).contains(&l)) {
        return Err(err(
            "bad_air",
            "wavelengths must lie between 10 nm and 1 mm",
        ));
    }
    let mut out = vec![];
    for (n, v) in airs.into_iter().enumerate() {
        let a: sse_light::air::Air =
            serde_json::from_value(v).map_err(|e| err("bad_air", format!("air {n}: {e}")))?;
        let o = sse_light::air::optics(&a, &wl, table).map_err(|e| err("bad_air", e))?;
        out.push(serde_json::to_value(o).expect("plain data"));
    }
    Ok(Response::AirOpticsRead { airs: out })
}

/// Spectrum as it reaches an observer (O12).
pub(crate) fn read_spectrum(
    targets: Vec<serde_json::Value>,
    observer: Option<serde_json::Value>,
) -> Out {
    use sse_light::observe::*;
    #[derive(serde::Deserialize)]
    struct In {
        #[serde(default)]
        conditions: Option<sse_light::emit::Conditions>,
        #[serde(default)]
        light: Option<sse_light::LightDescription>,
        #[serde(default)]
        radial_velocity_m_s: f64,
        #[serde(default)]
        photosphere_k: Option<f64>,
    }
    #[derive(serde::Deserialize)]
    struct T {
        address: String,
        #[serde(default)]
        name: Option<String>,
        distance_m: f64,
        components: Vec<In>,
    }
    let model = sse_light::emit::Model::base();
    let o: Observer = match observer {
        Some(v) => {
            serde_json::from_value(v).map_err(|e| err("bad_spectrum", format!("observer: {e}")))?
        }
        None => Observer {
            coverage: vec![],
            resolving_power: 5000.0,
        },
    };
    let mut spectra = vec![];
    for (n, v) in targets.into_iter().enumerate() {
        let t: T = serde_json::from_value(v)
            .map_err(|e| err("bad_spectrum", format!("target {n}: {e}")))?;
        let mut comps = vec![];
        for (k, c) in t.components.into_iter().enumerate() {
            let (light, photo) = match (c.conditions, c.light) {
                (Some(cond), _) => {
                    let (d, facts) = sse_light::emit::describe(&cond, model)
                        .map_err(|e| err("bad_conditions", e))?;
                    // A glowing photosphere (2000 K or hotter) shows absorption lines.
                    let tk = facts.effective_temperature_k;
                    (
                        d,
                        c.photosphere_k
                            .or(if cond.own_luminosity_w > 0.0 && tk >= 2000.0 {
                                Some(tk)
                            } else {
                                None
                            }),
                    )
                }
                (None, Some(d)) => (d, c.photosphere_k),
                (None, None) => {
                    return Err(err(
                        "bad_spectrum",
                        format!("target {n} component {k}: give conditions or light"),
                    ))
                }
            };
            comps.push(Component {
                light,
                radial_velocity_m_s: c.radial_velocity_m_s,
                photosphere_k: photo,
            });
        }
        let tg = Target {
            address: t.address,
            name: t.name,
            distance_m: t.distance_m,
            components: comps,
        };
        let s = observe(&tg, &o, &model.table).map_err(|e| err("bad_spectrum", e))?;
        spectra.push(serde_json::to_value(s).expect("plain data"));
    }
    Ok(Response::SpectrumRead { spectra })
}

/// What stops the wind at each body (MF6 part 1), its shielding (MF7) and
/// belts (MF7 part 2, with the electron's constants from the data sheet).
pub(crate) fn read_field(bodies: Vec<serde_json::Value>, sheet: &sse_reference::Reference) -> Out {
    let electron = match (
        sheet.constant("m_e"),
        sheet.constant("e"),
        sheet.constant("eps0"),
    ) {
        (Some(mass_kg), Some(charge_c), Some(eps0)) => Some(sse_field::Electron {
            mass_kg,
            charge_c,
            eps0,
        }),
        _ => None,
    };
    let set = sse_field::Settings {
        electron,
        ..sse_field::Settings::default()
    };
    let mut out = vec![];
    for (n, v) in bodies.into_iter().enumerate() {
        let b: sse_field::Body =
            serde_json::from_value(v).map_err(|e| err("bad_field", format!("body {n}: {e}")))?;
        let f = sse_field::read(&b, &set).map_err(|e| err("bad_field", e))?;
        out.push(serde_json::to_value(f).expect("serialisable"));
    }
    Ok(Response::FieldRead { bodies: out })
}

/// Magnitudes in photometric bands, through dust (night sky Y2, light side).
pub(crate) fn read_photometry(stars: Vec<serde_json::Value>, bands: Vec<String>) -> Out {
    use sse_light::photometry::*;
    #[derive(serde::Deserialize)]
    struct In {
        #[serde(flatten)]
        photosphere: Photosphere,
        #[serde(flatten)]
        sight: Sight,
    }
    let table = sse_light::LightTable::base();
    let pb = bands_of(&table, &bands).map_err(|e| err("bad_photometry", e))?;
    let mut out = vec![];
    for (n, v) in stars.into_iter().enumerate() {
        let s: In = serde_json::from_value(v)
            .map_err(|e| err("bad_photometry", format!("star {n}: {e}")))?;
        let p = star_photometry(&s.photosphere, &s.sight, &pb)
            .map_err(|e| err("bad_photometry", format!("star {n}: {e}")))?;
        out.push(serde_json::to_value(p).expect("plain data"));
    }
    Ok(Response::PhotometryRead {
        stars: out,
        references: references()
            .into_iter()
            .map(|r| serde_json::to_value(r).expect("plain data"))
            .collect(),
    })
}

/// Stellar activity (O10).
pub(crate) fn read_activity(
    stars: Vec<serde_json::Value>,
    seed: Option<&str>,
    at_s: Option<f64>,
    window: Option<serde_json::Value>,
) -> Out {
    use sse_activity::*;
    #[derive(serde::Deserialize)]
    struct Item {
        #[serde(flatten)]
        star: Star,
        #[serde(default)]
        targets: Vec<Target>,
    }
    #[derive(serde::Deserialize)]
    struct Window {
        from_s: f64,
        to_s: f64,
        #[serde(default = "min_e")]
        min_energy_j: f64,
    }
    fn min_e() -> f64 {
        1e25
    }
    let seed = sse_core::Seed::from_text(seed.unwrap_or("sse4"));
    let set = Settings::default();
    let window: Option<Window> = window
        .map(|w| serde_json::from_value(w).map_err(|e| err("bad_activity", format!("window: {e}"))))
        .transpose()?;
    let mut out = vec![];
    for (n, v) in stars.into_iter().enumerate() {
        let it: Item =
            serde_json::from_value(v).map_err(|e| err("bad_activity", format!("star {n}: {e}")))?;
        let a = activity(&seed, &it.star, &set).map_err(|e| err("bad_activity", e))?;
        let mo = moment(&a, at_s.unwrap_or(it.star.age_s), &set);
        let fl = match &window {
            Some(w) => Some(
                flares(
                    &seed,
                    &a,
                    w.from_s,
                    w.to_s,
                    w.min_energy_j,
                    &it.targets,
                    &set,
                )
                .map_err(|e| err("bad_activity", e))?,
            ),
            None => None,
        };
        out.push(serde_json::json!({ "activity": a, "moment": mo, "flares": fl }));
    }
    Ok(Response::ActivityRead { stars: out })
}

/// Planet finding from afar (O13).
pub(crate) fn survey_planets(survey: serde_json::Value) -> Out {
    let sv: sse_survey::Survey =
        serde_json::from_value(survey).map_err(|e| err("bad_survey", e.to_string()))?;
    if sv.star.mass_kg <= 0.0 || sv.observer.distance_m <= 0.0 || sv.watch_s < 0.0 {
        return Err(err(
            "bad_survey",
            "star mass and observer distance must be positive, watch time not negative",
        ));
    }
    let result = sse_survey::run(&sv);
    Ok(Response::PlanetsSurveyed {
        result: serde_json::to_value(result).expect("plain data"),
    })
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
