//! Box J (life design section 10): sifters name epochs afterwards and raise
//! story hooks; `view` assembles what the UI's biosphere view and life
//! timeline draw. Every item answers "why?" in three lenses (physics,
//! ecology, history), down to the physical cause.

use crate::biosphere::Biosphere;
use crate::history::LifeHistory;
use crate::signatures::Signatures;
use crate::RelationKind;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use std::collections::BTreeMap;

/// Settings (`provides.other.life_sift`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SiftParams {
    /// Plain epoch names by the newest capability (or `barren`, `dead`).
    pub epoch_names: BTreeMap<String, String>,
    /// Significance weights: persistence, reach, capacity change, reversibility.
    pub weights: [f64; 4],
    /// An epoch is major at or above this significance.
    pub major: f64,
    /// A catastrophe opens an "aftermath" epoch when it cuts diversity by this share.
    pub aftermath_share: f64,
    /// Story hook: a biosphere is doomed when its world stops being habitable within this, Myr.
    pub doomed_within_myr: f64,
}

impl Default for SiftParams {
    fn default() -> Self {
        let names = [
            ("barren", "The barren age"),
            ("chemotrophy", "The age of rock-eaters"),
            ("dormancy", "The age of rock-eaters"),
            ("anoxygenic", "The age of purple mats"),
            ("oxygenic", "The age of green mats"),
            ("complex_cells", "The age of complex cells"),
            ("sex", "The age of complex cells"),
            ("multicellularity", "The age of soft bodies"),
            ("large_bodies", "The age of large animals"),
            ("hard_parts", "The age of shells and bones"),
            ("land", "The age of the land"),
            ("dead", "The dead age"),
        ];
        SiftParams {
            epoch_names: names
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            weights: [0.4, 0.2, 0.3, 0.1],
            major: 0.25,
            aftermath_share: 0.3,
            doomed_within_myr: 500.0,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Why {
    pub physics: String,
    pub ecology: String,
    pub history: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NamedEpoch {
    pub name: String,
    pub from_myr: f64,
    pub to_myr: f64,
    /// The event that opened it.
    pub opened_by: String,
    /// Persistence, reach, capacity change, reversibility (each 0 to 1).
    pub scores: [f64; 4],
    pub significance: f64,
    pub major: bool,
    pub why: Why,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StoryHook {
    /// `one_short_of_animals`, `doomed`, `last_refuge`, `false_positive`,
    /// `hidden_life`, `dead_world_record`.
    pub id: String,
    pub text: String,
    pub why: Why,
}

/// Name the epochs of a history: boundaries at origin, each capability
/// gained or lost, sterilisation, and catastrophes that cut diversity by at
/// least `aftermath_share`.
pub fn epochs(h: &LifeHistory, now_myr: f64, p: &SiftParams) -> Vec<NamedEpoch> {
    let diversity_at = |t: f64| -> f64 {
        h.points
            .iter()
            .rfind(|x| x.at_myr <= t)
            .map(|x| x.diversity)
            .unwrap_or(0.0)
    };
    let production_at = |t: f64| -> f64 {
        h.points
            .iter()
            .rfind(|x| x.at_myr <= t)
            .map(|x| x.production_gc_yr)
            .unwrap_or(0.0)
    };
    let mut cuts: Vec<(f64, String, String)> = vec![(0.0, "formation".into(), "barren".into())];
    let mut newest = "barren".to_string();
    let mut held: Vec<String> = vec![];
    for e in &h.events {
        match e.kind.as_str() {
            "origin" => {
                newest = "chemotrophy".into();
                held = vec!["chemotrophy".into()];
                cuts.push((e.at_myr, e.id.clone(), newest.clone()));
            }
            "capability" => {
                held.push(e.subject.clone());
                if p.epoch_names.get(&e.subject) != p.epoch_names.get(&newest) {
                    newest = e.subject.clone();
                    cuts.push((e.at_myr, e.id.clone(), newest.clone()));
                }
            }
            "capability_lost" => {
                held.retain(|c| c != &e.subject);
                if e.subject == newest {
                    newest = held.last().cloned().unwrap_or_else(|| "barren".into());
                    cuts.push((e.at_myr, e.id.clone(), newest.clone()));
                }
            }
            "sterilised" => {
                newest = "dead".into();
                held.clear();
                cuts.push((e.at_myr, e.id.clone(), newest.clone()));
            }
            "catastrophe" => {
                let before = diversity_at(e.at_myr - 1.0e-6);
                let after = h
                    .points
                    .iter()
                    .find(|x| x.at_myr > e.at_myr)
                    .map(|x| x.diversity)
                    .unwrap_or(before);
                if before > 0.0 && (before - after) / before >= p.aftermath_share {
                    cuts.push((e.at_myr, e.id.clone(), format!("aftermath:{}", e.subject)));
                }
            }
            _ => {}
        }
    }
    cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total = now_myr.max(1.0);
    let max_div = h
        .points
        .iter()
        .map(|x| x.diversity)
        .fold(0.0, f64::max)
        .max(1e-9);
    let mut out = vec![];
    for (k, (from, by, state)) in cuts.iter().enumerate() {
        let to = cuts.get(k + 1).map(|c| c.0).unwrap_or(now_myr).min(now_myr);
        if to <= *from {
            continue;
        }
        let (name, label_state) = match state.strip_prefix("aftermath:") {
            Some(cat) => (format!("The aftermath of {cat}"), "aftermath"),
            None => (
                p.epoch_names
                    .get(state)
                    .cloned()
                    .unwrap_or_else(|| format!("The age of {state}")),
                state.as_str(),
            ),
        };
        let persistence = ((to - from) / total).min(1.0);
        let reach = (diversity_at(to - 1.0e-6) / max_div).clamp(0.0, 1.0);
        let (p0, p1) = (
            production_at(*from).max(1.0),
            production_at(to - 1.0e-6).max(1.0),
        );
        let capacity = (m::log10(p1 / p0).abs() / 6.0).min(1.0);
        let reversed = h
            .events
            .iter()
            .any(|e| e.kind == "capability_lost" && e.subject == *state && e.at_myr >= to);
        let reversibility = if reversed { 1.0 } else { 0.0 };
        let scores = [persistence, reach, capacity, reversibility];
        let significance: f64 = scores.iter().zip(&p.weights).map(|(s, w)| s * w).sum();
        let opened = h.events.iter().find(|e| &e.id == by);
        out.push(NamedEpoch {
            name,
            from_myr: *from,
            to_myr: to,
            opened_by: by.clone(),
            scores,
            significance,
            major: significance >= p.major,
            why: Why {
                physics: opened
                    .map(|e| e.why.clone())
                    .unwrap_or_else(|| "the world formed".into()),
                ecology: format!(
                    "production {:.1e} to {:.1e} g C/yr; diversity {:.2} of its peak",
                    production_at(*from),
                    production_at(to - 1.0e-6),
                    reach
                ),
                history: format!(
                    "{label_state} from {from:.0} to {to:.0} Myr ({:.0}% of the world's life so far)",
                    persistence * 100.0
                ),
            },
        });
    }
    out
}

/// What the hooks need beyond the biosphere and history.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct HookInputs {
    /// When the world stops being habitable (look-ahead `oceans_lost`), Myr.
    #[serde(default)]
    pub habitable_until_myr: Option<f64>,
    /// The world's O2 fraction, for "one innovation short".
    #[serde(default)]
    pub o2_fraction: f64,
    pub now_myr: f64,
}

pub fn hooks(
    bio: Option<&Biosphere>,
    h: Option<&LifeHistory>,
    sig: &Signatures,
    inp: &HookInputs,
    p: &SiftParams,
) -> Vec<StoryHook> {
    let mut out = vec![];
    let held = |c: &str| bio.is_some_and(|b| b.capabilities.iter().any(|x| x == c));
    if held("multicellularity") && !held("large_bodies") {
        out.push(StoryHook {
            id: "one_short_of_animals".into(),
            text:
                "A world one innovation short of animals: soft many-celled life, nothing large yet"
                    .into(),
            why: Why {
                physics: format!(
                    "oxygen at {:.1}% of the air; large bodies need about 10% to power big cells",
                    inp.o2_fraction * 100.0
                ),
                ecology: "food webs stop at small grazers".into(),
                history: "multicellularity arrived, large bodies have not".into(),
            },
        });
    }
    if let (Some(b), Some(until)) = (bio, inp.habitable_until_myr) {
        let left = until - inp.now_myr;
        if left > 0.0 && left <= p.doomed_within_myr {
            out.push(StoryHook {
                id: "doomed".into(),
                text: format!("A biosphere with about {left:.0} Myr left: its star is brightening"),
                why: Why {
                    physics: "the star brightens about 10% per Gyr; the oceans boil away when the runaway greenhouse starts".into(),
                    ecology: format!("{} guilds all depend on surface water", b.guilds.len()),
                    history: format!("oceans lost at {until:.0} Myr in the look-ahead"),
                },
            });
        }
    }
    if let Some(b) = bio {
        let living: Vec<&str> = b
            .budget
            .habitats
            .iter()
            .filter(|x| x.production_gc_yr.unwrap_or(0.0) > 0.0)
            .map(|x| x.kind.as_str())
            .collect();
        let deep = [
            "sea-floor",
            "deep-crust",
            "cave",
            "under-ice-ocean",
            "ocean-interior",
        ];
        if living.len() == 1 && deep.contains(&living[0]) {
            out.push(StoryHook {
                id: "last_refuge".into(),
                text: format!(
                    "A relic ecosystem in its last refuge: life holds on only in the {}",
                    living[0]
                ),
                why: Why {
                    physics: "every other habitat is closed (cold, dry or unlit)".into(),
                    ecology: format!("{} guilds on chemical energy alone", b.guilds.len()),
                    history: "the surface habitats have gone; the deep one remains".into(),
                },
            });
        }
    }
    if sig.candidate && !sig.alive {
        let s = sig.signals.iter().find(|s| s.id != "nothing");
        out.push(StoryHook {
            id: "false_positive".into(),
            text: format!(
                "A false-positive paradise: scans say life at {:.0}% confidence, and there is none",
                sig.confidence * 100.0
            ),
            why: Why {
                physics: s.map(|s| s.why.clone()).unwrap_or_default(),
                ecology: "no biosphere".into(),
                history: if sig.ever_alive {
                    "life was here once and is gone".into()
                } else {
                    "life never began here".into()
                },
            },
        });
    }
    if sig.alive && !sig.candidate {
        if let Some(blind) = &sig.blind {
            out.push(StoryHook {
                id: "hidden_life".into(),
                text: "Hidden life: the scans read empty and the world is alive".into(),
                why: Why {
                    physics: blind.clone(),
                    ecology: sig.summary.clone().unwrap_or_default(),
                    history: h
                        .and_then(|h| h.origin_myr)
                        .map(|o| format!("alive since {o:.0} Myr"))
                        .unwrap_or_default(),
                },
            });
        }
    }
    if !sig.alive && sig.ever_alive && sig.traces.iter().any(|t| t.class != "erased") {
        let kept: Vec<String> = sig
            .traces
            .iter()
            .filter(|t| t.class != "erased")
            .map(|t| format!("{} ({})", t.kind, t.class))
            .collect();
        out.push(StoryHook {
            id: "dead_world_record".into(),
            text: format!(
                "A dead world that still tells its story: {}",
                kept.join(", ")
            ),
            why: Why {
                physics: "the rocks that hold the record have not been recycled or eroded away"
                    .into(),
                ecology: "no living biosphere".into(),
                history: "life began and ended here".into(),
            },
        });
    }
    out
}

/// Everything the biosphere view and life timeline draw.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeView {
    pub body: String,
    pub evidence: String,
    pub timeline: Vec<crate::history::HistoryPoint>,
    pub events: Vec<crate::history::LifeEvent>,
    pub epochs: Vec<NamedEpoch>,
    pub hooks: Vec<StoryHook>,
    pub signatures: Signatures,
    /// Relation counts per lens (resource, habitat, capability, ancestry).
    pub lenses: BTreeMap<String, u32>,
    pub guilds: Vec<crate::biosphere::GuildDetail>,
    pub traits: Option<crate::biosphere::WorldTraits>,
    pub dials: BTreeMap<String, f64>,
    /// The energy ledger (LE2): what arrives, what life could tap, what it
    /// captures, what staying alive costs. Worlds of the chart only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy: Option<crate::ledger::EnergyLedger>,
}

pub fn view(
    body: &str,
    bio: Option<&Biosphere>,
    h: Option<&LifeHistory>,
    sig: &Signatures,
    inp: &HookInputs,
    p: &SiftParams,
) -> LifeView {
    let mut lenses = BTreeMap::new();
    if let Some(b) = bio {
        for r in &b.resolved.relations {
            let k = match r.kind {
                RelationKind::Resource => "resource",
                RelationKind::Habitat => "habitat",
                RelationKind::Capability => "capability",
                RelationKind::Ancestry => "ancestry",
            };
            *lenses.entry(k.to_string()).or_insert(0) += 1;
        }
    }
    LifeView {
        body: body.into(),
        evidence: "plausible reconstruction".into(),
        timeline: h.map(|h| h.points.clone()).unwrap_or_default(),
        events: h.map(|h| h.events.clone()).unwrap_or_default(),
        epochs: h.map(|h| epochs(h, inp.now_myr, p)).unwrap_or_default(),
        hooks: hooks(bio, h, sig, inp, p),
        signatures: sig.clone(),
        lenses,
        guilds: bio.map(|b| b.guilds.clone()).unwrap_or_default(),
        traits: bio.map(|b| b.traits.clone()),
        dials: h.map(|h| h.dials.clone()).unwrap_or_default(),
        energy: None,
    }
}
