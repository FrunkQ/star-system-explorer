//! The FormationHistory record (design section 5) and its time slices.
//!
//! A history is made once from its manifest (seed, system address, envelope,
//! pins, suite version). The time ruler reads stored slices and interpolates
//! between them; it never recomputes. Any slice reproduces from the manifest.

use crate::disc::*;
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
    pub source: Source,
    #[serde(default)]
    pub inherit_pins: InheritancePins,
    #[serde(default)]
    pub star_pins: StarPins,
}

/// Where stage 1 comes from.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Source {
    /// Drawn here from a patch envelope (the stand-in fixture, or a GM's own).
    Envelope { envelope: Box<PatchEnvelope> },
    /// Resolved by the cosmos suite (G0 onward): the system's birth facts, read
    /// at universe time `now_gyr`.
    Birth {
        birth: sse_cosmos::SystemBirth,
        now_gyr: f64,
    },
}

impl Manifest {
    /// A system drawn from a patch envelope.
    pub fn new(seed: &str, system: Address, envelope: PatchEnvelope) -> Self {
        Self::with_source(
            seed,
            system,
            Source::Envelope {
                envelope: Box::new(envelope),
            },
        )
    }
    /// A system the cosmos suite resolved (its address is the birth's).
    pub fn from_birth(seed: &str, birth: sse_cosmos::SystemBirth, now_gyr: f64) -> Self {
        Self::with_source(
            seed,
            birth.address.clone(),
            Source::Birth { birth, now_gyr },
        )
    }
    fn with_source(seed: &str, system: Address, source: Source) -> Self {
        Manifest {
            suite: SUITE.into(),
            version: SUITE_VERSION.into(),
            seed: seed.into(),
            system,
            source,
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
    /// Handed over by another suite, with its own evidence there.
    Inherited {
        from: String,
    },
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
    /// The gas disc while it lasts (stage 3).
    #[serde(default)]
    pub disc: Option<DiscSlice>,
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
    /// Stage 3: the disc, its bands and its ledger.
    pub disc: Disc,
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
pub fn slice_times(stars: &[Star], now: f64, horizon: f64, disc_end: f64) -> Vec<f64> {
    let mut t = vec![0.0, 0.1, 0.2, 0.3, 0.5, 1.0, 2.0, 3.0, 5.0, disc_end];
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
    if life.is_substellar() {
        return None;
    }
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

const AU_IN_RSUN: f64 = sse_core::sheet::AU / sse_core::sheet::R_SUN;
/// Metal range of the Hurley fits.
pub const Z_FITS: (f64, f64) = (1.0e-4, 0.03);

impl FormationHistory {
    /// Stages 1 and 2 (F0). Later stages add to the record.
    pub fn form(manifest: Manifest) -> Result<Self, FormationError> {
        let seed = Seed::from_text(&manifest.seed);
        let inh = match &manifest.source {
            Source::Envelope { envelope } => {
                envelope.inherit(&seed, &manifest.system, &manifest.inherit_pins)
            }
            Source::Birth { birth, now_gyr } => Inheritance::from_birth(birth, *now_gyr),
        };
        // The stellar fits cover Z = 1e-4 to 0.03. Outside, the nearest edge is
        // used and the evidence says so (metal-rich thin-disc and the most
        // metal-poor halo stars).
        let fits_z = inh.z.clamp(Z_FITS.0, Z_FITS.1);
        let fits = Fits::new(fits_z);
        let stars = form_stars(
            &fits,
            &seed,
            &manifest.system,
            &manifest.star_pins,
            inh.system_mass,
        )
        .map_err(|e| FormationError(e.0))?;
        let now = inh.age_gyr * 1000.0;
        let last_end = stars.iter().map(|s| s.life.t_end).fold(0.0, f64::max);
        let horizon = (last_end + 1000.0).min(20_000.0).max(now);

        let mut events = vec![];
        for s in &stars {
            let a = &s.address;
            let l = &s.life;
            if l.is_substellar() {
                events.push(ev(
                    0.0,
                    "brown_dwarf.born",
                    a,
                    format!(
                        "{:.3} solar-mass brown dwarf forms: too light ever to burn hydrogen",
                        s.mass
                    ),
                ));
                continue;
            }
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
            let giant = l.t_end > l.t_bgb;
            if giant {
                events.push(ev(l.t_bgb, "star.giant", a, "swells into a giant".into()));
            }
            if l.t_strip < l.t_end {
                events.push(ev(
                    l.t_strip,
                    "star.stripped",
                    a,
                    "grows brighter than any cool supergiant can be and sheds its hydrogen envelope: a hot Wolf-Rayet star".into(),
                ));
            }
            let end = match l.remnant {
                Phase::WhiteDwarf if !giant => format!(
                    "has burnt nearly all its hydrogen and settles, without ever swelling, as a {:.2} solar-mass helium white dwarf",
                    l.remnant_mass
                ),
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

        // Stage 3: the disc round the primary, or round a close pair.
        let companion = stars
            .get(1)
            .and_then(|s| s.orbit.as_ref())
            .map(|o| (o.semi_major_axis_au, o.eccentricity));
        let cb = companion.is_some_and(|(a, _)| a < 2.0);
        let (host_m, host_l) = if cb {
            (
                stars[0].mass + stars[1].mass,
                stars[0].life.l_zams + stars[1].life.l_zams,
            )
        } else {
            (stars[0].mass, stars[0].life.l_zams.max(1.0e-4))
        };
        let disc = Disc::new(disc_params(
            &seed,
            &manifest.system,
            host_m,
            host_l,
            inh.feh,
            inh.alpha_fe,
            inh.c_o,
            inh.fuv_g0,
            companion,
        ));
        let water = LINES
            .iter()
            .position(|l| l.0 == "water")
            .expect("water line");
        let end = disc.slice(disc.t_disperse);
        events.push(ev(0.0, "disc.forms", &manifest.system, format!(
            "a disc of {:.3} solar masses of gas and dust settles round the {}; water ice freezes out beyond {:.1} AU",
            disc.params.mass0,
            if cb { "pair" } else { "star" },
            disc.slice(0.0).lines[water]
        )));
        events.push(ev(
            disc.t_disperse,
            "disc.clears",
            &manifest.system,
            format!(
                "the gas disc clears after {:.1} Myr; the water ice line had crept in to {:.1} AU",
                disc.t_disperse, end.lines[water]
            ),
        ));
        events.sort_by(|x, y| {
            x.t_myr
                .total_cmp(&y.t_myr)
                .then(x.subject.to_string().cmp(&y.subject.to_string()))
        });

        let slices = slice_times(&stars, now, horizon, disc.t_disperse)
            .into_iter()
            .map(|t| Slice {
                t_myr: t,
                stars: stars.iter().map(|s| s.life.state_at(&fits, t)).collect(),
                disc: (t < disc.t_disperse).then(|| disc.slice(t)),
            })
            .collect();

        let mut evidence = BTreeMap::new();
        if fits_z != inh.z {
            evidence.insert(
                "stars.metallicity_for_fits".to_string(),
                How::Calculated {
                    source: format!(
                        "Z = {:.2e} is outside the Hurley fits (1e-4 to 0.03): stars use Z = {fits_z:.2e}, the nearest edge",
                        inh.z
                    ),
                },
            );
        }
        let drawn = |d: &sse_resolve::Distribution| How::Drawn {
            distribution: serde_json::to_string(d).expect("json"),
            spread: "see distribution".into(),
        };
        match &manifest.source {
            Source::Envelope { envelope: e } => {
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
            }
            Source::Birth { .. } => {
                for k in [
                    "feh",
                    "alpha_fe",
                    "age_gyr",
                    "c_o",
                    "mg_si",
                    "al26_rel",
                    "r_process_rel",
                    "k40_rel",
                    "cluster_members",
                    "fuv_g0",
                    "system_mass",
                ] {
                    evidence.insert(
                        format!("inheritance.{k}"),
                        How::Inherited {
                            from: "resolved by the cosmos suite (G0 patch and cohort envelopes)"
                                .into(),
                        },
                    );
                }
            }
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
        for (k, src) in [
            ("disc.structure", REF_HARTMANN),
            ("disc.temperature", REF_NAKAMOTO),
            ("disc.lines", REF_OBERG),
            ("disc.mix", REF_LODDERS),
            ("disc.lifetime", REF_OWEN),
        ] {
            evidence.insert(k.into(), How::Calculated { source: src.into() });
        }
        evidence.insert(
            "disc.mass0".into(),
            How::Drawn {
                distribution: "log-normal, median 0.03 of the star, 0.4 dex, capped at 0.15".into(),
                spread: REF_ANDREWS.into(),
            },
        );
        evidence.insert(
            "disc.rc0".into(),
            How::Drawn {
                distribution: "log-normal, median 30 AU, 0.3 dex, truncated by a companion".into(),
                spread: REF_ANDREWS.into(),
            },
        );
        let levels = vec![
            StageLevel {
                stage: "3 disc and material bands".into(),
                level: Level::Standard,
                block: "similarity-solution viscous disc, irradiated and accretion-heated, photoevaporative clearing".into(),
            },
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
            disc,
            events,
            slices,
            evidence,
            levels,
        })
    }

    pub fn fits(&self) -> Fits {
        Fits::new(self.inheritance.z.clamp(Z_FITS.0, Z_FITS.1))
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
