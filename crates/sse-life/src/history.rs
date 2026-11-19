//! Boxes C, D and E: origin, innovations and catastrophes, as a dated
//! history (life design sections 4 to 6, stage L1b).
//!
//! A world is given as epochs (a habitat graph and world facts from a start
//! age) and physical catastrophes (each with an id, the cause later events
//! point at). Stepping through the world's life in fixed steps:
//! - **C, origin**: each open habitat has an origin hazard = the origin dial
//!   x base rate x the sum of its settings' factors x its area / Earth's
//!   surface. Origin happens when the accumulated hazard passes -ln u, u a
//!   keyed draw per habitat and attempt, so a bigger dial never makes it later.
//!   Life then holds every open habitat of the world (spread is fast).
//! - **D, innovations**: opportunity accumulates per Myr as (production /
//!   Earth's)^exponent x a metabolic rate factor (exp(-E/kT) against 288 K)
//!   x habitat variety (open habitat kinds / reference). A capability whose
//!   prerequisites (capabilities and world facts) hold is acquired when its
//!   rate x dial x accumulated opportunity passes its own keyed threshold.
//!   Capabilities unlock metabolisms and media, which change box B's budget.
//! - **E, catastrophes**: a physical event removes a share of diversity in
//!   exposed habitats; capabilities younger than `fragile_myr` may be lost
//!   (keyed draw); a total event with no living refuge sterilises the world,
//!   and origin may happen again. Diversity relaxes back towards a carrying
//!   capacity set by production; a recovery event marks 90%.
//!
//! Every event lists its causes: earlier events, the input catastrophe, or
//! the epoch and setting that allowed it. Dials used are stamped on the result.

use crate::budget::{self, BudgetParams, WorldBudget};
use crate::habitat::HabitatGraph;
use crate::*;
use sse_core::math as m;
use sse_core::{Draws, Seed, Stream};

/// An origin setting (`provides.other.life_history.settings`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OriginSetting {
    pub id: String,
    pub label: String,
    /// Habitat kinds that offer it.
    pub habitats: Vec<String>,
    pub factor: f64,
    #[serde(default = "yes")]
    pub enabled: bool,
}
fn yes() -> bool {
    true
}

/// A world fact a capability needs: `fact` at least `min`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Need {
    pub fact: String,
    pub min: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub needs: Vec<String>,
    #[serde(default)]
    pub needs_facts: Vec<Need>,
    /// Acquisitions per unit of opportunity (Earth-Myr).
    pub rate: f64,
    /// The dial that scales `rate`, for contested hard steps.
    #[serde(default)]
    pub dial: Option<String>,
    #[serde(default)]
    pub unlocks_metabolisms: Vec<String>,
    #[serde(default)]
    pub unlocks_media: Vec<String>,
    /// Given at origin.
    #[serde(default)]
    pub at_origin: bool,
    pub effect: String,
}

/// Box C, D and E settings (`provides.other.life_history`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HistoryParams {
    pub step_myr: f64,
    /// Origin hazard per Myr for one Earth surface of a factor-1 setting.
    pub origin_rate_per_myr: f64,
    pub earth_area_m2: f64,
    pub settings: Vec<OriginSetting>,
    /// Media life holds from origin (others come with capabilities).
    pub origin_media: Vec<String>,
    pub production_ref_gc_yr: f64,
    pub population_exponent: f64,
    pub activation_ev: f64,
    pub reference_k: f64,
    pub variety_ref: f64,
    pub capabilities: Vec<Capability>,
    /// Dials by name (1 = the default pace).
    pub dials: BTreeMap<String, f64>,
    pub fragile_myr: f64,
    pub recovery_myr: f64,
    pub diversity_floor_gc_yr: f64,
    /// World fact that a high-power metabolism must exist for (an oxygen
    /// atmosphere): its epoch value counts only once one does.
    pub oxidant_fact: String,
    pub history_every_myr: f64,
}

impl HistoryParams {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("life_history"))
            .ok_or("no pack provides life_history")?;
        serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_history: {e}"))
    }
    pub fn dial(&self, name: &str) -> f64 {
        self.dials.get(name).copied().unwrap_or(1.0)
    }
}

/// The world in one period, from `from_myr` (world age) on.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Epoch {
    pub id: String,
    pub from_myr: f64,
    /// `None`: nothing habitable (no oceans yet, or gone).
    pub graph: Option<HabitatGraph>,
    #[serde(default)]
    pub facts: BTreeMap<String, f64>,
}

/// A physical catastrophe from the world's chronicle.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Catastrophe {
    pub id: String,
    pub at_myr: f64,
    pub kind: String,
    /// Share of diversity lost in exposed habitats; 1 kills every exposed one.
    pub severity: f64,
    /// Habitat kinds sheltered from it.
    #[serde(default)]
    pub refuges: Vec<String>,
    pub why: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldTimeline {
    pub body: Address,
    pub end_myr: f64,
    pub epochs: Vec<Epoch>,
    #[serde(default)]
    pub catastrophes: Vec<Catastrophe>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeEvent {
    pub id: String,
    pub at_myr: f64,
    /// `origin`, `capability`, `capability_lost`, `catastrophe`, `recovery`,
    /// `sterilised`.
    pub kind: String,
    pub subject: String,
    pub causes: Vec<String>,
    pub why: String,
    /// Dials this event depends on.
    pub dials: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub at_myr: f64,
    pub alive: bool,
    pub capabilities: Vec<String>,
    pub production_gc_yr: f64,
    pub diversity: f64,
    pub opportunity: f64,
    /// World facts from the physics boundary (coupled runs only).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub world: BTreeMap<String, f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeHistory {
    pub body: Address,
    pub events: Vec<LifeEvent>,
    pub points: Vec<HistoryPoint>,
    /// First acquisition of each capability, world age in Myr.
    pub first: BTreeMap<String, f64>,
    pub origin_myr: Option<f64>,
    pub dials: BTreeMap<String, f64>,
    pub evidence: LifeEvidence,
    /// Habitats whose open/closed state could not be judged.
    pub unknown: Vec<String>,
}

impl LifeHistory {
    pub fn has(&self, cap: &str) -> bool {
        self.first.contains_key(cap)
    }
}

fn push(events: &mut Vec<LifeEvent>, mut e: LifeEvent) -> String {
    e.id = format!("life:{}", events.len());
    let id = e.id.clone();
    events.push(e);
    id
}

fn mean_temperature(g: &HabitatGraph, open: &[usize]) -> Option<f64> {
    let ts: Vec<f64> = open
        .iter()
        .filter_map(|&i| {
            g.habitats[i]
                .profile
                .temperature_k
                .map(|t| 0.5 * (t.lo + t.hi))
        })
        .collect();
    (!ts.is_empty()).then(|| ts.iter().sum::<f64>() / ts.len() as f64)
}

struct Life {
    caps: BTreeMap<String, (f64, String)>,
    opportunity: f64,
    diversity: f64,
    origin_event: String,
}

/// Run boxes C, D and E over a world's timeline.
/// What life hands the physics side at a time boundary (L2, box H): the
/// last step's production by metabolism. The life suite never sets the
/// planet's state; it only reports these.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct StepFluxes {
    pub at_myr: f64,
    pub dt_myr: f64,
    pub alive: bool,
    pub production_gc_yr: f64,
    pub by_metabolism: BTreeMap<String, f64>,
    /// Life holds the land (weathering enhancement, cover).
    pub on_land: bool,
}

/// The physics side of a coupled run: at each step's start it receives the
/// previous step's fluxes and returns world facts (such as `o2_fraction`)
/// that override the epoch's own for this step.
pub trait Boundary {
    fn step(&mut self, fluxes: &StepFluxes) -> BTreeMap<String, f64>;
}

pub fn history(
    w: &WorldTimeline,
    bp: &BudgetParams,
    hp: &HistoryParams,
    seed: Seed,
) -> Result<LifeHistory, String> {
    history_with(w, bp, hp, seed, None)
}

/// As `history`, coupled to a physics boundary (L2) when one is given.
pub fn history_with(
    w: &WorldTimeline,
    bp: &BudgetParams,
    hp: &HistoryParams,
    seed: Seed,
    mut boundary: Option<&mut dyn Boundary>,
) -> Result<LifeHistory, String> {
    let mut last = StepFluxes::default();
    let mut world: BTreeMap<String, f64> = BTreeMap::new();
    let k_ev = 8.617_333e-5;
    let origin_draws = Stream::new("life.origin.v1").map_err(|e| e.to_string())?;
    let cap_stream = Stream::new("life.capability.v1").map_err(|e| e.to_string())?;
    let loss_stream = Stream::new("life.loss.v1").map_err(|e| e.to_string())?;
    let cap_draws = Draws::new(seed, w.body.clone(), cap_stream);
    let loss_draws = Draws::new(seed, w.body.clone(), loss_stream);
    let mut events: Vec<LifeEvent> = vec![];
    let mut points = vec![];
    let mut first = BTreeMap::new();
    let mut origin_myr = None;
    let mut life: Option<Life> = None;
    let mut hazard: BTreeMap<String, f64> = BTreeMap::new();
    let mut attempt: u64 = 0;
    let mut cap_attempt: BTreeMap<String, u64> = BTreeMap::new();
    let mut cap_progress: BTreeMap<String, f64> = BTreeMap::new();
    let mut pending_recovery: Option<String> = None;
    let mut unknown = vec![];
    let mut cache: BTreeMap<String, WorldBudget> = BTreeMap::new();
    let mut next_cat = 0;
    let mut cats = w.catastrophes.clone();
    cats.sort_by(|a, b| a.at_myr.total_cmp(&b.at_myr));
    let steps = m::ceil(w.end_myr / hp.step_myr) as u64;
    let every = m::round(hp.history_every_myr / hp.step_myr).max(1.0) as u64;
    let start = w.epochs.first().map_or(0.0, |e| e.from_myr);
    for s in 0..steps {
        let t0 = start.max(0.0) + hp.step_myr * s as f64;
        if t0 >= w.end_myr {
            break;
        }
        let t1 = (t0 + hp.step_myr).min(w.end_myr);
        let dt = t1 - t0;
        if let Some(b) = boundary.as_mut() {
            last.at_myr = t0;
            last.dt_myr = dt;
            world = b.step(&last);
            last = StepFluxes::default();
        }
        let Some((ei, ep)) = w
            .epochs
            .iter()
            .enumerate()
            .rev()
            .find(|(_, e)| e.from_myr <= t0)
        else {
            continue;
        };
        // Open habitats in this epoch.
        let open: Vec<usize> = match &ep.graph {
            None => vec![],
            Some(g) => g
                .habitats
                .iter()
                .enumerate()
                .filter(|(_, h)| budget::closed_reason(&h.profile, bp).is_none())
                .map(|(i, _)| i)
                .collect(),
        };
        if let Some(g) = &ep.graph {
            for h in &g.habitats {
                if h.profile.temperature_k.is_none() {
                    let a = h.address.to_string();
                    if !unknown.contains(&a) {
                        unknown.push(a);
                    }
                }
            }
        }
        // Box E: catastrophes in this step.
        while next_cat < cats.len() && cats[next_cat].at_myr < t1 {
            let c = cats[next_cat].clone();
            let ci = next_cat;
            next_cat += 1;
            if let Some(l) = life.as_mut() {
                let g = ep.graph.as_ref();
                let total = open.len().max(1) as f64;
                let exposed = open
                    .iter()
                    .filter(|&&i| g.is_some_and(|g| !c.refuges.contains(&g.habitats[i].kind)))
                    .count() as f64;
                let share = c.severity.clamp(0.0, 1.0) * exposed / total;
                let cid = push(
                    &mut events,
                    LifeEvent {
                        id: String::new(),
                        at_myr: c.at_myr,
                        kind: "catastrophe".into(),
                        subject: w.body.to_string(),
                        causes: vec![c.id.clone()],
                        why: format!(
                            "{}: {:.0}% of diversity lost ({} of {} living habitats exposed)",
                            c.why,
                            share * 100.0,
                            exposed,
                            total
                        ),
                        dials: vec![],
                    },
                );
                l.diversity *= 1.0 - share;
                pending_recovery = Some(cid.clone());
                let refuge = open.len() as f64 > exposed;
                if c.severity >= 1.0 && !refuge {
                    push(
                        &mut events,
                        LifeEvent {
                            id: String::new(),
                            at_myr: c.at_myr,
                            kind: "sterilised".into(),
                            subject: w.body.to_string(),
                            causes: vec![cid],
                            why: "no living habitat was sheltered from it".into(),
                            dials: vec![],
                        },
                    );
                    life = None;
                    hazard.clear();
                    attempt += 1;
                    pending_recovery = None;
                    continue;
                }
                let young: Vec<String> = l
                    .caps
                    .iter()
                    .filter(|(id, (since, _))| {
                        c.at_myr - since < hp.fragile_myr
                            && !hp.capabilities.iter().any(|x| &x.id == *id && x.at_origin)
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                for (k, id) in young.iter().enumerate() {
                    let u = loss_draws.unit(ci as u64 * 64 + k as u64);
                    if u < share {
                        l.caps.remove(id);
                        cap_progress.remove(id);
                        *cap_attempt.entry(id.clone()).or_default() += 1;
                        push(
                            &mut events,
                            LifeEvent {
                                id: String::new(),
                                at_myr: c.at_myr,
                                kind: "capability_lost".into(),
                                subject: id.clone(),
                                causes: vec![cid.clone()],
                                why: format!(
                                    "{id} was younger than {:.0} Myr, held by few lineages, and did not survive",
                                    hp.fragile_myr
                                ),
                                dials: vec![],
                            },
                        );
                    }
                }
            }
        }
        let Some(g) = &ep.graph else {
            continue;
        };
        // Box C: origin.
        if life.is_none() && !open.is_empty() {
            let dial = hp.dial("origin");
            let mut born: Option<(String, String)> = None;
            for &i in &open {
                let h = &g.habitats[i];
                let area = h.profile.area_m2.unwrap_or(0.0);
                let f: f64 = hp
                    .settings
                    .iter()
                    .filter(|st| st.enabled && st.habitats.contains(&h.kind))
                    .map(|st| st.factor)
                    .sum();
                if f <= 0.0 {
                    continue;
                }
                let key = h.address.to_string();
                let hz = hazard.entry(key.clone()).or_default();
                *hz += dial * hp.origin_rate_per_myr * f * area / hp.earth_area_m2 * dt;
                let u = Draws::new(seed, h.address.clone(), origin_draws.clone()).unit(attempt);
                if *hz >= -m::ln(u.max(1e-300)) && born.is_none() {
                    let names: Vec<String> = hp
                        .settings
                        .iter()
                        .filter(|st| st.enabled && st.habitats.contains(&h.kind))
                        .map(|st| format!("setting:{}", st.id))
                        .collect();
                    born = Some((key, names.join(",")));
                }
            }
            if let Some((hab, settings)) = born {
                let mut causes = vec![format!("epoch:{}", ep.id)];
                causes.extend(settings.split(',').map(String::from));
                let id = push(
                    &mut events,
                    LifeEvent {
                        id: String::new(),
                        at_myr: t1,
                        kind: "origin".into(),
                        subject: hab.clone(),
                        causes,
                        why: format!(
                            "life began in {hab} (origin dial {dial}); it spreads to every open habitat on the world"
                        ),
                        dials: vec!["origin".into()],
                    },
                );
                origin_myr.get_or_insert(t1);
                let mut caps = BTreeMap::new();
                for c in hp.capabilities.iter().filter(|c| c.at_origin) {
                    caps.insert(c.id.clone(), (t1, id.clone()));
                    first.entry(c.id.clone()).or_insert(t1);
                }
                life = Some(Life {
                    caps,
                    opportunity: 0.0,
                    diversity: 0.0,
                    origin_event: id,
                });
            }
        }
        // Box B for the capabilities held, then box D.
        let mut production = 0.0;
        if let Some(l) = life.as_mut() {
            let mets: Vec<String> = hp
                .capabilities
                .iter()
                .filter(|c| l.caps.contains_key(&c.id))
                .flat_map(|c| c.unlocks_metabolisms.clone())
                .collect();
            let mut media: Vec<String> = hp.origin_media.clone();
            for c in hp
                .capabilities
                .iter()
                .filter(|c| l.caps.contains_key(&c.id))
            {
                media.extend(c.unlocks_media.clone());
            }
            let key = format!("{ei}|{}|{}", mets.join(","), media.join(","));
            let wb = cache.entry(key).or_insert_with(|| {
                let mut bp2 = bp.clone();
                for mb in &mut bp2.metabolisms {
                    let mm: Vec<String> = mb
                        .media
                        .iter()
                        .filter(|x| media.contains(x))
                        .cloned()
                        .collect();
                    mb.media = mm;
                }
                budget::world_budget(g, &bp2, &mets)
            });
            production = wb.production_gc_yr;
            last.alive = true;
            last.production_gc_yr = production;
            last.on_land = l.caps.contains_key("land");
            for h in &wb.habitats {
                for (k, v) in &h.by_metabolism {
                    *last.by_metabolism.entry(k.clone()).or_default() += v;
                }
            }
            let mut facts = ep.facts.clone();
            for (k, v) in &world {
                facts.insert(k.clone(), *v);
            }
            let ox = facts.get(&hp.oxidant_fact).copied().unwrap_or(0.0);
            facts.insert(
                hp.oxidant_fact.clone(),
                if wb.high_power { ox } else { 0.0 },
            );
            facts.insert("high_power".into(), if wb.high_power { 1.0 } else { 0.0 });
            facts.insert("production_gc_yr".into(), production);
            let has = |ch: &str| {
                open.iter().any(|&i| {
                    g.habitats[i]
                        .profile
                        .energy
                        .available
                        .iter()
                        .any(|a| a == ch)
                })
            };
            facts.insert("light".into(), if has("light") { 1.0 } else { 0.0 });
            let kinds: Vec<&str> = open.iter().map(|&i| g.habitats[i].kind.as_str()).collect();
            facts.insert(
                "land".into(),
                if kinds.contains(&"land-surface") {
                    1.0
                } else {
                    0.0
                },
            );
            facts.insert(
                "surface_water".into(),
                if kinds.iter().any(|k| *k == "ocean-sunlit" || *k == "lake") {
                    1.0
                } else {
                    0.0
                },
            );
            let mut distinct: Vec<&str> = kinds.clone();
            distinct.sort();
            distinct.dedup();
            let temp = mean_temperature(g, &open).unwrap_or(hp.reference_k);
            let gen = m::exp(-hp.activation_ev / k_ev * (1.0 / temp - 1.0 / hp.reference_k));
            let pop = if production > 0.0 {
                m::pow(production / hp.production_ref_gc_yr, hp.population_exponent)
            } else {
                0.0
            };
            let variety = distinct.len() as f64 / hp.variety_ref;
            let d_opp = pop * gen * variety * dt;
            l.opportunity += d_opp;
            // Diversity relaxes towards a capacity set by production.
            let cap_div = m::log10(1.0 + production / hp.diversity_floor_gc_yr);
            l.diversity += (cap_div - l.diversity) * (1.0 - m::exp(-dt / hp.recovery_myr));
            if let Some(cid) = pending_recovery.clone() {
                if l.diversity >= 0.9 * cap_div && cap_div > 0.0 {
                    push(
                        &mut events,
                        LifeEvent {
                            id: String::new(),
                            at_myr: t1,
                            kind: "recovery".into(),
                            subject: w.body.to_string(),
                            causes: vec![cid],
                            why: format!(
                                "diversity back to 90% of what production supports, about {:.0} Myr after the catastrophe",
                                hp.recovery_myr * m::ln(10.0)
                            ),
                            dials: vec![],
                        },
                    );
                    pending_recovery = None;
                }
            }
            for (ci, c) in hp.capabilities.iter().enumerate() {
                if l.caps.contains_key(&c.id) || c.at_origin {
                    continue;
                }
                let ready = c.needs.iter().all(|n| l.caps.contains_key(n))
                    && c.needs_facts
                        .iter()
                        .all(|n| facts.get(&n.fact).copied().unwrap_or(0.0) >= n.min);
                if !ready {
                    continue;
                }
                let dial = c.dial.as_ref().map_or(1.0, |d| hp.dial(d));
                let p = cap_progress.entry(c.id.clone()).or_default();
                *p += c.rate * dial * d_opp;
                let a = cap_attempt.get(&c.id).copied().unwrap_or(0);
                let u = cap_draws.unit(ci as u64 * 1024 + a);
                if *p >= -m::ln(u.max(1e-300)) {
                    let mut causes: Vec<String> = c
                        .needs
                        .iter()
                        .filter_map(|n| l.caps.get(n).map(|x| x.1.clone()))
                        .collect();
                    if causes.is_empty() {
                        causes.push(l.origin_event.clone());
                    }
                    causes.push(format!("epoch:{}", ep.id));
                    let id = push(
                        &mut events,
                        LifeEvent {
                            id: String::new(),
                            at_myr: t1,
                            kind: "capability".into(),
                            subject: c.id.clone(),
                            causes,
                            why: format!(
                                "{}: {} (after {:.1} Earth-Myr of opportunity since origin)",
                                c.label, c.effect, l.opportunity
                            ),
                            dials: c.dial.iter().cloned().collect(),
                        },
                    );
                    l.caps.insert(c.id.clone(), (t1, id));
                    first.entry(c.id.clone()).or_insert(t1);
                }
            }
        }
        if s % every == every - 1 {
            points.push(HistoryPoint {
                at_myr: t1,
                alive: life.is_some(),
                capabilities: life
                    .as_ref()
                    .map_or(vec![], |l| l.caps.keys().cloned().collect()),
                production_gc_yr: production,
                diversity: life.as_ref().map_or(0.0, |l| l.diversity),
                opportunity: life.as_ref().map_or(0.0, |l| l.opportunity),
                world: world.clone(),
            });
        }
    }
    Ok(LifeHistory {
        body: w.body.clone(),
        events,
        points,
        first,
        origin_myr,
        dials: hp.dials.clone(),
        evidence: LifeEvidence::PlausibleReconstruction,
        unknown,
    })
}

/// Write a history's events to the chronicle as derived `life.<kind>`
/// events, causes linked, dated from the world's birth year.
pub fn record(
    ch: &mut sse_chronicle::Chronicle,
    h: &LifeHistory,
    birth_year: i64,
) -> Result<Vec<ContentHash>, String> {
    use sse_chronicle::{Event, EventBody, Evidence, Payload, Publisher};
    use sse_core::cbor::Value;
    let mut ids: BTreeMap<String, ContentHash> = BTreeMap::new();
    let mut out = vec![];
    for e in &h.events {
        let at = UniverseTime::years(birth_year + m::round(e.at_myr * 1.0e6) as i64);
        let mut data = BTreeMap::new();
        data.insert("why".into(), Value::Text(e.why.clone()));
        data.insert("subject".into(), Value::Text(e.subject.clone()));
        data.insert(
            "causes".into(),
            Value::Array(e.causes.iter().map(|c| Value::Text(c.clone())).collect()),
        );
        data.insert(
            "dials".into(),
            Value::Array(e.dials.iter().map(|c| Value::Text(c.clone())).collect()),
        );
        let ev = Event::new(EventBody {
            kind: format!("life.{}", e.kind),
            subject: vec![h.body.clone()],
            occurred_at: at,
            activated_at: at,
            recognised_at: ch.next_stamp(),
            causes: e
                .causes
                .iter()
                .filter_map(|c| ids.get(c).copied())
                .collect(),
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Derived,
            publisher: Publisher::new("sse.base-life", "history", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        let id = ch.append(ev).map_err(|e| format!("{e:?}"))?;
        ids.insert(e.id.clone(), id);
        out.push(id);
    }
    Ok(out)
}
