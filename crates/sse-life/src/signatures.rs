//! Box I (life design section 9): signatures and what observers can tell.
//!
//! True state, surviving evidence and what an observer can detect are kept
//! apart (Evolution Lab's signature contract). The true state is the
//! biosphere and history; the evidence is what the world's physics makes of
//! it (gases, a reflectance edge, colour, surviving traces); detection is
//! per disclosure tier. A scanned candidate carries a confidence, a
//! posterior from stated likelihood ratios, so physics can mislead players
//! for good reasons: oxygen from water loss round a red dwarf (a false
//! positive), life under ice that shows nothing (a false negative).
//!
//! `facts` writes the result under field names a data view filters by tier:
//! `life.obvious.*`, `life.signature.*` (scanned), `life.survey.*` and
//! `life.past.*` (surveyed), and the truth under `gm.life.*`.

use crate::biosphere::Biosphere;
use crate::history::LifeHistory;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use std::collections::BTreeMap;

/// The world as an observer could measure it (the planet layer supplies
/// these; fixtures stand in until P4 world stacks carry atmospheres).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Observables {
    pub star_teff_k: f64,
    /// Surface pressure, bar; `None` for no atmosphere.
    pub atmosphere_bar: Option<f64>,
    /// O2 mixing ratio from non-biological sources (photolysis and hydrogen
    /// escape after water loss).
    #[serde(default)]
    pub abiotic_o2: f64,
    /// CH4 from rock (serpentinisation, volcanism), ppm.
    #[serde(default)]
    pub abiotic_ch4_ppm: f64,
    /// Ice over the habitats, km (hides them from any remote scan).
    #[serde(default)]
    pub ice_shell_km: Option<f64>,
    /// Cloud cover, 0 to 1.
    #[serde(default)]
    pub cloud_cover: f64,
    pub land_fraction: f64,
    pub obliquity_deg: f64,
    /// Time for the surface to be recycled (plates); `None` for a stagnant lid.
    #[serde(default)]
    pub recycling_gyr: Option<f64>,
    /// Time for erosion to remove surface traces, Gyr (`None`: no weather).
    #[serde(default)]
    pub erosion_gyr: Option<f64>,
    /// The world's age now, Myr.
    pub age_myr: f64,
    /// Hydrogen dominates the air: methane there is what the gas makes by
    /// itself (a giant's air), not a sign of anything.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reducing_air: bool,
}

/// Settings (`provides.other.life_signatures`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SignatureParams {
    /// Prior probability of life the confidence starts from.
    pub prior: f64,
    /// Detection floors: O2 mixing ratio, CH4 ppm, red-edge area fraction.
    pub o2_floor: f64,
    pub ch4_floor_ppm: f64,
    pub edge_floor: f64,
    /// Biological O2 mixing ratio per unit of oxygenic production at Earth's
    /// rate, when oxygen accumulates (sketch: 0.21 for Earth today).
    pub o2_per_earth_production: f64,
    pub earth_oxygenic_gc_yr: f64,
    /// Biological CH4, ppm per g C/yr of methane-making carbon, in oxic air;
    /// anoxic air keeps methane `anoxic_ch4_factor` times longer. The carbon
    /// is the methanogens' own production plus the share of all production
    /// that decays through methanogenesis (oxic and anoxic worlds).
    pub ch4_ppm_per_gc_yr: f64,
    pub anoxic_ch4_factor: f64,
    pub decay_methane_oxic: f64,
    pub decay_methane_anoxic: f64,
    /// Likelihood ratios P(signal | life) / P(signal | no life).
    pub lr_o2_ch4: f64,
    pub lr_o2_alone: f64,
    /// O2 alone round a star cooler than `m_dwarf_teff_k`: water loss makes it
    /// without life.
    pub lr_o2_m_dwarf: f64,
    pub m_dwarf_teff_k: f64,
    pub lr_ch4_alone: f64,
    pub lr_edge: f64,
    /// Nothing seen where a scan could have seen it.
    pub lr_nothing: f64,
    /// Survival classes of past traces (fraction of the record left).
    pub abundant: f64,
    pub scattered: f64,
    pub rare: f64,
    pub sources: Vec<String>,
}

impl Default for SignatureParams {
    fn default() -> Self {
        SignatureParams {
            prior: 0.5,
            o2_floor: 1.0e-3,
            ch4_floor_ppm: 0.1,
            edge_floor: 0.05,
            o2_per_earth_production: 0.21,
            earth_oxygenic_gc_yr: 1.0e17,
            ch4_ppm_per_gc_yr: 4.5e-15,
            anoxic_ch4_factor: 100.0,
            decay_methane_oxic: 0.005,
            decay_methane_anoxic: 0.5,
            lr_o2_ch4: 20.0,
            lr_o2_alone: 3.0,
            lr_o2_m_dwarf: 1.5,
            m_dwarf_teff_k: 3900.0,
            lr_ch4_alone: 2.0,
            lr_edge: 10.0,
            lr_nothing: 0.5,
            abundant: 0.3,
            scattered: 0.03,
            rare: 1.0e-4,
            sources: vec![],
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Signal {
    /// `o2_ch4`, `o2`, `ch4`, `red_edge`, `nothing`.
    pub id: String,
    pub value: f64,
    /// Likelihood ratio it carries.
    pub lr: f64,
    pub why: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Trace {
    /// `isotopes`, `banded_iron`, `fossils`, `stromatolites`.
    pub kind: String,
    /// The event that left it, and when (Myr after the world formed).
    pub from_event: String,
    pub at_myr: f64,
    /// Fraction of the record surviving now, and its class.
    pub surviving: f64,
    pub class: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Signatures {
    /// True state (GM only).
    pub alive: bool,
    pub ever_alive: bool,
    /// Obvious tier.
    pub colour: Option<String>,
    pub seasonal: bool,
    /// Scanned tier.
    pub o2: f64,
    pub ch4_ppm: f64,
    pub edge_nm: Option<f64>,
    pub signals: Vec<Signal>,
    pub candidate: bool,
    pub confidence: f64,
    /// What a remote scan cannot see here, in words.
    pub blind: Option<String>,
    /// Surveyed tier.
    pub summary: Option<String>,
    pub guild_cards: Vec<String>,
    pub hazard: Option<String>,
    pub biomass_gc: f64,
    pub traces: Vec<Trace>,
    /// `plausible reconstruction`.
    pub evidence: String,
}

/// Box I for one world. `bio` is `None` for a lifeless world (or one whose
/// life died out); `history` gives the past.
pub fn signatures(
    bio: Option<&Biosphere>,
    history: Option<&LifeHistory>,
    obs: &Observables,
    p: &SignatureParams,
) -> Signatures {
    let alive = bio.is_some();
    let ever_alive = alive || history.is_some_and(|h| h.origin_myr.is_some());
    let by_met = |id: &str| -> f64 {
        bio.map(|b| {
            b.budget
                .habitats
                .iter()
                .map(|h| h.by_metabolism.get(id).copied().unwrap_or(0.0))
                .sum()
        })
        .unwrap_or(0.0)
    };
    let air = obs.atmosphere_bar.is_some_and(|b| b > 1.0e-3);
    let sealed = obs.ice_shell_km.is_some_and(|k| k > 0.01);
    // Gases: life's share only reaches the air from open-air habitats.
    let o2_bio = if air && !sealed {
        p.o2_per_earth_production * (by_met("oxygenic") / p.earth_oxygenic_gc_yr).min(1.0)
    } else {
        0.0
    };
    let o2 = (o2_bio + obs.abiotic_o2).min(1.0);
    let ch4_bio = if air && !sealed {
        let anoxic = o2 < 1.0e-3;
        let (k, decay) = if anoxic {
            (p.anoxic_ch4_factor, p.decay_methane_anoxic)
        } else {
            (1.0, p.decay_methane_oxic)
        };
        let total = bio.map(|b| b.budget.production_gc_yr).unwrap_or(0.0);
        p.ch4_ppm_per_gc_yr * (by_met("methanogen") + decay * total) * k
    } else {
        0.0
    };
    let ch4 = ch4_bio + obs.abiotic_ch4_ppm;
    // Red edge: land producers seen through the clouds.
    let land_plants = bio.is_some_and(|b| {
        b.guilds
            .iter()
            .any(|g| g.role == "producer" && g.grade == "plant")
    });
    let edge_area = if land_plants && air {
        obs.land_fraction * (1.0 - obs.cloud_cover)
    } else {
        0.0
    };
    let edge_nm = (edge_area >= p.edge_floor)
        .then(|| 700.0 * m::sqrt(sse_core::sheet::T_SUN / obs.star_teff_k.max(2000.0)));
    let mut signals = vec![];
    let see_o2 = air && o2 >= p.o2_floor;
    let see_ch4 = air && ch4 >= p.ch4_floor_ppm;
    let cool = obs.star_teff_k < p.m_dwarf_teff_k;
    if see_o2 && see_ch4 {
        signals.push(Signal {
            id: "o2_ch4".into(),
            value: o2,
            lr: p.lr_o2_ch4,
            why: "oxygen and methane together destroy each other within decades; something replenishes both".into(),
        });
    } else if see_o2 {
        signals.push(Signal {
            id: "o2".into(),
            value: o2,
            lr: if cool { p.lr_o2_m_dwarf } else { p.lr_o2_alone },
            why: if cool {
                "oxygen without methane round a red dwarf: water loss in the star's bright youth leaves oxygen behind without life".into()
            } else {
                "oxygen without methane: photosynthesis is the likely source, though photolysis can make some".into()
            },
        });
    } else if see_ch4 && !obs.reducing_air {
        signals.push(Signal {
            id: "ch4".into(),
            value: ch4,
            lr: p.lr_ch4_alone,
            why:
                "methane alone: microbes make it, but so do hot rocks and water (serpentinisation)"
                    .into(),
        });
    }
    if let Some(nm) = edge_nm {
        signals.push(Signal {
            id: "red_edge".into(),
            value: nm,
            lr: p.lr_edge,
            why: format!(
                "reflectance jumps near {nm:.0} nm over land, like leaves; minerals rarely do"
            ),
        });
    }
    let blind = if sealed {
        Some(format!(
            "an ice shell {:.0} km thick hides any ocean below; a remote scan sees nothing either way",
            obs.ice_shell_km.unwrap_or(0.0)
        ))
    } else if !air {
        Some("no atmosphere to carry gases; only the surface itself can be read".into())
    } else if obs.cloud_cover > 0.95 {
        Some("clouds hide the surface; only the gases can be read".into())
    } else {
        None
    };
    if signals.is_empty() {
        signals.push(Signal {
            id: "nothing".into(),
            value: 0.0,
            lr: if blind.is_some() { 1.0 } else { p.lr_nothing },
            why: match &blind {
                Some(b) => format!("nothing seen, and {b}"),
                None => "nothing seen where a scan could have seen it".into(),
            },
        });
    }
    let lr: f64 = signals.iter().map(|s| s.lr).product();
    let odds = p.prior / (1.0 - p.prior) * lr;
    let confidence = odds / (1.0 + odds);
    let candidate = signals.iter().any(|s| s.id != "nothing") && confidence > p.prior;
    let colour = edge_nm.and_then(|_| bio.and_then(|b| b.traits.producer_colour.clone()));
    let seasonal = colour.is_some() && obs.obliquity_deg > 10.0;
    // Surveyed: a summary, guild cards by production, the hazard.
    let (summary, guild_cards, hazard, biomass_gc) = match bio {
        Some(b) => {
            let mut g: Vec<_> = b.guilds.iter().collect();
            g.sort_by(|x, y| {
                y.production_gc_yr
                    .total_cmp(&x.production_gc_yr)
                    .then(y.intake_gc_yr.total_cmp(&x.intake_gc_yr))
                    .then(x.id.cmp(&y.id))
            });
            (
                Some(format!(
                    "{} biosphere, {} guilds, about {:.0} lineages",
                    b.budget.biomass_class,
                    b.guilds.len(),
                    b.diversity.lineages
                )),
                g.iter().take(6).map(|x| x.plain_name.clone()).collect(),
                Some(b.traits.to_visitors.clone()),
                b.budget.biomass_gc,
            )
        }
        None => (None, vec![], None, 0.0),
    };
    let traces = history.map(|h| traces(h, obs, p)).unwrap_or_default();
    Signatures {
        alive,
        ever_alive,
        colour,
        seasonal,
        o2,
        ch4_ppm: ch4,
        edge_nm,
        signals,
        candidate,
        confidence,
        blind,
        summary,
        guild_cards,
        hazard,
        biomass_gc,
        traces,
        evidence: "plausible reconstruction".into(),
    }
}

/// Traces of the past and how much of each survives: the record from `t`
/// ago is kept with probability exp(-t / recycling) x exp(-t / erosion).
fn traces(h: &LifeHistory, obs: &Observables, p: &SignatureParams) -> Vec<Trace> {
    let left = |at: f64| {
        let t = ((obs.age_myr - at) / 1000.0).max(0.0);
        let r = obs.recycling_gyr.map(|g| m::exp(-t / g)).unwrap_or(1.0);
        let e = obs.erosion_gyr.map(|g| m::exp(-t / g)).unwrap_or(1.0);
        r * e
    };
    let class = |x: f64| {
        if x >= p.abundant {
            "abundant"
        } else if x >= p.scattered {
            "scattered"
        } else if x >= p.rare {
            "rare"
        } else {
            "erased"
        }
    };
    let mut out = vec![];
    let mut push = |kind: &str, ev: &str, at: f64| {
        let s = left(at);
        out.push(Trace {
            kind: kind.into(),
            from_event: ev.into(),
            at_myr: at,
            surviving: s,
            class: class(s).into(),
        });
    };
    if let Some(o) = h.origin_myr {
        push("isotopes", "origin", o);
        push("stromatolites", "origin", o);
    }
    if let Some(&t) = h.first.get("oxygenic") {
        push("banded_iron", "oxygenic", t);
    }
    if let Some(&t) = h.first.get("large_bodies") {
        push("fossils", "large_bodies", t);
    }
    out
}

/// The result as facts by tier (field to JSON value), for a world's address.
pub fn facts(s: &Signatures) -> BTreeMap<String, serde_json::Value> {
    use serde_json::json;
    let mut f = BTreeMap::new();
    let mut put = |k: &str, v: serde_json::Value| {
        if !v.is_null() {
            f.insert(k.to_string(), v);
        }
    };
    put("gm.life.alive", json!(s.alive));
    put("gm.life.ever_alive", json!(s.ever_alive));
    put("life.obvious.colour", json!(s.colour));
    put("life.obvious.seasonal", json!(s.seasonal));
    put("life.signature.o2", json!(s.o2));
    put("life.signature.ch4_ppm", json!(s.ch4_ppm));
    put("life.signature.edge_nm", json!(s.edge_nm));
    put(
        "life.signature.signals",
        json!(s.signals.iter().map(|x| x.id.clone()).collect::<Vec<_>>()),
    );
    put("life.signature.candidate", json!(s.candidate));
    put("life.signature.confidence", json!(s.confidence));
    put("life.signature.blind", json!(s.blind));
    put("life.survey.summary", json!(s.summary));
    put("life.survey.guilds", json!(s.guild_cards));
    put("life.survey.hazard", json!(s.hazard));
    put("life.survey.biomass_gc", json!(s.biomass_gc));
    put(
        "life.past.traces",
        json!(s
            .traces
            .iter()
            .map(|t| format!("{} ({})", t.kind, t.class))
            .collect::<Vec<_>>()),
    );
    put("life.evidence", json!(s.evidence));
    f
}

/// The disclosure tier of each life field pattern, for a campaign's data
/// views (`provides.other.observe.disclosure.fields`).
pub fn disclosure() -> BTreeMap<String, String> {
    [
        ("life.obvious.*", "obvious"),
        ("life.signature.*", "scanned"),
        ("life.survey.*", "surveyed"),
        ("life.past.*", "surveyed"),
        ("life.evidence", "scanned"),
        ("gm.life.*", "secret"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}
