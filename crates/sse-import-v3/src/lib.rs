//! One-way importer for Star System Explorer 3 saves (build plan P3).
//!
//! The rule that decides every field (docs/v3-save-fields.md): V4 stores
//! authored facts only. V3 wrote many derived values into its saves; the
//! importer never takes those as truth. It recomputes what the first-wave
//! atoms can and lists every difference in the report, and it names every
//! derived field it saw but could not yet compare. Nothing is dropped
//! silently: a field the importer does not know is kept as an authored
//! `v3.<name>` fact and listed as unclassified.
#![forbid(unsafe_code)]

mod classify;

use classify::{body_class, starmap_class, system_class, Class};
use serde::Serialize;
use serde_json::{Map, Value as Json};
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, SessionStamp, UniverseTime,
};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, G};
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;
use std::fmt;

pub const IMPORTER_VERSION: &str = "0.1.0";
pub const REPORT_FORMAT: &str = "sse.v3-import-report/1";

#[derive(Clone, Debug, PartialEq)]
pub struct ImportError(pub String);
impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ImportError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub value: Json,
    pub evidence: Evidence,
}

/// One derived V3 value recomputed by V4.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Difference {
    pub address: String,
    pub name: String,
    pub field: String,
    pub v3: f64,
    pub v4: f64,
    /// |v4 - v3| / |v3|.
    pub relative: f64,
    pub tolerance: f64,
    pub within: bool,
    /// The V4 atom that produced `v4`.
    pub via: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Default)]
pub struct Counts {
    pub systems: u32,
    pub bodies: u32,
    pub barycentres: u32,
    pub constructs: u32,
    pub facts: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub format: String,
    pub importer_version: String,
    /// "starmap" or "system".
    pub source_kind: String,
    pub v3_app_version: Option<String>,
    pub bundle_format: Option<i64>,
    pub counts: Counts,
    /// Every compared derived value, in address order.
    pub differences: Vec<Difference>,
    /// Derived V3 fields seen and not imported, by name, with how often.
    pub derived_not_compared: BTreeMap<String, u32>,
    /// Presentation fields, for view settings, not the chronicle.
    pub presentation: BTreeMap<String, u32>,
    /// V3 bookkeeping read and ignored.
    pub dropped: BTreeMap<String, u32>,
    /// Fields the importer does not know: kept as authored `v3.<name>`.
    pub unclassified: BTreeMap<String, u32>,
    /// Addresses carrying V3 overrides or anomalies (authored-impossible).
    pub honesty_marks: Vec<String>,
    pub warnings: Vec<String>,
    /// Hash of the chronicle state the import produces.
    pub state_hash: String,
}

impl Report {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("report serialises") + "\n"
    }
    pub fn differences_outside_tolerance(&self) -> usize {
        self.differences.iter().filter(|d| !d.within).count()
    }
}

pub struct Import {
    pub universe: Address,
    pub facts: BTreeMap<Address, BTreeMap<String, Fact>>,
    pub report: Report,
}

impl Import {
    /// Authored events for the chronicle: one per address and evidence class,
    /// in address order, all at the universe origin.
    pub fn events(&self) -> Vec<Event> {
        let mut out = Vec::new();
        let mut stamp = 0u64;
        for (a, fields) in &self.facts {
            for ev in [
                Evidence::Catalogued,
                Evidence::Authored,
                Evidence::AuthoredImpossible,
            ] {
                let set: BTreeMap<String, sse_core::cbor::Value> = fields
                    .iter()
                    .filter(|(_, f)| f.evidence == ev)
                    .map(|(k, f)| (k.clone(), to_value(&f.value)))
                    .collect();
                if set.is_empty() {
                    continue;
                }
                stamp += 1;
                let mut s = BTreeMap::new();
                s.insert(a.clone(), set);
                out.push(
                    Event::new(EventBody {
                        kind: kinds::AUTHORED_SET.into(),
                        subject: vec![a.clone()],
                        occurred_at: UniverseTime::ORIGIN,
                        activated_at: UniverseTime::ORIGIN,
                        recognised_at: SessionStamp(stamp),
                        causes: vec![],
                        payload: Payload {
                            set: s,
                            ..Default::default()
                        },
                        evidence: ev,
                        publisher: Publisher::new("sse.import-v3", "importer", IMPORTER_VERSION),
                    })
                    .expect("import events are well formed"),
                );
            }
        }
        out
    }

    pub fn chronicle(&self) -> Chronicle {
        let mut c = Chronicle::new();
        for e in self.events() {
            c.append(e).expect("import events append in order");
        }
        c
    }

    pub fn state_hash(&self) -> ContentHash {
        self.chronicle()
            .state_at(UniverseTime::seconds(i64::MAX))
            .hash()
    }
}

fn to_value(j: &Json) -> sse_core::cbor::Value {
    serde_json::from_value(j.clone()).expect("JSON from a parsed file has no NaN")
}

/// Import a V3 `starmap.json` or `system.json` (the plain JSON save, or the
/// file of that name inside a `.sse.zip`). `universe_id` names the V4 universe.
pub fn import_json(text: &str, universe_id: &str) -> Result<Import, ImportError> {
    let root: Json =
        serde_json::from_str(text).map_err(|e| ImportError(format!("not JSON: {e}")))?;
    let obj = root
        .as_object()
        .ok_or_else(|| ImportError("a V3 save is a JSON object".into()))?;
    let universe = Address::universe(universe_id)
        .map_err(|e| ImportError(format!("universe id {universe_id:?}: {e}")))?;
    let mut im = Importer::new(universe.clone());
    let source_kind = if obj.contains_key("systems") {
        im.starmap(obj)?;
        "starmap"
    } else if obj.contains_key("nodes") {
        im.system(&universe, 0, obj, None)?;
        "system"
    } else {
        return Err(ImportError(
            "neither a starmap (no \"systems\") nor a system (no \"nodes\")".into(),
        ));
    };
    let bundle_format = match obj.get("bundleFormat") {
        None => None,
        Some(v) => match v.as_i64() {
            Some(n) => Some(n),
            None => return Err(ImportError("bundleFormat is not an integer".into())),
        },
    };
    let mut report = im.report;
    report.source_kind = source_kind.into();
    report.bundle_format = bundle_format;
    report.v3_app_version = obj
        .get("appVersion")
        .and_then(Json::as_str)
        .map(str::to_owned);
    report.counts.facts = im.facts.values().map(|f| f.len() as u32).sum();
    let mut out = Import {
        universe,
        facts: im.facts,
        report,
    };
    out.report.state_hash = out.state_hash().hex();
    Ok(out)
}

struct Importer {
    universe: Address,
    facts: BTreeMap<Address, BTreeMap<String, Fact>>,
    report: Report,
}

fn bump(m: &mut BTreeMap<String, u32>, k: &str) {
    *m.entry(k.to_owned()).or_insert(0) += 1;
}

fn num(o: &Map<String, Json>, k: &str) -> Option<f64> {
    o.get(k).and_then(Json::as_f64)
}

impl Importer {
    fn new(universe: Address) -> Self {
        Importer {
            universe,
            facts: BTreeMap::new(),
            report: Report {
                format: REPORT_FORMAT.into(),
                importer_version: IMPORTER_VERSION.into(),
                source_kind: String::new(),
                v3_app_version: None,
                bundle_format: None,
                counts: Counts::default(),
                differences: vec![],
                derived_not_compared: BTreeMap::new(),
                presentation: BTreeMap::new(),
                dropped: BTreeMap::new(),
                unclassified: BTreeMap::new(),
                honesty_marks: vec![],
                warnings: vec![],
                state_hash: String::new(),
            },
        }
    }

    fn set(&mut self, a: &Address, field: &str, value: Json, evidence: Evidence) {
        self.facts
            .entry(a.clone())
            .or_default()
            .insert(field.to_owned(), Fact { value, evidence });
    }

    fn starmap(&mut self, o: &Map<String, Json>) -> Result<(), ImportError> {
        let u = self.universe.clone();
        for (k, v) in o {
            match starmap_class(k) {
                Class::Structural => {}
                Class::Authored => self.set(&u, &format!("v3.{k}"), v.clone(), Evidence::Authored),
                Class::Presentation => bump(&mut self.report.presentation, k),
                Class::Dropped => bump(&mut self.report.dropped, k),
                Class::Derived => bump(&mut self.report.derived_not_compared, k),
                Class::Unknown => {
                    bump(&mut self.report.unclassified, k);
                    self.set(&u, &format!("v3.{k}"), v.clone(), Evidence::Authored);
                }
                Class::Impossible => self.set(
                    &u,
                    &format!("v3.{k}"),
                    v.clone(),
                    Evidence::AuthoredImpossible,
                ),
            }
        }
        let systems = o["systems"]
            .as_array()
            .ok_or_else(|| ImportError("\"systems\" is not a list".into()))?;
        for (i, s) in systems.iter().enumerate() {
            let s = s
                .as_object()
                .ok_or_else(|| ImportError(format!("systems[{i}] is not an object")))?;
            let inner = s
                .get("system")
                .and_then(Json::as_object)
                .ok_or_else(|| ImportError(format!("systems[{i}] has no \"system\" object")))?;
            self.system(&u, i, inner, Some(s))?;
        }
        Ok(())
    }

    /// One V3 system. `node` is the StarSystemNode wrapper when inside a starmap.
    fn system(
        &mut self,
        u: &Address,
        index: usize,
        o: &Map<String, Json>,
        node: Option<&Map<String, Json>>,
    ) -> Result<(), ImportError> {
        let sa = u
            .authored_child("sys", index as u64 + 1)
            .map_err(|e| ImportError(e.to_string()))?;
        self.report.counts.systems += 1;
        if let Some(n) = node {
            for (k, v) in n {
                match k.as_str() {
                    "system" => {}
                    "id" | "name" | "isNameUserDefined" | "subsectorId" => {
                        self.set(&sa, &format!("v3.node.{k}"), v.clone(), Evidence::Authored)
                    }
                    "position" => self.set(
                        &sa,
                        "system.position_map_units",
                        v.clone(),
                        Evidence::Authored,
                    ),
                    "viewport" | "time" => {
                        bump(&mut self.report.presentation, &format!("system.{k}"))
                    }
                    _ => {
                        bump(&mut self.report.unclassified, &format!("system node.{k}"));
                        self.set(&sa, &format!("v3.node.{k}"), v.clone(), Evidence::Authored);
                    }
                }
            }
        }
        for (k, v) in o {
            match system_class(k) {
                Class::Structural => {}
                Class::Authored => self.set(&sa, &format!("v3.{k}"), v.clone(), Evidence::Authored),
                Class::Presentation => bump(&mut self.report.presentation, &format!("system.{k}")),
                Class::Dropped => bump(&mut self.report.dropped, &format!("system.{k}")),
                Class::Derived => bump(
                    &mut self.report.derived_not_compared,
                    &format!("system.{k}"),
                ),
                Class::Impossible => self.set(
                    &sa,
                    &format!("v3.{k}"),
                    v.clone(),
                    Evidence::AuthoredImpossible,
                ),
                Class::Unknown => {
                    bump(&mut self.report.unclassified, &format!("system.{k}"));
                    self.set(&sa, &format!("v3.{k}"), v.clone(), Evidence::Authored);
                }
            }
        }
        if let Some(age) = num(o, "age_Gyr") {
            self.set(
                &sa,
                "system.age",
                Json::from(age * 1e9 * 365.25 * 86400.0),
                Evidence::Authored,
            );
        }
        let nodes = o
            .get("nodes")
            .and_then(Json::as_array)
            .ok_or_else(|| ImportError(format!("system {index}: \"nodes\" is not a list")))?;
        // Address every node first, so parents and hosts resolve in any order.
        let mut ids: BTreeMap<String, Address> = BTreeMap::new();
        let mut addrs = Vec::new();
        let (mut nb, mut nbar, mut nc) = (0u64, 0u64, 0u64);
        for (j, n) in nodes.iter().enumerate() {
            let n = n
                .as_object()
                .ok_or_else(|| ImportError(format!("system {index} node {j} is not an object")))?;
            let kind = n.get("kind").and_then(Json::as_str).unwrap_or("body");
            let (seg, k) = match kind {
                "barycenter" => {
                    nbar += 1;
                    self.report.counts.barycentres += 1;
                    ("bary", nbar)
                }
                "construct" => {
                    nc += 1;
                    self.report.counts.constructs += 1;
                    ("craft", nc)
                }
                _ => {
                    nb += 1;
                    self.report.counts.bodies += 1;
                    ("body", nb)
                }
            };
            let a = sa
                .authored_child(seg, k)
                .map_err(|e| ImportError(e.to_string()))?;
            if let Some(id) = n.get("id").and_then(Json::as_str) {
                if ids.insert(id.to_owned(), a.clone()).is_some() {
                    self.report
                        .warnings
                        .push(format!("system {index}: node id {id:?} appears twice"));
                }
            }
            addrs.push((a, n));
        }
        let by_id: BTreeMap<String, &Map<String, Json>> = addrs
            .iter()
            .filter_map(|(_, n)| {
                n.get("id")
                    .and_then(Json::as_str)
                    .map(|id| (id.to_owned(), *n))
            })
            .collect();
        for (a, n) in &addrs {
            self.node(a, n, &ids, &by_id);
        }
        Ok(())
    }

    fn node(
        &mut self,
        a: &Address,
        n: &Map<String, Json>,
        ids: &BTreeMap<String, Address>,
        by_id: &BTreeMap<String, &Map<String, Json>>,
    ) {
        let role = n.get("roleHint").and_then(Json::as_str).unwrap_or("");
        let is_star = role == "star";
        for (k, v) in n {
            match k.as_str() {
                "massKg" => self.quantity(a, "body.mass", v, 1.0),
                "radiusKm" => self.quantity(a, "body.radius", v, 1e3),
                "radiusInnerKm" => self.quantity(a, "body.radius_inner", v, 1e3),
                "radiusOuterKm" => self.quantity(a, "body.radius_outer", v, 1e3),
                "rotation_period_hours" => self.quantity(a, "body.rotation_period", v, 3600.0),
                "axial_tilt_deg" => self.quantity(a, "body.axial_tilt", v, m::PI / 180.0),
                "obliquity_deg" => self.quantity(a, "body.obliquity", v, m::PI / 180.0),
                "temperatureK" if is_star => self.quantity(a, "star.effective_temperature", v, 1.0),
                "parentId" => {
                    if let Some(p) = v.as_str() {
                        match ids.get(p) {
                            Some(pa) => self.set(
                                a,
                                "parent",
                                Json::from(pa.to_string()),
                                Evidence::Authored,
                            ),
                            None => self
                                .report
                                .warnings
                                .push(format!("{a}: parent {p:?} not found")),
                        }
                    }
                }
                "orbit" => self.orbit(a, n, v, ids, by_id),
                "tags" => self.tags(a, v),
                _ => match body_class(k) {
                    Class::Structural => {}
                    Class::Authored => {
                        self.set(a, &format!("v3.{k}"), v.clone(), Evidence::Authored)
                    }
                    Class::Impossible => {
                        let empty = v.is_null()
                            || v.as_array().is_some_and(Vec::is_empty)
                            || v.as_object().is_some_and(Map::is_empty);
                        if !empty {
                            self.set(
                                a,
                                &format!("v3.{k}"),
                                v.clone(),
                                Evidence::AuthoredImpossible,
                            );
                            let s = a.to_string();
                            if !self.report.honesty_marks.contains(&s) {
                                self.report.honesty_marks.push(s);
                            }
                        }
                    }
                    Class::Presentation => bump(&mut self.report.presentation, k),
                    Class::Dropped => bump(&mut self.report.dropped, k),
                    Class::Derived => {
                        if !self.compare(a, n, k, by_id) {
                            bump(&mut self.report.derived_not_compared, k);
                        }
                    }
                    Class::Unknown => {
                        bump(&mut self.report.unclassified, k);
                        self.set(a, &format!("v3.{k}"), v.clone(), Evidence::Authored);
                    }
                },
            }
        }
    }

    fn quantity(&mut self, a: &Address, field: &str, v: &Json, scale: f64) {
        match v.as_f64() {
            Some(x) => self.set(a, field, Json::from(x * scale), Evidence::Authored),
            None if v.is_null() => {}
            None => self
                .report
                .warnings
                .push(format!("{a}: {field} is not a number ({v})")),
        }
    }

    fn tags(&mut self, a: &Address, v: &Json) {
        let Some(list) = v.as_array() else {
            self.report
                .warnings
                .push(format!("{a}: tags is not a list"));
            return;
        };
        let mut kept = Vec::new();
        for t in list {
            match t.get("origin").and_then(Json::as_str) {
                Some("physics") | Some("rule") | Some("derived") => {
                    if t.get("override").and_then(Json::as_bool) == Some(true) {
                        kept.push(t.clone());
                    } else {
                        bump(
                            &mut self.report.derived_not_compared,
                            "tags (physics, rule or derived origin)",
                        );
                    }
                }
                _ => kept.push(t.clone()),
            }
        }
        if !kept.is_empty() {
            self.set(a, "v3.tags", Json::from(kept), Evidence::Authored);
        }
    }

    fn orbit(
        &mut self,
        a: &Address,
        n: &Map<String, Json>,
        v: &Json,
        ids: &BTreeMap<String, Address>,
        by_id: &BTreeMap<String, &Map<String, Json>>,
    ) {
        let Some(o) = v.as_object() else {
            if !v.is_null() {
                self.report
                    .warnings
                    .push(format!("{a}: orbit is not an object"));
            }
            return;
        };
        let derived_orbit = n.contains_key("coOrbital");
        let ev = Evidence::Authored;
        let host = o.get("hostId").and_then(Json::as_str);
        if let Some(h) = host {
            match ids.get(h) {
                Some(ha) => self.set(a, "orbit.host", Json::from(ha.to_string()), ev),
                None => self
                    .report
                    .warnings
                    .push(format!("{a}: orbit host {h:?} not found")),
            }
        }
        if derived_orbit {
            // V3 derives a Lagrange-point orbit from its secondary every pass.
            bump(
                &mut self.report.derived_not_compared,
                "orbit (co-orbital, derived)",
            );
            return;
        }
        if let Some(el) = o.get("elements").and_then(Json::as_object) {
            let deg = m::PI / 180.0;
            for (k3, k4, s) in [
                ("a_AU", "orbit.semi_major_axis", AU),
                ("e", "orbit.eccentricity", 1.0),
                ("i_deg", "orbit.inclination", deg),
                ("Omega_deg", "orbit.ascending_node", deg),
                ("omega_deg", "orbit.argument_of_periapsis", deg),
                ("M0_rad", "orbit.mean_anomaly_at_epoch", 1.0),
            ] {
                if let Some(x) = num(el, k3) {
                    self.set(a, k4, Json::from(x * s), ev);
                }
            }
        }
        for (k, val) in o {
            match k.as_str() {
                "hostId" | "elements" => {}
                "t0" => self.set(a, "orbit.epoch_v3_ms", val.clone(), ev),
                "isRetrogradeOrbit" | "resonance" | "frame" => {
                    self.set(a, &format!("orbit.v3.{k}"), val.clone(), ev)
                }
                "seed" | "lastEditedT0" => bump(&mut self.report.dropped, &format!("orbit.{k}")),
                "hostMu" | "n_rad_per_s" => {
                    if !self.compare_orbit(a, n, o, k, host.and_then(|h| by_id.get(h).copied())) {
                        bump(&mut self.report.derived_not_compared, &format!("orbit.{k}"));
                    }
                }
                _ => {
                    bump(&mut self.report.unclassified, &format!("orbit.{k}"));
                    self.set(a, &format!("orbit.v3.{k}"), val.clone(), ev);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn diff(
        &mut self,
        a: &Address,
        n: &Map<String, Json>,
        field: &str,
        v3: f64,
        v4: f64,
        tol: f64,
        via: &str,
    ) {
        let relative = if v3 == 0.0 {
            (v4 - v3).abs()
        } else {
            ((v4 - v3) / v3).abs()
        };
        self.report.differences.push(Difference {
            address: a.to_string(),
            name: n
                .get("name")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_owned(),
            field: field.to_owned(),
            v3,
            v4,
            relative,
            tolerance: tol,
            within: relative <= tol,
            via: via.to_owned(),
        });
    }

    /// Recompute a derived body field with V4 atoms. False when it cannot.
    fn compare(
        &mut self,
        a: &Address,
        n: &Map<String, Json>,
        k: &str,
        _by_id: &BTreeMap<String, &Map<String, Json>>,
    ) -> bool {
        let (Some(mass), Some(rkm)) = (num(n, "massKg"), num(n, "radiusKm")) else {
            return false;
        };
        if mass <= 0.0 || rkm <= 0.0 {
            return false;
        }
        let atoms = sse_atoms::registry();
        let eval = |id: &str| -> Option<f64> {
            let at = atoms.get(id)?;
            let p: Vec<f64> = at.decl().params.iter().map(|p| p.default).collect();
            at.eval(&[mass, rkm * 1e3], &p).ok().map(|v| v[0])
        };
        match k {
            "densityGcm3" => {
                let Some(v3) = num(n, k) else { return false };
                let Some(v4) = eval("sse.planet.bulk_density") else {
                    return false;
                };
                self.diff(a, n, k, v3, v4 / 1e3, 0.01, "sse.planet.bulk_density");
                true
            }
            "calculatedGravity_ms2" => {
                let Some(v3) = num(n, k) else { return false };
                let Some(v4) = eval("sse.planet.surface_gravity") else {
                    return false;
                };
                self.diff(a, n, k, v3, v4, 0.01, "sse.planet.surface_gravity");
                true
            }
            _ => false,
        }
    }

    fn compare_orbit(
        &mut self,
        a: &Address,
        n: &Map<String, Json>,
        o: &Map<String, Json>,
        k: &str,
        host: Option<&Map<String, Json>>,
    ) -> bool {
        let Some(v3) = num(o, k) else { return false };
        let Some(hm) = host.and_then(|h| num(h, "massKg")) else {
            return false;
        };
        let field = format!("orbit.{k}");
        match k {
            "hostMu" => {
                let v4 = G * hm;
                // V3 orbits carry hostMu in m³/s² (orbits.ts). A value that matches in
                // km³/s² is a V3 data slip (the real Lunar Gateway, 18 Nov 2026): V4
                // recomputes it, so say so in the report rather than leave it unexplained.
                let via = if ((v3 * 1e9 - v4) / v4).abs() <= 1e-3 {
                    "G × host mass; V3 value is in km³/s², a V3 data slip (V3 uses m³/s²)"
                } else {
                    "G × host mass"
                };
                self.diff(a, n, &field, v3, v4, 1e-3, via);
                true
            }
            "n_rad_per_s" => {
                let Some(aau) = o
                    .get("elements")
                    .and_then(|e| e.get("a_AU"))
                    .and_then(Json::as_f64)
                else {
                    return false;
                };
                let bm = num(n, "massKg").unwrap_or(0.0);
                let atoms = sse_atoms::registry();
                let Some(at) = atoms.get("sse.orbit.period") else {
                    return false;
                };
                let Ok(p) = at.eval(&[aau * AU, hm + bm], &[]) else {
                    return false;
                };
                self.diff(a, n, &field, v3, m::TAU / p[0], 5e-3, "sse.orbit.period");
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
