//! Kernel ops that bring whole systems into an open universe and read them
//! back (asked for by the UI track): `adopt` (an SSE 3 import or a formation
//! run as a handful of events), the aged-state write-back of
//! `commit_look_ahead`, and `resolved_system` (facts, a SystemSpec for
//! `look_ahead`, S0 envelopes and S1 tiles).
//!
//! Field names and units follow the importer (`docs/v3-save-fields.md`): SI,
//! angles in radians, `orbit.host` and `parent` as address text.

use crate::err;
use crate::protocol::{FormationRequest, Response, TileRequest};
use crate::systems;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::quantity::consts::{AU, JULIAN_YEAR, L_SUN, R_EARTH, R_SUN};
use sse_core::{Address, ContentHash, Seed};
use sse_formation::pipeline::SystemRun;
use sse_time::epoch::{PlanetSpec, SystemSpec};
use std::collections::BTreeMap;

/// Event kinds written here.
pub const ADOPT_V3: &str = "adopt.v3";
pub const ADOPT_FORMATION: &str = "adopt.formation";
pub const TIME_AGED: &str = "time.aged";

/// Masses as the importer uses them.
pub const M_SUN_KG: f64 = 1.988_47e30;

/// The serde name of a plain enum value (kebab-case labels).
fn json_name<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|x| x.as_str().map(str::to_string))
        .unwrap_or_default()
}
pub const M_EARTH_KG: f64 = 5.972_2e24;
/// Metal mass fraction assumed when a system has no `star.metallicity_z` fact.
pub const DEFAULT_Z: f64 = 0.0142;
/// Most tiles one `resolved_system` builds.
pub const MAX_TILES: usize = 24;

type Facts = BTreeMap<Address, BTreeMap<String, Value>>;

fn f(x: f64) -> Value {
    Value::Float(x)
}
fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}
fn deg(x: f64) -> f64 {
    x * (m::PI / 180.0)
}
fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}
fn as_text(v: &Value) -> Option<&str> {
    match v {
        Value::Text(s) => Some(s),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn event(
    ch: &Chronicle,
    kind: &str,
    subject: Vec<Address>,
    at: UniverseTime,
    set: Facts,
    data: Option<Value>,
    causes: Vec<ContentHash>,
    evidence: Evidence,
    publisher: Publisher,
) -> Result<Event, Response> {
    Event::new(EventBody {
        kind: kind.into(),
        subject,
        occurred_at: at,
        activated_at: at,
        recognised_at: ch.next_stamp(),
        causes,
        payload: Payload {
            set,
            data,
            ..Default::default()
        },
        evidence,
        publisher,
    })
    .map_err(|e| err("bad_event", format!("{e}")))
}

fn json_value(j: &serde_json::Value) -> Result<Value, Response> {
    serde_json::from_value(j.clone()).map_err(|e| err("bad_value", e.to_string()))
}

// ------------------------------------------------------------------ adopt

/// What `adopt` writes, before it is appended.
pub(crate) struct Adoption {
    pub events: Vec<Event>,
    pub systems: Vec<String>,
    pub facts: u32,
    /// Formation adoptions: the run, cached for `resolved_system`.
    pub run: Option<(String, SystemRun)>,
}

fn universe_key(root: &Address) -> Result<String, Response> {
    if root.segments().len() != 1 {
        return Err(err(
            "bad_address",
            format!("adopt needs the universe's root, not {root}"),
        ));
    }
    Ok(root.last().key().to_string())
}

fn refuse_existing(ch: &Chronicle, systems: &[Address]) -> Result<(), Response> {
    let st = ch.state_at(UniverseTime::seconds(i64::MAX));
    for s in systems {
        if st.facts.keys().any(|a| s.contains(a)) {
            return Err(err(
                "already_adopted",
                format!("{s} already has facts in this universe"),
            ));
        }
    }
    Ok(())
}

pub(crate) fn adopt(
    ch: &Chronicle,
    root: &Address,
    import_v3: Option<String>,
    formation: Option<FormationRequest>,
    occurred_at_s: Option<i64>,
) -> Result<Adoption, Response> {
    let at = UniverseTime::seconds(occurred_at_s.unwrap_or(0));
    match (import_v3, formation) {
        (Some(save), None) => adopt_v3(ch, root, &save, at),
        (None, Some(req)) => adopt_formation(ch, root, req, at),
        _ => Err(err(
            "bad_request",
            "give one of import_v3 (a save's text) or formation (run_formation fields)",
        )),
    }
}

fn adopt_v3(
    ch: &Chronicle,
    root: &Address,
    save: &str,
    at: UniverseTime,
) -> Result<Adoption, Response> {
    let im = sse_import_v3::import_json(save, &universe_key(root)?)
        .map_err(|e| err("import_failed", e.0))?;
    let systems: Vec<Address> = im
        .facts
        .keys()
        .filter(|a| a.segments().len() == 2 && a.last().kind() == "sys")
        .cloned()
        .collect();
    refuse_existing(ch, &systems)?;
    let subject = if systems.is_empty() {
        vec![root.clone()]
    } else {
        systems.clone()
    };
    let mut events: Vec<Event> = vec![];
    let mut count = 0u32;
    let mut scratch = ch.clone();
    for ev in [
        Evidence::Authored,
        Evidence::Catalogued,
        Evidence::AuthoredImpossible,
    ] {
        let mut set: Facts = BTreeMap::new();
        for (a, fields) in &im.facts {
            let mut row = BTreeMap::new();
            for (k, fact) in fields.iter().filter(|(_, x)| x.evidence == ev) {
                row.insert(k.clone(), json_value(&fact.value)?);
            }
            if !row.is_empty() {
                count += row.len() as u32;
                set.insert(a.clone(), row);
            }
        }
        if set.is_empty() && !events.is_empty() {
            continue;
        }
        let first = events.first().map(|e| e.id);
        let data = match first {
            None => Some(json_value(&serde_json::json!({
                "source": "v3",
                "report": serde_json::to_value(&im.report).expect("plain data"),
            }))?),
            Some(_) => None,
        };
        let e = event(
            &scratch,
            ADOPT_V3,
            subject.clone(),
            at,
            set,
            data,
            first.into_iter().collect(),
            ev,
            Publisher::new("sse.import-v3", "adopt", sse_import_v3::IMPORTER_VERSION),
        )?;
        scratch
            .append(e.clone())
            .map_err(|x| err("chronicle", format!("{x}")))?;
        events.push(e);
    }
    Ok(Adoption {
        events,
        systems: subject.iter().map(|a| a.to_string()).collect(),
        facts: count,
        run: None,
    })
}

/// Facts of a formation run, in the importer's field names. Orbit angles are
/// the ones `sse-time` draws for the run (`spec_from_run`), so a look-ahead of
/// the adopted system starts from exactly these orbits.
pub(crate) fn run_facts(run: &SystemRun) -> Facts {
    let h = &run.history;
    let spec = sse_time::generated::spec_from_run(run);
    let mut out: Facts = BTreeMap::new();
    let sys = h.manifest.system.clone();
    let mut row = BTreeMap::new();
    row.insert(
        "system.age".to_string(),
        f(spec.age_myr * 1.0e6 * JULIAN_YEAR),
    );
    row.insert(
        "system.birth_year".to_string(),
        Value::Int(i128::from(spec.birth_year)),
    );
    row.insert("star.metallicity_z".to_string(), f(h.inheritance.z));
    row.insert("system.pipeline".to_string(), t(&run.pipeline));
    out.insert(sys.clone(), row);
    let fits = sse_formation::hurley::Fits::new(h.inheritance.z);
    let tags = sse_formation::stars::stability(&h.stars);
    for (k, star) in h.stars.iter().enumerate() {
        let s = star.life.state_at(&fits, spec.age_myr);
        let mut row = BTreeMap::new();
        row.insert("body.kind".to_string(), t("star"));
        row.insert("parent".to_string(), t(&sys.to_string()));
        row.insert("body.mass".to_string(), f(s.mass * M_SUN_KG));
        row.insert("body.radius".to_string(), f(s.radius * R_SUN));
        row.insert("star.effective_temperature".to_string(), f(s.teff));
        row.insert("star.luminosity".to_string(), f(s.luminosity * L_SUN));
        row.insert("star.phase".to_string(), t(s.phase.label()));
        let own =
            sse_formation::body::self_light(&fits, star.life.initial_mass, spec.age_myr, None);
        row.insert("body.light_source".to_string(), t(&json_name(&own.source)));
        if let Some(tag) = &tags[k] {
            let word = if tag.stable { "stable" } else { "unstable" };
            row.insert("orbit.stability".to_string(), t(word));
            row.insert("orbit.stability_why".to_string(), t(&tag.why));
        }
        if k > 0 {
            if let Some(o) = &star.orbit {
                row.insert("orbit.host".to_string(), t(&h.stars[0].address.to_string()));
                row.insert(
                    "orbit.semi_major_axis".to_string(),
                    f(o.semi_major_axis_au * AU),
                );
                row.insert("orbit.eccentricity".to_string(), f(o.eccentricity));
            }
        }
        out.insert(star.address.clone(), row);
    }
    let host = h.stars[0].address.to_string();
    for (w, p) in run.worlds.iter().zip(&run.protoplanets) {
        let Some(ps) = spec
            .planets
            .iter()
            .find(|x| x.address == p.address.to_string())
        else {
            continue;
        };
        let mut row = BTreeMap::new();
        row.insert("body.kind".to_string(), t("planet"));
        row.insert("parent".to_string(), t(&host));
        row.insert("orbit.host".to_string(), t(&host));
        row.insert("body.mass".to_string(), f(ps.mass_me * M_EARTH_KG));
        row.insert("body.radius".to_string(), f(w.radius * R_EARTH));
        row.insert("body.solid_radius".to_string(), f(w.solid_radius * R_EARTH));
        row.insert("orbit.semi_major_axis".to_string(), f(ps.a_au * AU));
        row.insert("orbit.eccentricity".to_string(), f(ps.e));
        row.insert("orbit.inclination".to_string(), f(deg(ps.i_deg)));
        row.insert("orbit.ascending_node".to_string(), f(deg(ps.node_deg)));
        row.insert(
            "orbit.argument_of_periapsis".to_string(),
            f(deg(ps.varpi_deg - ps.node_deg)),
        );
        row.insert(
            "orbit.mean_anomaly_at_epoch".to_string(),
            f(deg(ps.mean_longitude_deg - ps.varpi_deg)),
        );
        row.insert("body.oceans".to_string(), Value::Bool(ps.oceans));
        // B1: the same light rule as the stars, on one mass axis. Giant
        // planets glow with the heat of their formation; rocky worlds do not.
        let own = sse_formation::body::self_light(
            &fits,
            ps.mass_me * M_EARTH_KG / M_SUN_KG,
            spec.age_myr,
            Some(w.radius * R_EARTH / R_SUN),
        );
        if own.luminosity_lsun > 0.0 {
            row.insert(
                "star.luminosity".to_string(),
                f(own.luminosity_lsun * L_SUN),
            );
            row.insert("star.effective_temperature".to_string(), f(own.teff_k));
            row.insert("body.light_source".to_string(), t(&json_name(&own.source)));
        }
        out.insert(p.address.clone(), row);
    }
    out
}

fn adopt_formation(
    ch: &Chronicle,
    root: &Address,
    req: FormationRequest,
    at: UniverseTime,
) -> Result<Adoption, Response> {
    let mut req = req;
    let system: Address = match &req.system {
        Some(s) => s.parse().map_err(|e| err("bad_address", format!("{e}")))?,
        None => root
            .child("sys", 0)
            .map_err(|e| err("bad_address", format!("{e}")))?,
    };
    if !root.contains(&system) || system == *root {
        return Err(err(
            "bad_address",
            format!("{system} is not a system in universe {root}"),
        ));
    }
    req.system = Some(system.to_string());
    refuse_existing(ch, std::slice::from_ref(&system))?;
    let (_, run, pinned) = systems::form(req.clone().into())?;
    let set = run_facts(&run);
    let count = set.values().map(|r| r.len() as u32).sum();
    let data = json_value(&serde_json::json!({
        "source": "formation",
        "request": serde_json::to_value(&req).expect("plain data"),
        "pipeline": run.pipeline,
        "block": serde_json::to_value(run.block).expect("plain data"),
        "manifest": run.history.manifest_hash.to_string(),
        "worlds": run.worlds.len(),
        "stars": run.history.stars.len(),
        "notes": run.notes,
        "pinned": pinned,
    }))?;
    let e = event(
        ch,
        ADOPT_FORMATION,
        vec![system.clone()],
        at,
        set,
        Some(data),
        vec![],
        Evidence::Resolved,
        Publisher::new(
            "sse.formation",
            "adopt",
            sse_formation::pipeline::PIPELINE_VERSION,
        ),
    )?;
    Ok(Adoption {
        events: vec![e],
        systems: vec![system.to_string()],
        facts: count,
        run: Some((system.to_string(), run)),
    })
}

// ------------------------------------------------------------- write-back

/// The aged state of a committed look-ahead as facts, at the snapshot at or
/// before `move_now_myr` (else the last snapshot). Engulfed worlds get only
/// `body.engulfed`.
pub(crate) fn aged_event(
    ch: &Chronicle,
    look: &sse_time::branch::LookAhead,
    move_now_myr: Option<f64>,
    commit: ContentHash,
) -> Result<Option<(Event, f64)>, Response> {
    let sp = &look.request.system;
    let snaps = &look.run.snapshots;
    let snap = match move_now_myr {
        Some(age) => snaps.iter().rfind(|s| s.age_myr <= age),
        None => snaps.last(),
    };
    let Some(snap) = snap else {
        return Ok(None);
    };
    let addr = |s: &str| -> Result<Address, Response> {
        s.parse().map_err(|e| err("bad_address", format!("{e}")))
    };
    let mut set: Facts = BTreeMap::new();
    let s = &snap.star;
    let star = addr(&sp.star_address)?;
    let mut row = BTreeMap::new();
    row.insert("body.mass".to_string(), f(s.mass * M_SUN_KG));
    row.insert("body.radius".to_string(), f(s.radius_rsun * R_SUN));
    row.insert("star.effective_temperature".to_string(), f(s.teff));
    row.insert("star.luminosity".to_string(), f(s.luminosity * L_SUN));
    row.insert("star.phase".to_string(), t(&s.phase));
    set.insert(star.clone(), row);
    let mut subject = vec![star];
    for (p, spec) in snap.planets.iter().zip(&sp.planets) {
        let a = addr(&spec.address)?;
        let mut row = BTreeMap::new();
        row.insert("body.engulfed".to_string(), Value::Bool(p.engulfed));
        if !p.engulfed {
            row.insert("orbit.semi_major_axis".to_string(), f(p.a_au * AU));
            row.insert("orbit.eccentricity".to_string(), f(p.e));
            row.insert("orbit.inclination".to_string(), f(deg(p.i_deg)));
        }
        set.insert(a.clone(), row);
        subject.push(a);
    }
    let at = sse_time::branch::universe_time(sp, snap.age_myr);
    let data = json_value(&serde_json::json!({"age_myr": snap.age_myr}))?;
    let e = event(
        ch,
        TIME_AGED,
        subject,
        at,
        set,
        Some(data),
        vec![commit],
        Evidence::Derived,
        Publisher::new("sse.time", "write-back", "1"),
    )?;
    Ok(Some((e, snap.age_myr)))
}

// --------------------------------------------------------- resolved system

/// How a system came into the universe: the formation request of its
/// `adopt.formation` event, if it has one.
pub(crate) fn formation_request(ch: &Chronicle, system: &Address) -> Option<FormationRequest> {
    ch.events()
        .iter()
        .rev()
        .find(|e| e.body.kind == ADOPT_FORMATION && e.body.subject.first() == Some(system))
        .and_then(|e| e.body.payload.data.as_ref())
        .and_then(|d| serde_json::to_value(d).ok())
        .and_then(|j| serde_json::from_value(j["request"].clone()).ok())
}

fn adopted_v3(ch: &Chronicle, system: &Address) -> bool {
    ch.events()
        .iter()
        .any(|e| e.body.kind == ADOPT_V3 && e.body.subject.contains(system))
}

/// A `SystemSpec` from facts: the star is the body with a temperature and a
/// mass and no host; planets are the bodies whose `orbit.host` is that star.
/// Universe time "now" is `system.now_year` (else the origin), birth is now
/// minus `system.age` unless `system.birth_year` is set.
pub(crate) fn spec_from_facts(facts: &Facts, system: &Address) -> Option<SystemSpec> {
    let row = facts.get(system)?;
    let age_s = row.get("system.age").and_then(as_f64)?;
    let age_myr = age_s / JULIAN_YEAR / 1.0e6;
    let now_year = row
        .get("system.now_year")
        .and_then(as_f64)
        .map(|y| y as i64)
        .unwrap_or(0);
    let birth_year = row
        .get("system.birth_year")
        .and_then(as_f64)
        .map(|y| y as i64)
        .unwrap_or(now_year - m::round(age_myr * 1.0e6) as i64);
    let (star, star_row) = facts.iter().find(|(a, r)| {
        system.contains(a)
            && r.contains_key("star.effective_temperature")
            && r.contains_key("body.mass")
            && !r.contains_key("orbit.host")
    })?;
    let star_mass = as_f64(&star_row["body.mass"])? / M_SUN_KG;
    let z = row
        .get("star.metallicity_z")
        .and_then(as_f64)
        .unwrap_or(DEFAULT_Z);
    let host = star.to_string();
    let rad = |r: &BTreeMap<String, Value>, k: &str| {
        r.get(k).and_then(as_f64).unwrap_or(0.0) * (180.0 / m::PI)
    };
    let mut planets: Vec<PlanetSpec> = facts
        .iter()
        .filter(|(a, r)| {
            system.contains(a)
                && r.get("orbit.host").and_then(as_text) == Some(host.as_str())
                && r.get("body.engulfed") != Some(&Value::Bool(true))
        })
        .filter_map(|(a, r)| {
            let a_m = r.get("orbit.semi_major_axis").and_then(as_f64)?;
            let mass = r.get("body.mass").and_then(as_f64)?;
            let node = rad(r, "orbit.ascending_node");
            let varpi = node + rad(r, "orbit.argument_of_periapsis");
            Some(PlanetSpec {
                name: a.to_string(),
                address: a.to_string(),
                mass_me: mass / M_EARTH_KG,
                a_au: a_m / AU,
                e: r.get("orbit.eccentricity")
                    .and_then(as_f64)
                    .unwrap_or(0.0)
                    .clamp(0.0, 0.9),
                i_deg: rad(r, "orbit.inclination"),
                node_deg: node,
                varpi_deg: varpi,
                mean_longitude_deg: varpi + rad(r, "orbit.mean_anomaly_at_epoch"),
                oceans: r.get("body.oceans") == Some(&Value::Bool(true)),
            })
        })
        .collect();
    planets.sort_by(|a, b| a.a_au.total_cmp(&b.a_au));
    Some(SystemSpec {
        name: system.to_string(),
        system: system.to_string(),
        star_address: host,
        star_mass,
        z,
        age_myr,
        birth_year,
        planets,
        debris: None,
        sources: "facts of the open universe".into(),
    })
}

/// Surface bodies from facts (imported or authored systems): planets and
/// moons with a mass and a radius, temperatures in equilibrium with the star
/// (albedo and greenhouse from the surface pack), deep envelopes for worlds
/// above 10 Earth masses with a density under 2500 kg/m^3.
fn bodies_from_facts(
    facts: &Facts,
    spec: &SystemSpec,
    s: &sse_surface::SurfaceSettings,
) -> Vec<sse_surface::Body> {
    let star: Address = match spec.star_address.parse() {
        Ok(a) => a,
        Err(_) => return vec![],
    };
    let srow = &facts[&star];
    let teff = srow
        .get("star.effective_temperature")
        .and_then(as_f64)
        .unwrap_or(5772.0);
    let r_star = srow.get("body.radius").and_then(as_f64).unwrap_or(R_SUN);
    let host_of = |r: &BTreeMap<String, Value>| -> Option<Address> {
        r.get("orbit.host").and_then(as_text)?.parse().ok()
    };
    let mut out = vec![];
    let system: Address = spec.system.parse().expect("spec system address");
    for (a, r) in facts.iter().filter(|(a, _)| system.contains(a)) {
        if *a == star || r.contains_key("star.effective_temperature") {
            continue;
        }
        if r.get("body.engulfed") == Some(&Value::Bool(true)) {
            continue;
        }
        let (Some(mass), Some(radius), Some(host)) = (
            r.get("body.mass").and_then(as_f64),
            r.get("body.radius").and_then(as_f64),
            host_of(r),
        ) else {
            continue;
        };
        // Distance from the star: own orbit, or the host planet's for a moon.
        let a_own = r
            .get("orbit.semi_major_axis")
            .and_then(as_f64)
            .unwrap_or(AU);
        let (a_star, e, host_msun) = if host == star {
            (
                a_own,
                r.get("orbit.eccentricity").and_then(as_f64).unwrap_or(0.0),
                spec.star_mass,
            )
        } else {
            let hr = match facts.get(&host) {
                Some(x) => x,
                None => continue,
            };
            (
                hr.get("orbit.semi_major_axis")
                    .and_then(as_f64)
                    .unwrap_or(AU),
                r.get("orbit.eccentricity").and_then(as_f64).unwrap_or(0.0),
                hr.get("body.mass").and_then(as_f64).unwrap_or(0.0) / M_SUN_KG,
            )
        };
        let mass_earth = mass / M_EARTH_KG;
        let density = mass / (4.0 / 3.0 * m::PI * radius * radius * radius);
        // Imports carry surface water in Earth oceans; formation adoption
        // carries only whether there are oceans (one Earth ocean then).
        let oceans = match r.get("body.surface_water_oceans").and_then(as_f64) {
            Some(x) => x.max(0.0),
            None if r.get("body.oceans") == Some(&Value::Bool(true)) => 1.0,
            None => 0.0,
        };
        let greenhouse = if oceans > 0.0 { s.greenhouse_k } else { 0.0 };
        let t_eq = teff * m::sqrt(r_star / (2.0 * a_star)) * m::pow(1.0 - s.albedo_default, 0.25);
        out.push(sse_surface::Body {
            address: a.clone(),
            label: r
                .get("v3.name")
                .and_then(as_text)
                .map(str::to_string)
                .unwrap_or_else(|| a.to_string()),
            mass_earth,
            radius_earth: radius / R_EARTH,
            core_mass_fraction: 0.32,
            water_fraction: 0.0,
            oceans,
            deep_envelope: mass_earth > 10.0 && density < 2500.0,
            pressure_bar: r
                .get("body.surface_pressure")
                .and_then(as_f64)
                .map(|p| p / 1e5)
                .unwrap_or(0.0),
            c_o: 0.55,
            k40_rel: 1.0,
            r_process_rel: 1.0,
            age_gyr: spec.age_myr / 1000.0,
            magma_end_myr: 0.0,
            climate: sse_surface::Climate::Fixed {
                t_surface_k: t_eq + greenhouse,
            },
            tidal_w_m2: None,
            orbit: Some(sse_surface::Orbit {
                a_au: a_own / AU,
                e,
                host_msun,
            }),
            locked: None,
        });
    }
    out
}

pub(crate) struct Resolved {
    pub evidence: serde_json::Value,
    pub source: String,
    pub at_s: Option<i64>,
    pub facts: serde_json::Value,
    pub spec: Option<serde_json::Value>,
    pub surfaces: Vec<serde_json::Value>,
    pub tiles: Vec<serde_json::Value>,
    pub refused: Vec<serde_json::Value>,
    pub clouds: Vec<serde_json::Value>,
}

/// F5's leftovers (belts, the scattered disc, the Oort cloud, Trojans,
/// irregular moons and rings) as the formation run made them: read from the
/// run, never written as events.
fn clouds_of(run: Option<&SystemRun>) -> Vec<serde_json::Value> {
    let Some(l) = run.and_then(|r| r.leftovers.as_ref()) else {
        return vec![];
    };
    l.clouds
        .iter()
        .map(|c| {
            serde_json::json!({
                "address": c.address.to_string(),
                "kind": c.kind,
                "around": c.around,
                "source": c.source,
                "a_min_au": c.a_min_au,
                "a_max_au": c.a_max_au,
                "largest_km": c.largest_km,
                "over_1km": c.over_1km,
            })
        })
        .collect()
}

/// A system's worlds at a date: where they came from, the spec, the facts
/// read and how far past the system's "now" the date is.
pub(crate) struct Worlds {
    pub source: &'static str,
    pub spec: Option<SystemSpec>,
    pub bodies: Vec<sse_surface::Body>,
    pub facts: Facts,
    pub at_s: i64,
    /// How far past the system's own "now" the asked date is, Gyr.
    pub shift_gyr: f64,
}

pub(crate) fn worlds_at(
    ch: &Chronicle,
    system: &Address,
    at_s: Option<i64>,
    run: Option<&SystemRun>,
    s: &sse_surface::SurfaceSettings,
) -> Result<Worlds, Response> {
    let all = ch.state_at(UniverseTime::seconds(i64::MAX));
    let now_s = all
        .get(system, "system.now_year")
        .and_then(as_f64)
        .map(|y| y as i64 * 31_557_600);
    // Once the one clock has moved this system (its own age shift, or campaign
    // time elapsed), "now" is its present on its own dates, not the latest
    // event: a kept look-ahead's future must not read as the present.
    let moved =
        crate::clock::age_shift_myr(&all, system) != 0.0 || crate::clock::elapsed_myr(ch) != 0.0;
    let present_s = match (at_s, now_s) {
        (None, None) if moved => worlds_at(ch, system, Some(i64::MAX), run, s)?
            .spec
            .map(|sp| m::round((sp.birth_year as f64 + sp.age_myr * 1.0e6) * 31_557_600.0) as i64),
        _ => None,
    };
    let at_s = at_s.or(now_s).or(present_s).unwrap_or(i64::MAX);
    let st = ch.state_at(UniverseTime::seconds(at_s));
    let facts: Facts = st
        .facts
        .iter()
        .filter(|(a, _)| system.contains(a))
        .map(|(a, r)| (a.clone(), r.clone()))
        .collect();
    if facts.is_empty() {
        return Err(err(
            "unknown_system",
            format!("{system} has no facts in this universe"),
        ));
    }
    let (source, spec, bodies) = match run {
        Some(run) => {
            let spec = sse_time::generated::spec_from_run(run);
            let gone: Vec<&Address> = facts
                .iter()
                .filter(|(_, r)| r.get("body.engulfed") == Some(&Value::Bool(true)))
                .map(|(a, _)| a)
                .collect();
            let bodies: Vec<sse_surface::Body> = sse_surface::bodies_of(run, s)
                .into_iter()
                .filter(|b| !gone.contains(&&b.address))
                .collect();
            ("formation", Some(spec), bodies)
        }
        None => {
            let spec = spec_from_facts(&facts, system);
            let bodies = spec
                .as_ref()
                .map(|sp| bodies_from_facts(&facts, sp, s))
                .unwrap_or_default();
            let source = if adopted_v3(ch, system) {
                "v3"
            } else {
                "facts"
            };
            (source, spec, bodies)
        }
    };
    let mut shift_gyr = match (&spec, at_s) {
        (Some(sp), a) if a != i64::MAX => {
            let now_year = sp.birth_year as f64 + sp.age_myr * 1.0e6;
            (a as f64 / 31_557_600.0 - now_year) / 1.0e9
        }
        _ => 0.0,
    };
    // One clock (top-level design 12.2): the system's own age shift makes it
    // older at every date; campaign time elapsed makes "now" later.
    let age_shift = crate::clock::age_shift_myr(&all, system);
    let elapsed = crate::clock::elapsed_myr(ch);
    shift_gyr += age_shift / 1000.0;
    if at_s == i64::MAX {
        shift_gyr += elapsed / 1000.0;
    }
    let spec = spec.map(|mut sp| {
        sp.age_myr += age_shift + elapsed;
        sp.birth_year -= m::round(age_shift * 1.0e6) as i64;
        if run.is_some() {
            if let Some(z) = all.get(system, "star.metallicity_z").and_then(as_f64) {
                sp.z = z;
            }
        }
        sp
    });
    Ok(Worlds {
        source,
        spec,
        bodies,
        facts,
        at_s,
        shift_gyr,
    })
}

/// Orbit atlases kept so that reopening a world reuses its plate run.
pub const ORBIT_CACHE_SIZE: usize = 8;

/// Recent Orbit-level atlases by key, newest last.
pub(crate) type OrbitCache = Vec<(String, sse_surface::atlas::Atlas)>;

/// The Orbit atlas of one world from the cache, or worked out and kept.
fn orbit_atlas<'c>(
    cache: &'c mut OrbitCache,
    s: &sse_surface::SurfaceSettings,
    body: &sse_surface::Body,
    env: &sse_surface::SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
) -> Result<&'c sse_surface::atlas::Atlas, String> {
    let key = sse_surface::detail::orbit_key(env, seed, epoch_gyr);
    match cache.iter().position(|(k, _)| *k == key) {
        Some(i) => {
            let hit = cache.remove(i);
            cache.push(hit);
        }
        None => {
            let a = sse_surface::detail::orbit_atlas(s, body, seed, epoch_gyr)?;
            if cache.len() >= ORBIT_CACHE_SIZE {
                cache.remove(0);
            }
            cache.push((key, a));
        }
    }
    Ok(&cache[cache.len() - 1].1)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolved_system(
    ch: &Chronicle,
    seed: &Seed,
    system: &Address,
    at_s: Option<i64>,
    tiles: &[TileRequest],
    run: Option<&SystemRun>,
    orbits: &mut OrbitCache,
) -> Result<Resolved, Response> {
    if tiles.len() > MAX_TILES {
        return Err(err(
            "too_many_tiles",
            format!("at most {MAX_TILES} tiles per request"),
        ));
    }
    let s = sse_surface::scenario::settings();
    let Worlds {
        source,
        spec,
        bodies,
        facts,
        at_s,
        shift_gyr,
    } = worlds_at(ch, system, at_s, run, &s)?;
    let envelopes: Vec<sse_surface::SurfaceEnvelope> = bodies
        .iter()
        .map(|b| sse_surface::envelope(&s, b))
        .collect();
    let mut out_tiles = vec![];
    let mut refused = vec![];
    for tr in tiles {
        let key = sse_surface::atlas::TileKey::new(tr.face, tr.level, tr.x, tr.y);
        let found = bodies
            .iter()
            .zip(&envelopes)
            .find(|(b, _)| b.address.to_string() == tr.body);
        let reason = match (key, found) {
            (None, _) => Some("bad tile key".to_string()),
            (_, None) => Some("no surface body at that address".to_string()),
            (Some(key), Some((b, env))) => {
                // Orbit detail (S16): the coarse plate run, the same world a
                // survey later refines; reused while it is in the cache.
                let atlas = orbit_atlas(orbits, &s, b, env, seed, (b.age_gyr + shift_gyr).max(0.0));
                match atlas.map(|a| a.tile(key, sse_surface::SurveyState::Surveyed)) {
                    Ok(Ok(tile)) => {
                        out_tiles.push(serde_json::to_value(&tile).expect("plain data"));
                        None
                    }
                    Ok(Err(r)) => Some(format!("{r:?}")),
                    Err(e) => Some(e),
                }
            }
        };
        if let Some(why) = reason {
            refused.push(serde_json::json!({
                "body": tr.body, "face": tr.face, "level": tr.level, "x": tr.x, "y": tr.y,
                "why": why,
            }));
        }
    }
    let evidence = evidence_of(ch, system, at_s);
    let facts_json: serde_json::Map<String, serde_json::Value> = facts
        .iter()
        .map(|(a, r)| {
            (
                a.to_string(),
                serde_json::to_value(r).expect("facts are plain data"),
            )
        })
        .collect();
    Ok(Resolved {
        evidence,
        source: source.into(),
        at_s: (at_s != i64::MAX).then_some(at_s),
        facts: serde_json::Value::Object(facts_json),
        spec: spec.map(|x| serde_json::to_value(x).expect("plain data")),
        surfaces: envelopes
            .iter()
            .map(|e| serde_json::to_value(e).expect("plain data"))
            .collect(),
        tiles: out_tiles,
        refused,
        clouds: if source == "formation" {
            clouds_of(run)
        } else {
            vec![]
        },
    })
}

fn evidence_label(e: Evidence) -> &'static str {
    match e {
        Evidence::Catalogued => "catalogued",
        Evidence::Derived => "derived",
        Evidence::Resolved => "resolved",
        Evidence::Authored => "authored",
        Evidence::AuthoredImpossible => "authored-impossible",
        Evidence::Narrative => "narrative",
    }
}

/// Who set each fact under `system` as of `at_s`: address to field to
/// `{evidence, by}` (the evidence label and the setting event's kind), from
/// the last event in fold order that set it.
pub(crate) fn evidence_of(ch: &Chronicle, system: &Address, at_s: i64) -> serde_json::Value {
    let at = UniverseTime::seconds(at_s);
    let mut sel: Vec<&Event> = ch
        .events()
        .iter()
        .filter(|e| e.body.activated_at <= at)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut who: BTreeMap<String, BTreeMap<String, serde_json::Value>> = BTreeMap::new();
    for e in sel {
        for (a, fields) in &e.body.payload.set {
            if !system.contains(a) {
                continue;
            }
            let row = who.entry(a.to_string()).or_default();
            for k in fields.keys() {
                row.insert(
                    k.clone(),
                    serde_json::json!({"evidence": evidence_label(e.body.evidence), "by": e.body.kind}),
                );
            }
        }
        for (a, fields) in &e.body.payload.unset {
            if let Some(row) = who.get_mut(&a.to_string()) {
                for k in fields {
                    row.remove(k);
                }
            }
        }
    }
    serde_json::to_value(who).expect("plain data")
}

/// The systems of a universe (addresses `<root>/sys:*` with facts at or
/// under them), with where each came from and a name if one is known.
pub(crate) fn systems_of(ch: &Chronicle, root: &Address) -> Vec<serde_json::Value> {
    let st = ch.state_at(UniverseTime::seconds(i64::MAX));
    let mut found: BTreeMap<Address, u32> = BTreeMap::new();
    for a in st.facts.keys() {
        if !root.contains(a) || a.segments().len() <= root.segments().len() {
            continue;
        }
        let mut sys = a.clone();
        while sys.segments().len() > root.segments().len() + 1 {
            match sys.parent() {
                Some(p) => sys = p,
                None => break,
            }
        }
        if sys.last().kind() != "sys" {
            continue;
        }
        *found.entry(sys).or_insert(0) += 1;
    }
    found
        .into_iter()
        .map(|(sys, n)| {
            let adopted = ch.events().iter().find(|e| {
                (e.body.kind == ADOPT_V3 || e.body.kind == ADOPT_FORMATION)
                    && e.body.subject.contains(&sys)
            });
            let source = match adopted.map(|e| e.body.kind.as_str()) {
                Some(ADOPT_V3) => "v3",
                Some(ADOPT_FORMATION) => "formation",
                _ => "facts",
            };
            let name = ["name", "v3.name"]
                .iter()
                .find_map(|k| st.get(&sys, k).and_then(as_text).map(str::to_string));
            serde_json::json!({
                "system": sys.to_string(),
                "source": source,
                "name": name,
                "addresses": n,
                "adopted_event": adopted.map(|e| e.id.hex()),
            })
        })
        .collect()
}

/// `export_chronicle` document format.
pub const EXPORT_FORMAT: &str = "sse.chronicle-export/1";

/// L1d's life view for a fixture world (bodies come later).
pub(crate) fn life_view(
    body: Option<&str>,
    fixture: Option<&str>,
) -> Result<(String, serde_json::Value), Response> {
    if let Some(b) = body {
        return Err(err(
            "no_habitats",
            format!("{b}: adopted systems do not carry habitat graphs yet; use a fixture"),
        ));
    }
    let name = fixture.ok_or_else(|| err("bad_request", "give a fixture or a body"))?;
    let cases = sse_life::scenario::signature_cases();
    let names: Vec<String> = cases.iter().map(|c| c.0.clone()).collect();
    let (n, b, h, o) = cases.into_iter().find(|c| c.0 == name).ok_or_else(|| {
        err(
            "unknown_fixture",
            format!("no life fixture {name}; fixtures: {}", names.join(", ")),
        )
    })?;
    let s = sse_life::signatures::signatures(
        b.as_ref(),
        h.as_ref(),
        &o,
        &sse_life::signatures::SignatureParams::default(),
    );
    let inp = sse_life::sift::HookInputs {
        habitable_until_myr: None,
        o2_fraction: s.o2,
        now_myr: o.age_myr,
    };
    let v = sse_life::sift::view(
        &n,
        b.as_ref(),
        h.as_ref(),
        &s,
        &inp,
        &sse_life::sift::SiftParams::default(),
    );
    Ok((
        format!("fixture:{n}"),
        serde_json::to_value(v).expect("plain data"),
    ))
}

pub(crate) fn hex(b: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push(D[(x >> 4) as usize] as char);
        s.push(D[(x & 15) as usize] as char);
    }
    s
}

pub(crate) fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let v = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            _ => None,
        }
    };
    s.as_bytes()
        .chunks(2)
        .map(|p| Some(v(p[0])? << 4 | v(p[1])?))
        .collect()
}
