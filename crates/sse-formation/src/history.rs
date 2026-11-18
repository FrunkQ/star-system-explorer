//! The FormationHistory record (design section 5) and its time slices.
//!
//! A history is made once from its manifest (seed, system address, envelope,
//! pins, suite version). The time ruler reads stored slices and interpolates
//! between them; it never recomputes. Any slice reproduces from the manifest.

use crate::hurley::Fits;
use crate::inherit::{Inheritance, InheritancePins, PatchEnvelope};
use crate::lifeline::{Level, LifeLine, Phase, StarState};
use crate::stars::{form_stars, roche_lobe_fraction, Star, StarPins};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Event, EventBody, Evidence, Payload, Publisher, SessionStamp, UniverseTime};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::{Address, ContentHash, Seed};
use std::collections::BTreeMap;

pub const FORMAT: &str = "sse.formation-history/1";
pub const SUITE: &str = "sse.formation";
pub const SUITE_VERSION: &str = "0.1.0";
/// Latest date the chronicle can hold, years (2^63 s is about 292 Gyr).
pub const CHRONICLE_LIMIT_YEARS: f64 = 2.5e11;
/// Years per Myr.
const MYR: f64 = 1.0e6;

/// Everything needed to make the history again, on any device.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub suite: String,
    pub version: String,
    pub seed: String,
    pub system: Address,
    pub envelope: PatchEnvelope,
    #[serde(default)]
    pub inherit_pins: InheritancePins,
    #[serde(default)]
    pub star_pins: StarPins,
}

impl Manifest {
    pub fn new(seed: &str, system: Address, envelope: PatchEnvelope) -> Self {
        Manifest {
            suite: SUITE.into(),
            version: SUITE_VERSION.into(),
            seed: seed.into(),
            system,
            envelope,
            inherit_pins: InheritancePins::default(),
            star_pins: StarPins::default(),
        }
    }
    pub fn hash(&self) -> ContentHash {
        ContentHash::of(&serde_json::to_string(self).expect("json")).expect("hash")
    }
}

/// How a value was worked out (design principle 6).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "how", rename_all = "kebab-case")]
pub enum How {
    Calculated {
        source: String,
    },
    Tabulated {
        source: String,
    },
    Drawn {
        distribution: String,
        spread: String,
    },
    Authored,
    Reconstructed,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StageLevel {
    pub stage: String,
    pub level: Level,
    pub block: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HistoryEvent {
    /// Myr from the system's birth.
    pub t_myr: f64,
    pub kind: String,
    pub subject: Address,
    pub text: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Slice {
    pub t_myr: f64,
    pub stars: Vec<StarState>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Budgets {
    /// Mass that ended up in stars, solar masses.
    pub stellar_mass: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FormationHistory {
    pub format: String,
    pub manifest: Manifest,
    pub manifest_hash: String,
    pub inheritance: Inheritance,
    pub budgets: Budgets,
    pub stars: Vec<Star>,
    pub events: Vec<HistoryEvent>,
    pub slices: Vec<Slice>,
    /// Per value path: how it was worked out.
    pub evidence: BTreeMap<String, How>,
    pub levels: Vec<StageLevel>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FormationError(pub String);

/// Slice times, Myr from birth: dense through the first 300 Myr, then every
/// 250 Myr to the horizon, plus every phase boundary and "now".
pub fn slice_times(stars: &[Star], now: f64, horizon: f64) -> Vec<f64> {
    let mut t = vec![0.0, 0.1, 0.3, 1.0, 3.0];
    let mut x = 10.0;
    while x <= 300.0 {
        t.push(x);
        x += 10.0;
    }
    let mut x = 500.0;
    while x <= horizon {
        t.push(x);
        x += 250.0;
    }
    for s in stars {
        for b in s.life.boundaries() {
            if b <= horizon {
                t.push(b);
            }
        }
    }
    if now <= horizon {
        t.push(now);
    }
    t.sort_by(f64::total_cmp);
    t.dedup();
    t
}

/// First age (Myr) at which `life`'s radius exceeds `limit` solar radii, by
/// bisection on the pure life line (fixed 60 steps, so identical everywhere).
fn first_overflow(fits: &Fits, life: &LifeLine, limit: f64) -> Option<f64> {
    let r = |t: f64| life.state_at(fits, t).radius;
    // Search the phases where radius grows: main sequence to the end of the giants.
    let (mut lo, mut hi) = (life.t_zams, life.t_end - 1e-9 * life.t_end);
    if r(hi) <= limit {
        return None;
    }
    if r(lo) > limit {
        return Some(lo);
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if r(mid) > limit {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi)
}

const AU_IN_RSUN: f64 = 215.032;

impl FormationHistory {
    /// Stages 1 and 2 (F0). Later stages add to the record.
    pub fn form(manifest: Manifest) -> Result<Self, FormationError> {
        let seed = Seed::from_text(&manifest.seed);
        let inh = manifest
            .envelope
            .inherit(&seed, &manifest.system, &manifest.inherit_pins);
        if !(1.0e-4..=0.03).contains(&inh.z) {
            return Err(FormationError(format!(
                "metal fraction Z = {:.2e} ([Fe/H] {:.2}) is outside the stellar fits (1e-4 to 0.03)",
                inh.z, inh.feh
            )));
        }
        let fits = Fits::new(inh.z);
        let stars = form_stars(&fits, &seed, &manifest.system, &manifest.star_pins)
            .map_err(|e| FormationError(e.0))?;
        let now = inh.age_gyr * 1000.0;
        let last_end = stars.iter().map(|s| s.life.t_end).fold(0.0, f64::max);
        let horizon = (last_end + 1000.0).min(20_000.0).max(now);

        let mut events = vec![];
        for s in &stars {
            let a = &s.address;
            let l = &s.life;
            events.push(ev(
                0.0,
                "star.born",
                a,
                format!("{:.2} solar-mass star forms from the cloud core", s.mass),
            ));
            events.push(ev(
                l.t_zams,
                "star.zams",
                a,
                "hydrogen burning begins: the star joins the main sequence".into(),
            ));
            events.push(ev(
                l.t_tms,
                "star.leaves_main_sequence",
                a,
                "core hydrogen runs out".into(),
            ));
            events.push(ev(l.t_bgb, "star.giant", a, "swells into a giant".into()));
            let end = match l.remnant {
                Phase::WhiteDwarf => format!(
                    "sheds its envelope and leaves a {:.2} solar-mass white dwarf",
                    l.remnant_mass
                ),
                Phase::NeutronStar => {
                    "explodes as a core-collapse supernova, leaving a neutron star".into()
                }
                _ => "collapses, leaving a black hole".into(),
            };
            events.push(ev(l.t_end, "star.end", a, end));
        }
        // Close pairs: the first star to fill its Roche lobe at periastron.
        for (i, s) in stars.iter().enumerate().skip(1) {
            let o = s.orbit.as_ref().expect("companions orbit");
            let peri = o.semi_major_axis_au * (1.0 - o.eccentricity) * AU_IN_RSUN;
            let inner = &stars[i - 1];
            let mut cands = vec![];
            if let Some(t) = first_overflow(
                &fits,
                &s.life,
                peri * roche_lobe_fraction(s.mass / o.inner_mass),
            ) {
                cands.push((t, s));
            }
            if i == 1 {
                if let Some(t) = first_overflow(
                    &fits,
                    &inner.life,
                    peri * roche_lobe_fraction(inner.mass / s.mass),
                ) {
                    cands.push((t, inner));
                }
            }
            cands.sort_by(|x, y| x.0.total_cmp(&y.0));
            if let Some((t, donor)) = cands.first() {
                events.push(ev(
                    *t,
                    "binary.mass_transfer",
                    &donor.address,
                    "fills its Roche lobe and starts pouring gas onto its companion (outcome rules arrive with F7)".into(),
                ));
            }
        }
        events.sort_by(|x, y| {
            x.t_myr
                .total_cmp(&y.t_myr)
                .then(x.subject.to_string().cmp(&y.subject.to_string()))
        });

        let slices = slice_times(&stars, now, horizon)
            .into_iter()
            .map(|t| Slice {
                t_myr: t,
                stars: stars.iter().map(|s| s.life.state_at(&fits, t)).collect(),
            })
            .collect();

        let mut evidence = BTreeMap::new();
        let drawn = |d: &sse_resolve::Distribution| How::Drawn {
            distribution: serde_json::to_string(d).expect("json"),
            spread: "see distribution".into(),
        };
        let e = &manifest.envelope;
        let p = &manifest.inherit_pins;
        for (k, d, pin) in [
            ("inheritance.feh", &e.feh, p.feh),
            ("inheritance.age_gyr", &e.age_gyr, p.age_gyr),
            ("inheritance.c_o", &e.c_o, p.c_o),
            ("inheritance.mg_si", &e.mg_si, p.mg_si),
            ("inheritance.al26_rel", &e.al26_rel, p.al26_rel),
            (
                "inheritance.cluster_members",
                &e.cluster_members,
                p.cluster_members,
            ),
            ("inheritance.fuv_g0", &e.fuv_g0, p.fuv_g0),
        ] {
            evidence.insert(
                k.to_string(),
                if pin.is_some() {
                    How::Authored
                } else {
                    drawn(d)
                },
            );
        }
        for (i, s) in stars.iter().enumerate() {
            let k = format!("stars.{i}");
            let mass = if i == 0 && manifest.star_pins.primary_mass.is_some() {
                How::Authored
            } else if i == 0 {
                How::Drawn {
                    distribution: crate::stars::REF_KROUPA.into(),
                    spread: "0.1 to 100 solar masses".into(),
                }
            } else {
                How::Drawn {
                    distribution: crate::stars::REF_RAGHAVAN.into(),
                    spread: "mass ratio 0.1 to 1".into(),
                }
            };
            evidence.insert(format!("{k}.mass"), mass);
            evidence.insert(
                format!("{k}.main_sequence"),
                How::Calculated { source: "Hurley, Pols and Tout 2000, MNRAS 315, 543; Tout et al. 1996, MNRAS 281, 257".into() },
            );
            evidence.insert(
                format!("{k}.remnant_mass"),
                How::Calculated { source: "Cummings et al. 2018, ApJ 866, 21 (white dwarfs); sketch above 8 solar masses".into() },
            );
            if s.orbit.is_some() {
                evidence.insert(
                    format!("{k}.orbit"),
                    How::Drawn {
                        distribution: crate::stars::REF_RAGHAVAN.into(),
                        spread: "log P sigma 2.28 dex".into(),
                    },
                );
            }
        }
        let levels = vec![
            StageLevel {
                stage: "1 inheritance".into(),
                level: Level::Standard,
                block: "keyed draws from the patch envelope".into(),
            },
            StageLevel {
                stage: "2 stars: main sequence".into(),
                level: Level::Standard,
                block: "Hurley 2000 fits".into(),
            },
            StageLevel {
                stage: "2 stars: after the main sequence".into(),
                level: Level::Sketch,
                block: "phase rules on the Hurley fits; F7 replaces".into(),
            },
            StageLevel {
                stage: "2 stars: multiplicity".into(),
                level: Level::Standard,
                block: "Duchene and Kraus 2013, Raghavan 2010".into(),
            },
        ];
        let manifest_hash = manifest.hash().hex();
        Ok(FormationHistory {
            format: FORMAT.into(),
            budgets: Budgets {
                stellar_mass: stars.iter().map(|s| s.mass).sum(),
            },
            manifest,
            manifest_hash,
            inheritance: inh,
            stars,
            events,
            slices,
            evidence,
            levels,
        })
    }

    pub fn fits(&self) -> Fits {
        Fits::new(self.inheritance.z)
    }

    pub fn hash(&self) -> ContentHash {
        ContentHash::of(&serde_json::to_string(self).expect("json")).expect("hash")
    }

    /// Star states at `t` Myr, read from the stored slices (log-linear between
    /// neighbours in the same phase; the earlier slice across a phase change).
    pub fn state_at(&self, t: f64) -> Vec<StarState> {
        let s = &self.slices;
        let i = s.partition_point(|x| x.t_myr <= t);
        if i == 0 {
            return s[0].stars.clone();
        }
        if i == s.len() {
            return s[i - 1].stars.clone();
        }
        let (a, b) = (&s[i - 1], &s[i]);
        let f = (t - a.t_myr) / (b.t_myr - a.t_myr);
        a.stars
            .iter()
            .zip(&b.stars)
            .map(|(x, y)| {
                if x.phase != y.phase {
                    return x.clone();
                }
                let lerp =
                    |p: f64, q: f64| m::pow(10.0, m::log10(p) + f * (m::log10(q) - m::log10(p)));
                let (l, r) = (lerp(x.luminosity, y.luminosity), lerp(x.radius, y.radius));
                StarState {
                    luminosity: l,
                    radius: r,
                    teff: crate::lifeline::teff(l, r),
                    ..x.clone()
                }
            })
            .collect()
    }

    /// The history's events as chronicle events, dated in universe time.
    /// Universe time ends near 290 Gyr; later events (the far future of red
    /// dwarfs) stay in the history only.
    pub fn chronicle_events(&self) -> Vec<Event> {
        let born_years = self.inheritance.born_gyr * 1.0e9;
        let publisher = Publisher::new(SUITE, "formation.f0", SUITE_VERSION);
        self.events
            .iter()
            .filter(|e| born_years + e.t_myr * MYR < CHRONICLE_LIMIT_YEARS)
            .enumerate()
            .map(|(i, e)| {
                let at = UniverseTime::years(m::round(born_years + e.t_myr * MYR) as i64);
                let mut data = BTreeMap::new();
                data.insert("kind".to_string(), Value::Text(e.kind.clone()));
                data.insert("text".to_string(), Value::Text(e.text.clone()));
                data.insert("t_myr".to_string(), Value::Float(e.t_myr));
                Event::new(EventBody {
                    kind: format!("formation.{}", e.kind),
                    subject: vec![e.subject.clone()],
                    occurred_at: at,
                    activated_at: at,
                    recognised_at: SessionStamp(i as u64),
                    causes: vec![],
                    payload: Payload {
                        data: Some(Value::Map(data)),
                        ..Default::default()
                    },
                    evidence: Evidence::Resolved,
                    publisher: publisher.clone(),
                })
                .expect("formation events are well formed")
            })
            .collect()
    }
}

fn ev(t: f64, kind: &str, subject: &Address, text: String) -> HistoryEvent {
    HistoryEvent {
        t_myr: t,
        kind: kind.into(),
        subject: subject.clone(),
        text,
    }
}
