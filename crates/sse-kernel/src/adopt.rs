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
pub const M_SUN_KG: f64 = sse_core::sheet::M_SUN;

/// The serde name of a plain enum value (kebab-case labels).
fn json_name<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|x| x.as_str().map(str::to_string))
        .unwrap_or_default()
}
pub const M_EARTH_KG: f64 = sse_core::sheet::M_EARTH;
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

/// A system grown from a drawn star (`from_star`) may hold the star's own
/// promotion record at its address, and nothing else.
fn refuse_existing_but_the_star(ch: &Chronicle, system: &Address) -> Result<(), Response> {
    let st = ch.state_at(UniverseTime::seconds(i64::MAX));
    let ok = st
        .facts
        .iter()
        .filter(|(a, _)| system.contains(a))
        .all(|(a, f)| a == system && f.contains_key(crate::cosmos::STAR_DESCRIPTION));
    if !ok {
        return Err(err(
            "already_adopted",
            format!("{system} already has a system grown in this universe"),
        ));
    }
    Ok(())
}

pub(crate) fn adopt(
    ch: &Chronicle,
    root: &Address,
    import_v3: Option<String>,
    formation: Option<FormationRequest>,
    occurred_at_s: Option<i64>,
    from_star: Option<&str>,
) -> Result<Adoption, Response> {
    let at = UniverseTime::seconds(occurred_at_s.unwrap_or(0));
    match (import_v3, formation) {
        (Some(save), None) => adopt_v3(ch, root, &save, at),
        (None, Some(req)) => adopt_formation(ch, root, req, at, from_star.is_some()),
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
    let mut im = sse_import_v3::import_json(save, &universe_key(root)?)
        .map_err(|e| err("import_failed", e.0))?;
    let said = rehome_barycentres(&mut im.facts);
    im.report.barycentres_mapped = said;
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

type V3Facts = BTreeMap<Address, BTreeMap<String, sse_import_v3::Fact>>;

fn is_bary(a: &Address) -> bool {
    a.last().kind() == "bary"
}

fn text_of(row: Option<&BTreeMap<String, sse_import_v3::Fact>>, k: &str) -> Option<String> {
    row?.get(k)?.value.as_str().map(str::to_owned)
}

/// The members of a V3 barycentre: its `v3.memberIds` (addresses, as the
/// importer resolves them) that are in the save, in V3's order.
fn bary_members(facts: &V3Facts, b: &Address) -> Vec<Address> {
    let listed: Vec<Address> = facts
        .get(b)
        .and_then(|r| r.get("v3.memberIds"))
        .and_then(|f| f.value.as_array())
        .map(|xs| {
            xs.iter()
                .filter_map(|x| x.as_str()?.parse::<Address>().ok())
                .filter(|a| a != b && facts.contains_key(a))
                .collect()
        })
        .unwrap_or_default();
    listed
}

/// A node's mass: its own `body.mass`, or for a barycentre its members' sum.
fn node_mass(facts: &V3Facts, a: &Address, depth: u32) -> f64 {
    if depth > 16 {
        return 0.0;
    }
    if is_bary(a) {
        return bary_members(facts, a)
            .iter()
            .map(|x| node_mass(facts, x, depth + 1))
            .sum();
    }
    facts
        .get(a)
        .and_then(|r| r.get("body.mass"))
        .and_then(|f| f.value.as_f64())
        .filter(|x| x.is_finite() && *x > 0.0)
        .unwrap_or(0.0)
}

/// The heaviest member of a barycentre (the first listed on a tie).
fn heaviest(facts: &V3Facts, members: &[Address]) -> Option<Address> {
    let mut best: Option<(&Address, f64)> = None;
    for x in members {
        let mx = node_mass(facts, x, 0);
        if best.is_none_or(|(_, mb)| mx > mb) {
            best = Some((x, mx));
        }
    }
    best.map(|(x, _)| x.clone())
}

/// The body that stands for a node: a body itself, or the body at the bottom
/// of a barycentre's heaviest members.
fn anchor(facts: &V3Facts, a: &Address) -> Option<Address> {
    let mut at = a.clone();
    for _ in 0..16 {
        if !is_bary(&at) {
            return Some(at);
        }
        at = heaviest(facts, &bary_members(facts, &at))?;
    }
    None
}

fn orbit_keys(row: &BTreeMap<String, sse_import_v3::Fact>) -> Vec<String> {
    row.keys()
        .filter(|k| k.starts_with("orbit."))
        .cloned()
        .collect()
}

/// V3 barycentre nodes onto the hierarchy (BC1: a barycentre always, no
/// barycentre objects). In a V3 save the members of a pair orbit a massless
/// `bary:N`, each at its own distance from it. SSE4 keeps only each body's
/// host and works the barycentre out (`orbits_at`), so on import:
/// - the heaviest member (its anchor body, for a nested pair) takes the pair's
///   own orbit round its host, or orbits nothing when the pair is the root;
/// - each other member orbits that body, at the pair's separation (its own
///   distance plus the heavy member's, or scaled by the masses when that is
///   not given), keeping its own shape, angles and phase;
/// - anything else that orbited the barycentre (a planet round both stars)
///   orbits the heavy body with its elements unchanged: the hierarchy counts
///   both stars for a world outside the companion;
/// - the barycentre's row is left out; its `v3.*` facts go to that body as
///   `v3.pair.<bary:N>.<name>`.
///
/// Returns one line per barycentre for the import report.
pub(crate) fn rehome_barycentres(facts: &mut V3Facts) -> Vec<String> {
    use sse_import_v3::Fact;
    let barys: Vec<Address> = facts.keys().filter(|a| is_bary(a)).cloned().collect();
    if barys.is_empty() {
        return vec![];
    }
    let member_of: BTreeMap<Address, Address> = barys
        .iter()
        .flat_map(|b| {
            bary_members(facts, b)
                .into_iter()
                .map(move |x| (x, b.clone()))
        })
        .collect();
    let mut said = vec![];
    // Writes are worked out from the save as read, then applied together, so
    // the order barycentres are visited in never matters.
    let mut writes: Vec<(Address, BTreeMap<String, Fact>, String)> = vec![];
    let mut mapped: BTreeMap<Address, Address> = BTreeMap::new();
    for b in &barys {
        let members = bary_members(facts, b);
        let (Some(heavy), Some(primary)) = (heaviest(facts, &members), anchor(facts, b)) else {
            said.push(format!(
                "{b}: no members in the save; what orbited it is left unplaced"
            ));
            continue;
        };
        mapped.insert(b.clone(), primary.clone());
        let host_of = |x: &Address| -> Option<String> {
            let h: Address = text_of(facts.get(x), "orbit.host")?.parse().ok()?;
            Some(match is_bary(&h) {
                true => anchor(facts, &h)?.to_string(),
                false => h.to_string(),
            })
        };
        // The heavy side carries the pair's own orbit, unless the pair is a
        // member of a wider pair, which places it.
        if !member_of.contains_key(b) {
            let mut row: BTreeMap<String, Fact> = BTreeMap::new();
            if let Some(src) = facts.get(b) {
                for k in orbit_keys(src) {
                    row.insert(k.clone(), src[&k].clone());
                }
            }
            let parent = text_of(facts.get(b), "parent");
            match host_of(b) {
                Some(h) => {
                    row.insert(
                        "orbit.host".into(),
                        Fact {
                            value: h.clone().into(),
                            evidence: Evidence::Authored,
                        },
                    );
                    row.insert(
                        "parent".into(),
                        Fact {
                            value: h.into(),
                            evidence: Evidence::Authored,
                        },
                    );
                }
                None => {
                    if let Some(p) = parent.and_then(|p| p.parse::<Address>().ok()) {
                        let p = if is_bary(&p) {
                            anchor(facts, &p)
                        } else {
                            Some(p)
                        };
                        if let Some(p) = p {
                            row.insert(
                                "parent".into(),
                                Fact {
                                    value: p.to_string().into(),
                                    evidence: Evidence::Authored,
                                },
                            );
                        }
                    }
                }
            }
            writes.push((primary.clone(), row, "heavy".into()));
        }
        let m_heavy = node_mass(facts, &heavy, 0);
        let a_heavy = facts
            .get(&heavy)
            .and_then(|r| r.get("orbit.semi_major_axis"))
            .and_then(|f| f.value.as_f64());
        let mut names = vec![primary.to_string()];
        for x in members.iter().filter(|x| **x != heavy) {
            let Some(target) = anchor(facts, x) else {
                continue;
            };
            let Some(src) = facts.get(x) else { continue };
            let mut row: BTreeMap<String, Fact> = BTreeMap::new();
            for k in orbit_keys(src) {
                row.insert(k.clone(), src[&k].clone());
            }
            let m_x = node_mass(facts, x, 0);
            if let Some(a_x) = src
                .get("orbit.semi_major_axis")
                .and_then(|f| f.value.as_f64())
            {
                let sep = match a_heavy {
                    Some(ah) => a_x + ah,
                    None if m_heavy > 0.0 => a_x * (m_heavy + m_x) / m_heavy,
                    None => a_x,
                };
                row.insert(
                    "orbit.semi_major_axis".into(),
                    Fact {
                        value: sep.into(),
                        evidence: src["orbit.semi_major_axis"].evidence,
                    },
                );
            }
            for k in ["orbit.host", "parent"] {
                row.insert(
                    k.into(),
                    Fact {
                        value: primary.to_string().into(),
                        evidence: Evidence::Authored,
                    },
                );
            }
            names.push(target.to_string());
            writes.push((target, row, "companion".into()));
        }
        said.push(format!(
            "{b}: the pair {} now orbit {primary} (its heaviest member); the barycentre is worked out, not kept",
            names.join(" and ")
        ));
    }
    // Everything else that named a barycentre as host or parent names its body.
    let written: Vec<Address> = writes.iter().map(|(a, _, _)| a.clone()).collect();
    for (a, row) in facts.iter_mut() {
        if is_bary(a) || written.contains(a) {
            continue;
        }
        for k in ["orbit.host", "parent"] {
            let to = row
                .get(k)
                .and_then(|f| f.value.as_str())
                .and_then(|s| s.parse::<Address>().ok())
                .and_then(|h| mapped.get(&h).cloned());
            if let (Some(to), Some(f)) = (to, row.get_mut(k)) {
                f.value = to.to_string().into();
            }
        }
    }
    for (a, row, _) in writes {
        let r = facts.entry(a).or_default();
        for k in orbit_keys(r) {
            r.remove(&k);
        }
        r.remove("parent");
        r.extend(row);
    }
    // The pair's own hand-set V3 facts (its name, a Lagrange point it rides)
    // go with the body that stands for it, so nothing authored is lost.
    for (b, primary) in &mapped {
        let Some(row) = facts.remove(b) else { continue };
        let seg = format!("{}:{}", b.last().kind(), b.last().key());
        let kept: Vec<(String, Fact)> = row
            .into_iter()
            .filter_map(|(k, f)| {
                let rest = k.strip_prefix("v3.")?;
                Some((format!("v3.pair.{seg}.{rest}"), f))
            })
            .collect();
        facts.entry(primary.clone()).or_default().extend(kept);
    }
    said
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
    from_star: bool,
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
    if from_star {
        refuse_existing_but_the_star(ch, &system)?;
    } else {
        refuse_existing(ch, std::slice::from_ref(&system))?;
    }
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
                name: crate::names::typed_common(r).unwrap_or_else(|| a.to_string()),
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
const MAKEUP: [&str; 5] = [
    "makeup.metal",
    "makeup.rock",
    "makeup.carbon",
    "makeup.ice",
    "makeup.gas",
];

/// A typed makeup's core share (metal against metal, rock and carbon) and
/// water share (ice of the whole); `None` when no makeup is typed.
fn makeup_shares(r: &BTreeMap<String, Value>) -> Option<(f64, f64)> {
    let share = |k: &str| r.get(k).and_then(as_f64).filter(|x| *x >= 0.0);
    if MAKEUP.iter().all(|k| share(k).is_none()) {
        return None;
    }
    let v = |k: &str| share(k).unwrap_or(0.0);
    let solid = v("makeup.metal") + v("makeup.rock") + v("makeup.carbon");
    let total = solid + v("makeup.ice") + v("makeup.gas");
    let cmf = if solid > 0.0 {
        v("makeup.metal") / solid
    } else {
        0.32
    };
    let wf = if total > 0.0 {
        v("makeup.ice") / total
    } else {
        0.0
    };
    Some((cmf.min(1.0), wf.min(1.0)))
}

/// K3 (editing design 15): the radius a solid world's mass and makeup give
/// when none is set: the Zeng, Sasselov and Jacobsen rocky block
/// (`sse.interior.rocky_radius`) with the core share read from the makeup's
/// metal against its rock (Earth's 0.32 when no makeup is set), and a water
/// or ice mantle adding half its mass share to the radius, as formation's
/// stage 8 does (Zeng et al. 2019: half water by mass is about a quarter
/// larger). Outside the block's 1 to 8 Earth masses it is an extrapolation,
/// as in formation. A world mostly of gas gets none: no giant radius block
/// is in the base pack yet.
pub(crate) fn radius_from_makeup(mass: f64, r: &BTreeMap<String, Value>) -> Option<f64> {
    let share = |k: &str| {
        r.get(k)
            .and_then(as_f64)
            .filter(|x| *x >= 0.0)
            .unwrap_or(0.0)
    };
    let total: f64 = MAKEUP.iter().map(|k| share(k)).sum();
    if total > 0.0 && share("makeup.gas") / total > 0.5 {
        return None;
    }
    let (cmf, wf) = makeup_shares(r).unwrap_or((0.32, 0.0));
    let atoms = sse_atoms::atoms();
    let a = atoms
        .iter()
        .find(|a| a.decl().id == "sse.interior.rocky_radius")?;
    let p: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
    let out = a.eval(&[mass, cmf.min(1.0)], &p).ok()?;
    Some(out[0] * (1.0 + 0.5 * wf)).filter(|x| x.is_finite() && *x > 0.0)
}

fn bodies_from_facts(
    facts: &Facts,
    catalogued: &Facts,
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
        .unwrap_or(sse_core::sheet::T_SUN);
    let r_star = srow.get("body.radius").and_then(as_f64).unwrap_or(R_SUN);
    let host_of = |r: &BTreeMap<String, Value>| -> Option<Address> {
        r.get("orbit.host").and_then(as_text)?.parse().ok()
    };
    // Today's temperatures are anchored to the host star's life line, so a
    // world's history follows the star's light (the faint young Sun).
    let fits = sse_formation::hurley::Fits::new(spec.z);
    let anchor = sse_formation::lifeline::LifeLine::new(&fits, spec.star_mass)
        .ok()
        .map(|life| sse_surface::Anchor {
            life: Box::new(life),
            z: spec.z,
            formed_myr: 0.0,
            measured_myr: spec.age_myr,
        });
    let mut out = vec![];
    let system: Address = spec.system.parse().expect("spec system address");
    for (a, r) in facts.iter().filter(|(a, _)| system.contains(a)) {
        if *a == star || r.contains_key("star.effective_temperature") {
            continue;
        }
        if r.get("body.engulfed") == Some(&Value::Bool(true)) {
            continue;
        }
        let (Some(mass), Some(host)) = (r.get("body.mass").and_then(as_f64), host_of(r)) else {
            continue;
        };
        // K3: a hand-made world with no radius takes the one its mass and
        // makeup give (`radius_from_makeup`), so it still has ground.
        let Some(radius) = r
            .get("body.radius")
            .and_then(as_f64)
            .or_else(|| radius_from_makeup(mass, r))
        else {
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
        // A catalogued world brings what was measured: its own Bond albedo,
        // its mean surface temperature and its air. Only what is missing is
        // estimated (equilibrium with the star, the pack's greenhouse).
        let albedo = r
            .get("body.albedo")
            .and_then(as_f64)
            .filter(|x| (0.0..1.0).contains(x))
            .unwrap_or(s.albedo_default);
        let greenhouse = if oceans > 0.0 { s.greenhouse_k } else { 0.0 };
        let t_eq = teff * m::sqrt(r_star / (2.0 * a_star)) * m::pow(1.0 - albedo, 0.25);
        // A measured mean temperature belongs to the world it was measured
        // on. Typed by the GM, it stands as typed. Catalogued, it holds the
        // air's warming as a ratio to equilibrium (the grey factor
        // (1 + 3/4 tau)^(1/4)), so a world moved or re-whitened since keeps
        // its air but answers to its new sunlight.
        let measured = r
            .get("planet.surface_temperature")
            .and_then(as_f64)
            .filter(|t| *t > 0.0);
        let t_surface_k = match measured {
            None => t_eq + greenhouse,
            Some(t) => {
                let cat = catalogued.get(a);
                let same = cat
                    .and_then(|c| c.get("planet.surface_temperature"))
                    .and_then(as_f64)
                    == Some(t);
                match (
                    same,
                    cat.and_then(|c| equilibrium_of(c, catalogued, s, teff, r_star)),
                ) {
                    (true, Some(t_eq_then)) if t_eq_then > 0.0 => t * t_eq / t_eq_then,
                    _ => t,
                }
            }
        };
        let pressure_pa = r
            .get("atmosphere.surface_pressure")
            .or_else(|| r.get("body.surface_pressure"))
            .and_then(as_f64)
            .unwrap_or(0.0)
            .max(0.0);
        out.push(sse_surface::Body {
            address: a.clone(),
            label: r
                .get("v3.name")
                .and_then(as_text)
                .map(str::to_string)
                .unwrap_or_else(|| a.to_string()),
            mass_earth,
            radius_earth: radius / R_EARTH,
            // A measured core share (catalogued or typed) wins; else K3's typed
            // makeup sets the core and water shares inside; else Earth's.
            core_mass_fraction: r
                .get("interior.core_mass_fraction")
                .and_then(as_f64)
                .filter(|x| (0.0..=1.0).contains(x))
                .unwrap_or_else(|| makeup_shares(r).map_or(0.32, |m| m.0)),
            water_fraction: makeup_shares(r).map_or(0.0, |m| m.1),
            oceans,
            deep_envelope: mass_earth > 10.0 && density < 2500.0,
            pressure_bar: pressure_pa / 1e5,
            c_o: 0.55,
            k40_rel: 1.0,
            r_process_rel: 1.0,
            age_gyr: spec.age_myr / 1000.0,
            magma_end_myr: 0.0,
            climate: sse_surface::Climate::Fixed {
                t_surface_k,
                then: anchor.clone(),
            },
            tidal_w_m2: None,
            orbit: Some(sse_surface::Orbit {
                a_au: a_own / AU,
                e,
                host_msun,
            }),
            locked: None,
            tidal_history: vec![],
            spin_hours: r
                .get("body.rotation_period")
                .and_then(as_f64)
                .map(|t| t / 3600.0),
            obliquity_deg: r
                .get("body.obliquity")
                .or_else(|| r.get("body.axial_tilt"))
                .and_then(as_f64)
                .map(|x| x * 180.0 / m::PI),
            spin_source: r
                .keys()
                .any(|k| {
                    k.starts_with("body.rotation_period")
                        || k.starts_with("body.obliquity")
                        || k.starts_with("body.axial_tilt")
                })
                .then(|| {
                    if r.keys().any(|k| k.starts_with("v3.")) {
                        "authored (V3)".to_string()
                    } else {
                        "authored".to_string()
                    }
                }),
            core_light_fraction: None,
            makeup: {
                let share = |k: &str| r.get(k).and_then(as_f64).filter(|x| *x >= 0.0);
                let keys = [
                    "makeup.metal",
                    "makeup.rock",
                    "makeup.carbon",
                    "makeup.ice",
                    "makeup.gas",
                ];
                if keys.iter().any(|k| share(k).is_some()) {
                    Some(sse_surface::Makeup {
                        metal: share(keys[0]).unwrap_or(0.0),
                        rock: share(keys[1]).unwrap_or(0.0),
                        carbon: share(keys[2]).unwrap_or(0.0),
                        ice: share(keys[3]).unwrap_or(0.0),
                        gas: share(keys[4]).unwrap_or(0.0),
                    })
                } else {
                    None
                }
            },
        });
    }
    out
}

/// Equilibrium temperature of a fact row (its own orbit, or its host's for a
/// moon; its own Bond albedo, else the pack default), for comparing a world
/// now with the world as catalogued.
fn equilibrium_of(
    r: &BTreeMap<String, Value>,
    facts: &Facts,
    s: &sse_surface::SurfaceSettings,
    teff: f64,
    r_star: f64,
) -> Option<f64> {
    let host: Option<Address> = r
        .get("orbit.host")
        .and_then(as_text)
        .and_then(|h| h.parse().ok());
    let own = r.get("orbit.semi_major_axis").and_then(as_f64);
    let a = match host.as_ref().and_then(|h| facts.get(h)) {
        Some(h) if !h.contains_key("star.effective_temperature") => {
            h.get("orbit.semi_major_axis").and_then(as_f64)
        }
        _ => own,
    }?;
    let albedo = r
        .get("body.albedo")
        .and_then(as_f64)
        .filter(|x| (0.0..1.0).contains(x))
        .unwrap_or(s.albedo_default);
    (a > 0.0).then(|| teff * m::sqrt(r_star / (2.0 * a)) * m::pow(1.0 - albedo, 0.25))
}

/// The facts as their catalogued events left them: only events with
/// catalogued evidence, in fold order, the last value of each field kept.
fn catalogued_facts(ch: &Chronicle, system: &Address) -> Facts {
    let mut sel: Vec<&Event> = ch
        .events()
        .iter()
        .filter(|e| e.body.evidence == Evidence::Catalogued)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut out: Facts = BTreeMap::new();
    for e in sel {
        for (a, fields) in &e.body.payload.set {
            if system.contains(a) {
                let row = out.entry(a.clone()).or_default();
                for (k, v) in fields {
                    row.insert(k.clone(), v.clone());
                }
            }
        }
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
            // F5's inclination model: Rayleigh-distributed tilts make a disc
            // (sigma in degrees); isotropic ones a shell between a_min and a_max
            // (the Oort cloud; "shell" is the night sky design's Y0 name).
            let (shape, i_sigma_deg) = match c.i {
                sse_formation::leftovers::Tilt::Isotropic => ("shell", None),
                sse_formation::leftovers::Tilt::Rayleigh { sigma_deg } => ("disc", Some(sigma_deg)),
            };
            serde_json::json!({
                "shape": shape,
                "i_sigma_deg": i_sigma_deg,
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
        .map(|y| y as i64 * (sse_core::sheet::YR_JULIAN as i64));
    // Once the one clock has moved this system (its own age shift, or campaign
    // time elapsed), "now" is its present on its own dates, not the latest
    // event: a kept look-ahead's future must not read as the present.
    let moved =
        crate::clock::age_shift_myr(&all, system) != 0.0 || crate::clock::elapsed_myr(ch) != 0.0;
    let present_s = match (at_s, now_s) {
        (None, None) if moved => worlds_at(ch, system, Some(i64::MAX), run, s)?
            .spec
            .map(|sp| {
                m::round((sp.birth_year as f64 + sp.age_myr * 1.0e6) * sse_core::sheet::YR_JULIAN)
                    as i64
            }),
        _ => None,
    };
    let at_s = at_s.or(now_s).or(present_s).unwrap_or(i64::MAX);
    let mut st = ch.state_at(UniverseTime::seconds(at_s));
    // Paste support: a withdrawn body (or system) is not there.
    crate::withdraw::drop_withdrawn(&mut st.facts);
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
                .map(|sp| bodies_from_facts(&facts, &catalogued_facts(ch, system), sp, s))
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
            (a as f64 / sse_core::sheet::YR_JULIAN - now_year) / 1.0e9
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

/// SOL5: a world's measured air, from its `atmosphere.composition.<gas>`
/// facts (volume fractions, 0 to 1), for the climate's gas mix. None when
/// the record holds no composition, so the climate says it assumed one.
pub(crate) fn catalogued_gases(
    row: Option<&BTreeMap<String, Value>>,
) -> Option<BTreeMap<String, f64>> {
    let gases: BTreeMap<String, f64> = row?
        .iter()
        .filter_map(|(k, v)| {
            let gas = k.strip_prefix("atmosphere.composition.")?;
            let x = as_f64(v).filter(|x| (0.0..=1.0).contains(x))?;
            Some((gas.to_ascii_lowercase(), x))
        })
        .collect();
    (!gases.is_empty()).then_some(gases)
}

/// SW1: a world's distance from its star, AU: its own orbit when it goes
/// round a star, else its host planet's (a moon), walking up the addresses.
pub(crate) fn star_distance_au(bodies: &[sse_surface::Body], b: &sse_surface::Body) -> Option<f64> {
    let mut cur = b;
    for _ in 0..4 {
        if let Some(o) = cur
            .orbit
            .as_ref()
            .filter(|o| o.host_msun >= sse_surface::weathering::STAR_MSUN)
        {
            return Some(o.a_au);
        }
        let up = cur.address.parent()?;
        cur = bodies.iter().find(|x| x.address == up)?;
    }
    None
}

/// Orbit atlases kept so that reopening a world reuses its plate run.
pub const ORBIT_CACHE_SIZE: usize = 8;

/// Recent Orbit-level atlases by key, newest last.
pub(crate) type OrbitCache = Vec<(String, sse_surface::atlas::Atlas)>;

/// The Orbit atlas of one world from the cache, or worked out and kept.
/// The Orbit atlas cache's key: the world, seed and epoch, and the survey
/// options that change it (SOL5's measured air, SW1's star distance).
fn orbit_cache_key(
    env: &sse_surface::SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
    opts: &sse_surface::survey::SurveyOptions,
) -> String {
    format!(
        "{}/{}",
        sse_surface::detail::orbit_key(env, seed, epoch_gyr),
        serde_json::to_string(&(&opts.gases, opts.star_distance_au)).expect("plain data")
    )
}

/// The survey options a world's Orbit atlas is built with, from its record.
fn orbit_opts(
    facts: &Facts,
    bodies: &[sse_surface::Body],
    b: &sse_surface::Body,
) -> sse_surface::survey::SurveyOptions {
    sse_surface::survey::SurveyOptions {
        gases: catalogued_gases(facts.get(&b.address)),
        star_distance_au: star_distance_au(bodies, b),
        ..Default::default()
    }
}

fn orbit_atlas<'c>(
    cache: &'c mut OrbitCache,
    s: &sse_surface::SurfaceSettings,
    body: &sse_surface::Body,
    env: &sse_surface::SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
    opts: &sse_surface::survey::SurveyOptions,
) -> Result<&'c sse_surface::atlas::Atlas, String> {
    let key = orbit_cache_key(env, seed, epoch_gyr, opts);
    match cache.iter().position(|(k, _)| *k == key) {
        Some(i) => {
            let hit = cache.remove(i);
            cache.push(hit);
        }
        None => {
            let a = sse_surface::detail::orbit_atlas_with(s, body, seed, epoch_gyr, opts)?;
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
        mut facts,
        at_s,
        shift_gyr,
    } = worlds_at(ch, system, at_s, run, &s)?;
    let envelopes: Vec<sse_surface::SurfaceEnvelope> = bodies
        .iter()
        .map(|b| sse_surface::envelope(&s, b))
        .collect();
    // MF2: every world with an iron core publishes its field facts from the
    // ledger, unless the record already holds them.
    for (b, e) in bodies.iter().zip(&envelopes) {
        if let Some(ff) = sse_surface::core::field_facts(e, b) {
            let row = facts.entry(b.address.clone()).or_default();
            for (k, v) in ff {
                row.entry(k.to_string()).or_insert(f(v));
            }
        }
    }
    // MF3 part 1: a magnetised world's dipole tilt. Nothing predicts it, so
    // it is drawn from the body's address (Rayleigh, spread of the dipolar
    // planets' measured tilts), and said to be.
    for b in &bodies {
        let row = facts.entry(b.address.clone()).or_default();
        let on = row
            .get("body.magnetic_moment")
            .and_then(as_f64)
            .is_some_and(|x| x > 0.0);
        if on && !row.contains_key("field.tilt") {
            row.insert(
                "field.tilt".to_string(),
                f(drawn_tilt_deg(seed, &b.address)),
            );
            row.insert("field.tilt_grade".to_string(), t("drawn"));
        }
    }
    // Every world's moment of inertia factor (for flattening and
    // precession), unless the record holds a measured one.
    for (b, e) in bodies.iter().zip(&envelopes) {
        if let Some(c) = inertia_factor(b, e) {
            facts
                .entry(b.address.clone())
                .or_default()
                .entry("body.moment_of_inertia_factor".to_string())
                .or_insert(f(c));
        }
    }
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
                let opts = orbit_opts(&facts, &bodies, b);
                let atlas = orbit_atlas(
                    orbits,
                    &s,
                    b,
                    env,
                    seed,
                    (b.age_gyr + shift_gyr).max(0.0),
                    &opts,
                );
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
        // Each world's surface record, with its vapour plumes when it has
        // any (third audit R10: drawn above the limb in True), and how its
        // bare ground weathers (SW1) where something from space reaches it.
        surfaces: bodies
            .iter()
            .zip(&envelopes)
            .map(|(b, e)| {
                let mut v = serde_json::to_value(e).expect("plain data");
                if let Some(p) = sse_surface::plumes::plumes(b, e, seed) {
                    v["plumes"] = serde_json::to_value(p).expect("plain data");
                }
                if let Some(w) =
                    sse_surface::weathering::weathering(b, e, star_distance_au(&bodies, b))
                        .filter(|w| w.active)
                {
                    v["weathering"] = serde_json::to_value(w).expect("plain data");
                }
                // What stains its ground (TRUE4), when its tiles were read: from
                // the surveyed atlas's one climate, already in the cache.
                let key = orbit_cache_key(
                    e,
                    seed,
                    (b.age_gyr + shift_gyr).max(0.0),
                    &orbit_opts(&facts, &bodies, b),
                );
                if let Some((_, a)) = orbits.iter().find(|(k, _)| *k == key) {
                    let st = sse_surface::stains::stains_of(b, e, a);
                    if !st.is_empty() {
                        v["stains"] = serde_json::to_value(st).expect("plain data");
                    }
                }
                v
            })
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
            let cited = match &e.body.payload.data {
                Some(Value::Map(d)) if e.body.kind == sse_chronicle::kinds::AUTHORED_SET => {
                    match d.get("source") {
                        Some(Value::Text(t)) => Some(t.clone()),
                        _ => None,
                    }
                }
                _ => None,
            };
            for k in fields.keys() {
                let mut who = serde_json::json!({"evidence": evidence_label(e.body.evidence), "by": e.body.kind});
                if let Some(t) = &cited {
                    who["source"] = serde_json::Value::String(t.clone());
                }
                row.insert(k.clone(), who);
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

/// The systems of a universe (the outermost `sys:*` under the root, at any
/// depth, with facts at or under them), with where each came from and a name if one is known.
pub(crate) fn systems_of(ch: &Chronicle, root: &Address) -> Vec<serde_json::Value> {
    let mut st = ch.state_at(UniverseTime::seconds(i64::MAX));
    crate::withdraw::drop_withdrawn(&mut st.facts);
    let mut found: BTreeMap<Address, u32> = BTreeMap::new();
    for a in st.facts.keys() {
        if !root.contains(a) || a.segments().len() <= root.segments().len() {
            continue;
        }
        // The outermost `sys` at any depth under the root: a system grown
        // from a drawn star sits at `<root>/halo:…/cell:…/sys:…`.
        let mut chain = vec![a.clone()];
        while let Some(p) = chain.last().and_then(|x| x.parent()) {
            if p.segments().len() <= root.segments().len() {
                break;
            }
            chain.push(p);
        }
        let Some(sys) = chain.into_iter().rev().find(|x| x.last().kind() == "sys") else {
            continue;
        };
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

/// A world's moment of inertia factor C = I/(M R²): a giant as an n = 1
/// polytrope less its heavy-element core (`sse.interior.inertia_giant`, its
/// heavy share from the mass-metallicity relation at the default star), a
/// world with an iron core as core and mantle (`sse.interior.inertia_two_layer`).
pub(crate) fn inertia_factor(
    b: &sse_surface::Body,
    e: &sse_surface::SurfaceEnvelope,
) -> Option<f64> {
    use sse_atoms::{inertia as ia, matter as mt};
    if b.deep_envelope {
        let z = mt::heavy_element_share(
            DEFAULT_Z,
            b.mass_earth * sse_core::sheet::M_EARTH,
            sse_core::sheet::M_JUP,
            mt::GIANT_METAL_COEFFICIENT,
            mt::GIANT_METAL_SLOPE,
        );
        return Some(ia::giant(z, ia::CORE_SHARE));
    }
    let c = e.core.as_ref()?;
    let r = b.radius_earth * R_EARTH;
    (r > 0.0).then(|| ia::two_layer(b.core_mass_fraction, c.core_radius_km * 1000.0, r))
}

/// Spread of the dipolar planets' dipole tilts, degrees (Earth 9.4,
/// Jupiter 9.6, Ganymede 4; Stevenson 2003, Earth Planet. Sci. Lett. 208, 1).
pub(crate) const TILT_SPREAD_DEG: f64 = 10.0;

/// A keyed Rayleigh draw of a dipole's tilt from the spin axis, degrees.
pub(crate) fn drawn_tilt_deg(seed: &Seed, address: &Address) -> f64 {
    let stream = sse_core::draw::Stream::new("mf3.tilt.v1").expect("stream name");
    let d = sse_core::draw::Draws::new(*seed, address.clone(), stream);
    let u = d.unit(0).clamp(0.0, 1.0 - f64::EPSILON);
    (TILT_SPREAD_DEG * m::sqrt(-2.0 * m::ln(1.0 - u))).min(90.0)
}
