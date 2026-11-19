//! One formation pipeline: a `FormationHistory` (stages 1 to 3) carried
//! through growth (stage 4 and 5, modern or Classic Accrete), the violent
//! phase (6), the leftovers (7) and each world's early life (8), then the
//! base-physics heat flow run on every solid world. Callers no longer chain
//! the stages by hand.
//!
//! The heat budget is the visible `sse.base.planet-heat` flow from the
//! base-physics pack, evaluated block by block, so each number carries the
//! same explanation (and paper links) the flow editor shows.

use crate::accrete::{self, Rule};
use crate::early::{early_life, Envelope, WorldStart, M_EARTH_KG, R_EARTH_M};
use crate::growth::{self, Growth};
use crate::history::{FormationHistory, HistoryEvent};
use crate::hurley::Fits;
use crate::leftovers::{self, Leftovers};
use crate::violent::{self, Upheaval};
use serde::{Deserialize, Serialize};
use sse_core::{Address, Quantity, Seed};
use sse_flow::{Context, Flow, InputValue, SourceLabel};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The pipeline's own version: bump when its chaining changes.
pub const PIPELINE_VERSION: &str = "formation-pipeline-v1";
/// The heat flow run on every solid world.
pub const HEAT_FLOW: &str = "sse.base.planet-heat";

const BASE_PHYSICS: &str = include_str!("../../../packs/base-physics/pack.json");
const GYR_S: f64 = 1e9 * 365.25 * 86_400.0;

/// Which block grows the planets (design section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Block {
    /// F3 to F5: pebbles and planetesimal rings, migration, the violent phase, leftovers.
    Modern,
    /// F2: SSE 3's Accrete (System Lab), straight to early life (no violent phase).
    Classic,
    /// F2's corrected rule (`accrete-dole1970-v1`), driven by the inheritance.
    ClassicCorrected,
}

/// One solid world's radiogenic heat, from the planet-heat flow.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldHeat {
    pub address: Address,
    pub flow: String,
    /// The world's age at the system's present, Gyr.
    pub age_gyr: f64,
    pub radiogenic_power_w: f64,
    pub heat_flow_w_m2: f64,
    /// Age at which volcanism and plate motion stop, Gyr.
    pub active_geology_until_gyr: f64,
    pub active_now: bool,
}

/// Everything the stages made for one system.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SystemRun {
    pub pipeline: String,
    pub block: Block,
    pub history: FormationHistory,
    #[serde(default)]
    pub growth: Option<Growth>,
    #[serde(default)]
    pub upheaval: Option<Upheaval>,
    #[serde(default)]
    pub leftovers: Option<Leftovers>,
    #[serde(default)]
    pub accrete: Option<accrete::Run>,
    /// What growth handed to early life, one per world (orbit, formation date, provenance).
    pub protoplanets: Vec<crate::early::Protoplanet>,
    pub worlds: Vec<WorldStart>,
    pub heat: Vec<WorldHeat>,
    /// Dated events this pipeline adds to the history's own (geology falling quiet).
    pub events: Vec<HistoryEvent>,
    /// Anything a block refused, in words.
    #[serde(default)]
    pub notes: Vec<String>,
}

struct HeatFlow {
    flow: Flow,
    atoms: sse_flow::AtomRegistry,
    kinds: BTreeMap<String, sse_core::QuantityKind>,
    units: sse_core::Units,
}

fn heat_flow() -> &'static HeatFlow {
    static CELL: OnceLock<HeatFlow> = OnceLock::new();
    CELL.get_or_init(|| {
        let pack =
            sse_pack::PackManifest::from_json(BASE_PHYSICS).expect("base-physics pack parses");
        let stack = sse_pack::resolve(&[pack], "0.1.0").expect("base-physics pack resolves");
        let atoms = sse_atoms::registry();
        let doc = stack
            .flows
            .values()
            .find(|f| f.doc.id == HEAT_FLOW)
            .expect("planet-heat flow in base-physics")
            .doc
            .clone();
        let flow = {
            let cx = Context {
                atoms: &atoms,
                kinds: &stack.kinds,
                units: &stack.units,
            };
            Flow::compile(doc, &cx).expect("planet-heat compiles")
        };
        HeatFlow {
            flow,
            atoms,
            kinds: stack.kinds,
            units: stack.units,
        }
    })
}

/// Run the planet-heat flow for one solid world.
pub fn world_heat(
    w: &WorldStart,
    age_gyr: f64,
    k40_rel: f64,
    r_process_rel: f64,
) -> Option<WorldHeat> {
    let h = heat_flow();
    let cx = Context {
        atoms: &h.atoms,
        kinds: &h.kinds,
        units: &h.units,
    };
    let mut ins = BTreeMap::new();
    let mut put = |name: &str, kind: &str, v: f64| {
        let dim = h
            .kinds
            .get(kind)
            .map(|k| k.dim)
            .unwrap_or(sse_core::Dimension::NONE);
        ins.insert(
            name.to_string(),
            InputValue {
                q: Quantity::si(v, dim),
                source: SourceLabel::Authored,
            },
        );
    };
    put("age", "body.age", age_gyr.max(0.0) * GYR_S);
    put("mass", "body.mass", w.mass_earth * M_EARTH_KG);
    put(
        "core_mass_fraction",
        "interior.core_mass_fraction",
        w.core_mass_fraction,
    );
    put("radius", "body.radius", w.solid_radius * R_EARTH_M);
    put("k40_rel", "inheritance.k40_rel", k40_rel);
    put("r_process_rel", "inheritance.r_process_rel", r_process_rel);
    let ev = h.flow.eval(&cx, &ins).ok()?;
    let until = ev.published("interior.active_geology_lifetime")?.value / GYR_S;
    Some(WorldHeat {
        address: w.address.clone(),
        flow: HEAT_FLOW.into(),
        age_gyr,
        radiogenic_power_w: ev.published("interior.radiogenic_power")?.value,
        heat_flow_w_m2: ev.published("interior.radiogenic_heat_flow")?.value,
        active_geology_until_gyr: until,
        active_now: age_gyr < until,
    })
}

/// Carry a formed history through stages 4 to 8 and the heat flow.
pub fn run(history: FormationHistory, block: Block) -> SystemRun {
    let seed = Seed::from_text(&history.manifest.seed);
    let system = history.manifest.system.clone();
    let inh = history.inheritance.clone();
    let fits = Fits::new(inh.z);
    let host = &history.stars[0].life;
    let (mut growth, mut upheaval, mut left, mut acc) = (None, None, None, None);
    let mut notes = vec![];
    let protos = match block {
        Block::Modern => {
            let g = growth::grow(&seed, &system, &history.disc);
            let u = violent::upheave(&seed, &system, &history.disc, &g);
            let l = leftovers::leftovers(&seed, &system, &history.disc, &u);
            let p = violent::protoplanets_of(&l.worlds, &system);
            growth = Some(g);
            upheaval = Some(u);
            left = Some(l);
            p
        }
        Block::Classic | Block::ClassicCorrected => {
            let mut req = accrete::Request::from_history(&history, &history.manifest.seed);
            if block == Block::Classic {
                // SSE 3's own settings: System Lab's default dust and gas
                // controls whatever the star, as V3 ran it.
                req.rule = Rule::SystemLab;
                req.controls = accrete::Controls::DEFAULT;
            }
            match accrete::run(&req) {
                Ok(r) => {
                    let p = accrete::protoplanets(&r, &system, Some(&history.disc));
                    acc = Some(r);
                    p
                }
                Err(e) => {
                    notes.push(format!("{block:?} block refused this system: {e}"));
                    vec![]
                }
            }
        }
    };
    let worlds: Vec<WorldStart> = protos.iter().map(|p| early_life(p, &fits, host)).collect();
    let now_myr = inh.age_gyr * 1000.0;
    let mut heat = vec![];
    let mut events = vec![];
    for (w, p) in worlds.iter().zip(&protos) {
        if w.envelope == Envelope::Giant {
            continue;
        }
        let age = (now_myr - p.formed_myr).max(0.0) / 1000.0;
        if let Some(hh) = world_heat(w, age, inh.k40_rel, inh.r_process_rel) {
            let quiet_myr = p.formed_myr + hh.active_geology_until_gyr * 1000.0;
            if hh.active_geology_until_gyr > 0.0 && hh.active_geology_until_gyr < 100.0 {
                events.push(HistoryEvent {
                    t_myr: quiet_myr,
                    kind: "geology-quiet".into(),
                    subject: w.address.clone(),
                    text: format!(
                        "Radioactive heat can no longer drive volcanism or plate motion ({:.1} Gyr after formation).",
                        hh.active_geology_until_gyr
                    ),
                });
            }
            heat.push(hh);
        }
    }
    events.sort_by(|a, b| a.t_myr.total_cmp(&b.t_myr));
    SystemRun {
        pipeline: PIPELINE_VERSION.into(),
        block,
        history,
        growth,
        upheaval,
        leftovers: left,
        accrete: acc,
        protoplanets: protos,
        worlds,
        heat,
        events,
        notes,
    }
}
